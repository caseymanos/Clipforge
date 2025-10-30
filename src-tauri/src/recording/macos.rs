// macOS screen recording implementation using FFmpeg with screen capture

use super::{AudioInputType, RecordingConfig, RecordingError, RecordingSource, RecordingState, ScreenRecorder};
use crate::ffmpeg_utils;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use log::{info, warn, error};

/// macOS screen recorder using FFmpeg's avfoundation device
///
/// This implementation uses FFmpeg's built-in avfoundation device to capture
/// the screen, which is simpler than direct AVFoundation bindings and provides
/// better cross-platform consistency.
pub struct MacOSRecorder {
    state: Arc<Mutex<RecorderState>>,
    ffmpeg_path: PathBuf,
}

struct RecorderState {
    state: RecordingState,
    process: Option<Child>,
    output_path: Option<PathBuf>,
    start_time: Option<Instant>,
}

impl MacOSRecorder {
    pub fn new() -> Self {
        // Find FFmpeg path during initialization
        let ffmpeg_path = ffmpeg_utils::find_ffmpeg_path()
            .unwrap_or_else(|e| {
                warn!("Failed to find FFmpeg: {}. Recording features will be unavailable.", e);
                PathBuf::from("ffmpeg") // Fallback to PATH lookup
            });

        info!("MacOSRecorder initialized with FFmpeg at: {:?}", ffmpeg_path);

        Self {
            state: Arc::new(Mutex::new(RecorderState {
                state: RecordingState::Idle,
                process: None,
                output_path: None,
                start_time: None,
            })),
            ffmpeg_path,
        }
    }

    /// Get available screen capture devices
    ///
    /// Uses Core Graphics to enumerate actual connected displays.
    /// The FFmpeg device IDs for AVFoundation screens start after video input devices,
    /// typically at index 5 or higher. We map CG display IDs to FFmpeg device indices.
    fn get_screen_devices(_ffmpeg_path: &PathBuf) -> Result<Vec<(String, String)>, RecordingError> {
        // Core Graphics display enumeration using raw FFI
        // This is a minimal implementation to avoid adding external dependencies
        extern "C" {
            fn CGGetActiveDisplayList(
                max_displays: u32,
                active_displays: *mut u32,
                display_count: *mut u32,
            ) -> i32;
        }

        let mut displays = [0u32; 10];  // Support up to 10 displays
        let mut display_count: u32 = 0;

        // Call Core Graphics to get active displays
        let result = unsafe {
            CGGetActiveDisplayList(10, displays.as_mut_ptr(), &mut display_count)
        };

        if result != 0 {
            error!("Failed to enumerate displays via Core Graphics: error code {}", result);
            // Fallback: assume at least one screen
            return Ok(vec![("0".to_string(), "Capture screen 0".to_string())]);
        }

        info!("Core Graphics found {} active display(s)", display_count);

        // FFmpeg AVFoundation device IDs for screens typically start at index 5
        // (after cameras/video devices). We create a sequential mapping.
        let mut devices = Vec::new();
        for i in 0..display_count {
            let screen_index = i;
            let device_name = format!("Capture screen {}", screen_index);
            // FFmpeg device ID is the screen index (0, 1, 2, etc.)
            devices.push((screen_index.to_string(), device_name));
            info!("Screen {}: {} (FFmpeg device ID: {})", i, devices[i as usize].1, screen_index);
        }

        if devices.is_empty() {
            warn!("No displays detected, adding default screen");
            devices.push(("0".to_string(), "Capture screen 0".to_string()));
        }

        Ok(devices)
    }
}


#[async_trait::async_trait]
impl ScreenRecorder for MacOSRecorder {
    async fn list_sources(&self) -> Result<Vec<RecordingSource>, RecordingError> {
        info!("Listing macOS recording sources");

        let devices = Self::get_screen_devices(&self.ffmpeg_path)?;

        // Initialize preview generator
        let preview_generator = match crate::screen_preview::ScreenPreviewGenerator::new() {
            Ok(gen) => Some(gen),
            Err(e) => {
                warn!("Failed to initialize screen preview generator: {:?}", e);
                None
            }
        };

        let mut sources = Vec::new();

        for (id, name) in devices {
            // Generate preview thumbnail for this screen
            let preview_path = if let Some(ref generator) = preview_generator {
                match generator.capture_screen_preview(&id).await {
                    Ok(path) => {
                        info!("Generated preview for screen {}: {:?}", id, path);
                        path.to_str().map(|s| s.to_string())
                    }
                    Err(e) => {
                        warn!("Failed to generate preview for screen {}: {:?}", id, e);
                        None
                    }
                }
            } else {
                None
            };

            // Parse screen resolution (default to common resolution)
            sources.push(RecordingSource::Screen {
                id: id.clone(),
                name: name.clone(),
                width: 1920,  // Default width
                height: 1080, // Default height
                preview_path,
            });
        }

        // If no devices found, add at least one default screen
        if sources.is_empty() {
            warn!("No screen devices detected, adding default screen");
            sources.push(RecordingSource::Screen {
                id: "1".to_string(),
                name: "Capture screen 0".to_string(),
                width: 1920,
                height: 1080,
                preview_path: None,
            });
        }

        info!("Found {} recording sources", sources.len());
        Ok(sources)
    }

    async fn check_permissions(&self) -> Result<bool, RecordingError> {
        // On macOS 10.15+, screen recording requires permission
        // We can't programmatically check this without native code,
        // so we'll assume we need to request it

        info!("Checking screen recording permissions");

        // Try to list devices - if this fails, we likely don't have permission
        match Self::get_screen_devices(&self.ffmpeg_path) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn request_permissions(&self) -> Result<bool, RecordingError> {
        info!("Requesting screen recording permissions");

        // On macOS, the system will automatically prompt when we try to record
        // We return true to indicate the user should attempt recording
        // The system dialog will appear on first recording attempt

        Ok(true)
    }

    async fn start_recording(
        &mut self,
        source: &RecordingSource,
        config: RecordingConfig,
    ) -> Result<(), RecordingError> {
        let mut state = self.state.lock().unwrap();

        if !state.state.can_start() {
            return Err(RecordingError::AlreadyRecording);
        }

        info!("Starting screen recording for source: {}", source.name());

        // Validate output path
        if config.output_path.to_str().is_none() {
            return Err(RecordingError::InvalidConfig(
                "Invalid output path".to_string(),
            ));
        }

        // Create parent directory if it doesn't exist
        if let Some(parent) = config.output_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                RecordingError::SystemError(format!("Failed to create output directory: {}", e))
            })?;
        }

        // Build FFmpeg command for screen capture
        // Format: ffmpeg -f avfoundation -capture_cursor 1 -framerate 30 -i "1" output.mp4

        let mut cmd = Command::new(&self.ffmpeg_path);

        // Input format (AVFoundation)
        cmd.arg("-f").arg("avfoundation");

        // Capture cursor
        if config.show_cursor {
            cmd.arg("-capture_cursor").arg("1");
        }

        // Frame rate
        cmd.arg("-framerate").arg(config.fps.to_string());

        // Input device (screen capture by name)
        // On macOS with AVFoundation, screens are accessed by name, not numeric index:
        // - "Capture screen 0" = first screen
        // - "Capture screen 1" = second screen, etc.
        // Format: "video_device_name:audio_device_index" or "video_device_name:none"
        //
        // IMPORTANT: Numeric indices 0-4 are camera devices! We must use the name format.
        let screen_name = format!("Capture screen {}", source.id());
        let device_input = match config.audio_input {
            AudioInputType::None => {
                format!("{}:none", screen_name) // Screen capture only (no audio)
            }
            AudioInputType::Microphone => {
                // Use audio device ID if provided, otherwise default to 0 (first audio input)
                let audio_id = config.audio_device_id.as_deref().unwrap_or("0");
                format!("{}:{}", screen_name, audio_id) // Screen + microphone
            }
            AudioInputType::SystemAudio => {
                // System audio on macOS requires additional setup (e.g., BlackHole)
                // For now, we'll use the first audio device as a fallback
                // TODO: Implement proper system audio capture with virtual audio devices
                warn!("System audio capture requires additional setup on macOS (e.g., BlackHole virtual audio device)");
                format!("{}:0", screen_name) // Screen + first audio device
            }
            AudioInputType::Both => {
                // Recording both system audio and microphone requires audio mixing
                // This is complex and typically requires virtual audio devices
                // For now, we'll just capture microphone
                // TODO: Implement audio mixing for system + microphone
                warn!("Recording both system and microphone audio requires audio mixing setup");
                format!("{}:0", screen_name) // Screen + microphone (fallback)
            }
        };
        cmd.arg("-i").arg(device_input);

        // Video codec and quality
        cmd.arg("-c:v").arg("libx264");
        cmd.arg("-preset").arg("ultrafast"); // Fast encoding for real-time
        cmd.arg("-crf").arg((51 - config.quality * 5).to_string()); // Convert 1-10 to CRF

        // Apply crop filter if region is specified
        if let Some(crop) = &config.crop_region {
            let crop_filter = format!("crop={}:{}:{}:{}", crop.width, crop.height, crop.x, crop.y);
            cmd.arg("-vf").arg(crop_filter);
            info!("Applying crop filter: {}x{} at ({}, {})", crop.width, crop.height, crop.x, crop.y);
        }

        // Audio codec (if recording audio)
        if config.audio_input != AudioInputType::None {
            cmd.arg("-c:a").arg("aac");
            cmd.arg("-b:a").arg("128k");
        }

        // Pixel format for compatibility
        cmd.arg("-pix_fmt").arg("yuv420p");

        // Overwrite output
        cmd.arg("-y");

        // Output file
        cmd.arg(&config.output_path);

        // Capture stderr for debugging (FFmpeg outputs progress/errors there)
        cmd.stderr(Stdio::piped());
        cmd.stdout(Stdio::null());

        info!("FFmpeg command: {:?}", cmd);

        // Spawn FFmpeg process
        let child = cmd.spawn().map_err(|e| {
            error!("Failed to spawn FFmpeg: {}", e);
            RecordingError::RecordingFailed(format!("Failed to start recording: {}", e))
        })?;

        state.process = Some(child);
        state.output_path = Some(config.output_path.clone());
        state.start_time = Some(Instant::now());
        state.state = RecordingState::Recording;

        info!("Screen recording started successfully");
        Ok(())
    }

    async fn stop_recording(&mut self) -> Result<PathBuf, RecordingError> {
        let mut state = self.state.lock().unwrap();

        if !state.state.can_stop() {
            return Err(RecordingError::NotRecording);
        }

        info!("Stopping screen recording");
        state.state = RecordingState::Finalizing;

        // Get the process and output path
        let mut process = state.process.take().ok_or_else(|| {
            RecordingError::RecordingFailed("No recording process found".to_string())
        })?;

        let output_path = state.output_path.take().ok_or_else(|| {
            RecordingError::RecordingFailed("No output path found".to_string())
        })?;

        // Set state to Idle immediately after taking the process
        // This allows new recordings to start while we finalize the current one
        state.state = RecordingState::Idle;
        state.start_time = None;

        // Send SIGINT to FFmpeg for graceful shutdown FIRST
        // This allows FFmpeg to finalize the video file properly
        let pid = process.id();

        drop(state); // Release lock before potentially blocking operation

        #[cfg(unix)]
        {
            info!("Sending SIGINT to FFmpeg process {}", pid);
            unsafe {
                libc::kill(pid as i32, libc::SIGINT);
            }
        }

        #[cfg(not(unix))]
        {
            // On non-Unix platforms, use kill() as fallback
            let _ = process.kill();
        }

        // Now read stderr and wait for process to exit
        let mut stderr_output = String::new();
        if let Some(mut stderr) = process.stderr.take() {
            use std::io::Read;
            let _ = stderr.read_to_string(&mut stderr_output);
        }

        // Wait for process to exit
        let wait_result = process.wait();

        match wait_result {
            Ok(status) => {
                info!("Recording stopped, exit status: {}", status);
                if !stderr_output.is_empty() {
                    info!("FFmpeg stderr (last 500 chars): {}",
                        &stderr_output.chars().rev().take(500).collect::<String>().chars().rev().collect::<String>());
                }
            }
            Err(e) => {
                warn!("Error waiting for recording process: {}", e);
                if !stderr_output.is_empty() {
                    warn!("FFmpeg stderr: {}", stderr_output);
                }
            }
        }

        // State was already set to Idle earlier, no need to update again

        // Verify the output file was actually created by FFmpeg
        // FFmpeg needs time to finalize the MP4 moov atom after receiving SIGINT
        // Retry up to 5 times with 100ms delays (total 500ms max wait)
        let mut file_exists = false;
        for attempt in 0..5 {
            if output_path.exists() {
                file_exists = true;
                break;
            }
            if attempt < 4 {
                info!("Output file not found yet, waiting 100ms (attempt {}/5)", attempt + 1);
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }

        if !file_exists {
            error!("Recording file was not created by FFmpeg after 500ms: {}", output_path.display());
            error!("FFmpeg may have failed silently. Check stderr output above.");
            return Err(RecordingError::RecordingFailed(format!(
                "Recording file not created: {}. FFmpeg may have failed during finalization.",
                output_path.display()
            )));
        }

        // Verify file has non-zero size
        match std::fs::metadata(&output_path) {
            Ok(metadata) => {
                let file_size = metadata.len();
                if file_size == 0 {
                    error!("Recording file is empty (0 bytes): {}", output_path.display());
                    return Err(RecordingError::RecordingFailed(format!(
                        "Recording file is empty. Recording may have been too short or FFmpeg failed."
                    )));
                }
                info!("Recording saved successfully: {} ({} bytes)", output_path.display(), file_size);
            }
            Err(e) => {
                error!("Cannot read recording file metadata: {}", e);
                return Err(RecordingError::RecordingFailed(format!(
                    "Cannot verify recording file: {}", e
                )));
            }
        }

        Ok(output_path)
    }

    fn get_state(&self) -> RecordingState {
        let state = self.state.lock().unwrap();
        state.state
    }

    fn get_duration(&self) -> f64 {
        let state = self.state.lock().unwrap();

        if let Some(start_time) = state.start_time {
            start_time.elapsed().as_secs_f64()
        } else {
            0.0
        }
    }
}

impl Default for MacOSRecorder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macos_recorder_creation() {
        let recorder = MacOSRecorder::new();
        assert_eq!(recorder.get_state(), RecordingState::Idle);
        assert_eq!(recorder.get_duration(), 0.0);
    }

    #[test]
    fn test_extract_device_id() {
        let line = "[AVFoundation indev @ 0x12345] [1] Capture screen 0";
        assert_eq!(extract_device_id(line), Some("1".to_string()));
    }

    #[test]
    fn test_extract_device_name() {
        let line = "[AVFoundation indev @ 0x12345] [1] Capture screen 0";
        assert_eq!(extract_device_name(line), Some("Capture screen 0".to_string()));
    }
}
