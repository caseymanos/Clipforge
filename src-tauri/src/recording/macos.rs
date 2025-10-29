// macOS screen recording implementation using FFmpeg with screen capture

use super::{AudioInputType, RecordingConfig, RecordingError, RecordingSource, RecordingState, ScreenRecorder};
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
}

struct RecorderState {
    state: RecordingState,
    process: Option<Child>,
    output_path: Option<PathBuf>,
    start_time: Option<Instant>,
}

impl MacOSRecorder {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(RecorderState {
                state: RecordingState::Idle,
                process: None,
                output_path: None,
                start_time: None,
            })),
        }
    }

    /// Get available screen capture devices using FFmpeg
    fn get_screen_devices() -> Result<Vec<(String, String)>, RecordingError> {
        // On macOS, FFmpeg can capture screens using:
        // - "Capture screen 0" (main display)
        // - "Capture screen 1" (secondary display, if available)

        // For simplicity, we'll enumerate common screen indices
        // In a production app, we'd query the system for actual screens

        let output = Command::new("ffmpeg")
            .args(&["-f", "avfoundation", "-list_devices", "true", "-i", ""])
            .output()
            .map_err(|e| RecordingError::SystemError(format!("Failed to run ffmpeg: {}", e)))?;

        // FFmpeg outputs device list to stderr
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut devices = Vec::new();

        // Parse FFmpeg output for AVFoundation devices
        // Example: "[AVFoundation indev @ 0x...] [0] FaceTime HD Camera"
        // Example: "[AVFoundation indev @ 0x...] [1] Capture screen 0"

        for line in stderr.lines() {
            if line.contains("Capture screen") {
                if let Some(device_id) = extract_device_id(line) {
                    if let Some(device_name) = extract_device_name(line) {
                        devices.push((device_id, device_name));
                    }
                }
            }
        }

        // If we couldn't parse devices, add default screens
        if devices.is_empty() {
            info!("Could not parse FFmpeg devices, using defaults");
            devices.push(("5".to_string(), "Capture screen 0".to_string()));
        }

        Ok(devices)
    }
}

/// Extract device ID from FFmpeg device list line
fn extract_device_id(line: &str) -> Option<String> {
    // Format: "[AVFoundation indev @ 0x...] [1] Capture screen 0"
    if let Some(start) = line.find('[') {
        if let Some(end) = line[start + 1..].find(']') {
            let id_str = &line[start + 1..start + 1 + end];
            // Check if it's a number
            if id_str.chars().all(|c| c.is_numeric()) {
                return Some(id_str.to_string());
            }
        }
    }
    None
}

/// Extract device name from FFmpeg device list line
fn extract_device_name(line: &str) -> Option<String> {
    // Get text after the last ']'
    if let Some(pos) = line.rfind(']') {
        let name = line[pos + 1..].trim();
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    None
}

#[async_trait::async_trait]
impl ScreenRecorder for MacOSRecorder {
    async fn list_sources(&self) -> Result<Vec<RecordingSource>, RecordingError> {
        info!("Listing macOS recording sources");

        let devices = Self::get_screen_devices()?;

        let mut sources = Vec::new();

        for (id, name) in devices {
            // Parse screen resolution (default to common resolution)
            sources.push(RecordingSource::Screen {
                id: id.clone(),
                name: name.clone(),
                width: 1920,  // Default width
                height: 1080, // Default height
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
        match Self::get_screen_devices() {
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

        let mut cmd = Command::new("ffmpeg");

        // Input format (AVFoundation)
        cmd.arg("-f").arg("avfoundation");

        // Capture cursor
        if config.show_cursor {
            cmd.arg("-capture_cursor").arg("1");
        }

        // Frame rate
        cmd.arg("-framerate").arg(config.fps.to_string());

        // Input device (screen index)
        // Format: "video_device_index:audio_device_index" or just "video_index"
        // On macOS with AVFoundation:
        // - "1:none" = screen only (no audio)
        // - "1:0" = screen + first audio input device (usually microphone)
        // - ":0" = audio only (for system audio, we'd need additional setup)
        let device_input = match config.audio_input {
            AudioInputType::None => {
                format!("{}:none", source.id()) // Video only
            }
            AudioInputType::Microphone => {
                // Use audio device ID if provided, otherwise default to 0 (first audio input)
                let audio_id = config.audio_device_id.as_deref().unwrap_or("0");
                format!("{}:{}", source.id(), audio_id) // Video + microphone
            }
            AudioInputType::SystemAudio => {
                // System audio on macOS requires additional setup (e.g., BlackHole)
                // For now, we'll use the first audio device as a fallback
                // TODO: Implement proper system audio capture with virtual audio devices
                warn!("System audio capture requires additional setup on macOS (e.g., BlackHole virtual audio device)");
                format!("{}:0", source.id()) // Video + first audio device
            }
            AudioInputType::Both => {
                // Recording both system audio and microphone requires audio mixing
                // This is complex and typically requires virtual audio devices
                // For now, we'll just capture microphone
                // TODO: Implement audio mixing for system + microphone
                warn!("Recording both system and microphone audio requires audio mixing setup");
                format!("{}:0", source.id()) // Video + microphone (fallback)
            }
        };
        cmd.arg("-i").arg(device_input);

        // Video codec and quality
        cmd.arg("-c:v").arg("libx264");
        cmd.arg("-preset").arg("ultrafast"); // Fast encoding for real-time
        cmd.arg("-crf").arg((51 - config.quality * 5).to_string()); // Convert 1-10 to CRF

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

        // Update state
        let mut state = self.state.lock().unwrap();
        state.state = RecordingState::Idle;
        state.start_time = None;

        info!("Recording saved to: {}", output_path.display());
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
