// macOS screen recording implementation using FFmpeg with screen capture

use super::{AudioInputType, RecordingConfig, RecordingError, RecordingMode, RecordingSource, RecordingState, ScreenRecorder, SourceTypeFilter};
use crate::ffmpeg_utils;
#[cfg(target_os = "macos")]
use super::voice_processing::{start_voice_processing_capture, VoiceProcHandle};
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
    process: Option<Child>,             // Screen recording process
    webcam_process: Option<Child>,      // Webcam process (for dual mode)
    output_path: Option<PathBuf>,
    webcam_output_path: Option<PathBuf>, // Webcam output path
    temp_video_path: Option<PathBuf>,    // Video-only temp path when VPIO enabled
    temp_audio_path: Option<PathBuf>,    // WAV captured via VPIO
    voice_handle: Option<VoiceProcHandle>,
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
                webcam_process: None,         // NEW
                output_path: None,
                webcam_output_path: None,     // NEW
                temp_video_path: None,
                temp_audio_path: None,
                voice_handle: None,
                start_time: None,
            })),
            ffmpeg_path,
        }
    }

    /// Get available screen capture devices via FFmpeg device list
    fn get_screen_devices(ffmpeg_path: &PathBuf) -> Result<Vec<(String, String)>, RecordingError> {
        info!("Discovering screen devices via FFmpeg device list...");

        // Run: ffmpeg -f avfoundation -list_devices true -i ""
        let output = Command::new(ffmpeg_path)
            .arg("-f")
            .arg("avfoundation")
            .arg("-list_devices")
            .arg("true")
            .arg("-i")
            .arg("")
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| RecordingError::SystemError(format!("Failed to list devices: {}", e)))?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let mut devices = Vec::new();
        let mut in_video_section = false;

        // Parse lines like:
        // [AVFoundation indev @ ...] AVFoundation video devices:
        // [AVFoundation indev @ ...] [6] Capture screen 0
        for line in stderr.lines() {
            if line.contains("AVFoundation video devices:") {
                in_video_section = true;
                continue;
            }
            if in_video_section && line.contains("AVFoundation audio devices:") {
                break;
            }

            if in_video_section && line.contains("] [") {
                if let Some(bracket_pos) = line.rfind("] [") {
                    let after = &line[bracket_pos + 3..];
                    if let Some(close) = after.find(']') {
                        let id = after[..close].trim();
                        let name = after[close + 2..].trim();
                        if name.starts_with("Capture screen") {
                            info!("Found screen device: [{}] {}", id, name);
                            devices.push((id.to_string(), name.to_string()));
                        }
                    }
                }
            }
        }

        if devices.is_empty() {
            warn!("No 'Capture screen' devices detected via FFmpeg; falling back to default");
            return Ok(vec![("0".to_string(), "Capture screen 0".to_string())]);
        }

        Ok(devices)
    }

    /// Get available webcam/camera devices via FFmpeg
    ///
    /// Get list of available webcam devices via FFmpeg device list
    fn get_webcam_devices(ffmpeg_path: &PathBuf) -> Result<Vec<(String, String)>, RecordingError> {
        info!("Discovering webcam devices via FFmpeg device list...");

        // Run: ffmpeg -f avfoundation -list_devices true -i ""
        let output = Command::new(ffmpeg_path)
            .arg("-f")
            .arg("avfoundation")
            .arg("-list_devices")
            .arg("true")
            .arg("-i")
            .arg("")
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| RecordingError::SystemError(format!("Failed to list devices: {}", e)))?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let mut devices = Vec::new();
        let mut in_video_section = false;

        // Parse output like:
        // [AVFoundation indev @ 0x...] AVFoundation video devices:
        // [AVFoundation indev @ 0x...] [0] MacBook Pro Camera
        // [AVFoundation indev @ 0x...] [1] OBS Virtual Camera
        // ...
        // [AVFoundation indev @ 0x...] AVFoundation audio devices:
        for line in stderr.lines() {
            if line.contains("AVFoundation video devices:") {
                in_video_section = true;
                continue;
            }

            if line.contains("AVFoundation audio devices:") {
                break;  // Stop at audio section
            }

            if in_video_section && line.contains("] [") {
                // Extract device ID and name from lines like: "[AVFoundation...] [0] MacBook Pro Camera"
                if let Some(bracket_pos) = line.rfind("] [") {
                    let after_bracket = &line[bracket_pos + 3..];  // Skip "] ["
                    if let Some(close_bracket) = after_bracket.find(']') {
                        let device_id = &after_bracket[..close_bracket];
                        let device_name = after_bracket[close_bracket + 2..].trim();

                        // Skip "Capture screen" devices (those are handled separately)
                        if device_name.starts_with("Capture screen") {
                            continue;
                        }

                        // Filter out common virtual cameras (OBS, Snap Camera, etc.)
                        // Virtual cameras can cause issues and are typically not what users want
                        let virtual_camera_patterns = [
                            "Virtual Camera",
                            "OBS Virtual",
                            "Snap Camera",
                            "ManyCam",
                            "CamTwist",
                            "e2eSoft",
                            "SplitCam",
                            "XSplit",
                        ];

                        let is_virtual = virtual_camera_patterns.iter().any(|pattern| device_name.contains(pattern));
                        if is_virtual {
                            info!("Skipping virtual camera device: [{}] {}", device_id, device_name);
                            continue;
                        }

                        // Add physical webcam to list
                        info!("Found webcam device: [{}] {}", device_id, device_name);
                        devices.push((device_id.to_string(), device_name.to_string()));
                    }
                }
            }
        }

        if devices.is_empty() {
            info!("No webcam devices detected via device list");
        } else {
            info!("Found {} webcam device(s) via device list", devices.len());
        }

        Ok(devices)
    }

    /// Get available audio input devices via FFmpeg
    ///
    /// Get list of available audio input devices (microphones) via FFmpeg device list
    pub fn get_audio_devices(ffmpeg_path: &PathBuf) -> Result<Vec<(String, String)>, RecordingError> {
        info!("Discovering audio input devices via FFmpeg device list...");

        // Run: ffmpeg -f avfoundation -list_devices true -i ""
        let output = Command::new(ffmpeg_path)
            .arg("-f")
            .arg("avfoundation")
            .arg("-list_devices")
            .arg("true")
            .arg("-i")
            .arg("")
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| RecordingError::SystemError(format!("Failed to list audio devices: {}", e)))?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let mut devices = Vec::new();
        let mut in_audio_section = false;

        // Parse output like:
        // [AVFoundation indev @ 0x...] AVFoundation video devices:
        // ...
        // [AVFoundation indev @ 0x...] AVFoundation audio devices:
        // [AVFoundation indev @ 0x...] [0] MacBook Pro Microphone
        // [AVFoundation indev @ 0x...] [1] External Microphone
        for line in stderr.lines() {
            if line.contains("AVFoundation audio devices:") {
                in_audio_section = true;
                continue;
            }

            // Stop parsing if we hit another section or end
            if in_audio_section && (line.contains("AVFoundation") && line.contains("devices:") && !line.contains("audio devices:")) {
                break;
            }

            if in_audio_section && line.contains("] [") {
                // Extract device ID and name from lines like: "[AVFoundation...] [0] MacBook Pro Microphone"
                if let Some(bracket_pos) = line.rfind("] [") {
                    let after_bracket = &line[bracket_pos + 3..];  // Skip "] ["
                    if let Some(close_bracket) = after_bracket.find(']') {
                        let device_id = &after_bracket[..close_bracket];
                        let device_name = after_bracket[close_bracket + 2..].trim();

                        info!("Found audio device: [{}] {}", device_id, device_name);
                        devices.push((device_id.to_string(), device_name.to_string()));
                    }
                }
            }
        }

        if devices.is_empty() {
            info!("No audio input devices detected via device list");
        } else {
            info!("Found {} audio input device(s) via device list", devices.len());
        }

        Ok(devices)
    }

    /// Spawn FFmpeg process for webcam recording
    ///
    /// Parameters:
    /// - webcam_id: Device ID for the webcam
    /// - config: Recording configuration (includes desired output framerate)
    /// - input_framerate: Framerate to request from the webcam hardware
    fn spawn_webcam_recording(
        &self,
        webcam_id: &str,
        config: &RecordingConfig,
        input_framerate: u32,
    ) -> Result<Child, RecordingError> {
        info!("Spawning webcam recording process for device {} at {}fps input, {}fps output",
            webcam_id, input_framerate, config.fps);
        eprintln!("[RECORDING] Spawning webcam recording process for device {} at {}fps input, {}fps output",
            webcam_id, input_framerate, config.fps);

        let mut cmd = Command::new(&self.ffmpeg_path);

        // Input format (AVFoundation)
        cmd.arg("-f").arg("avfoundation");

        // Input framerate - what we request from the webcam hardware
        cmd.arg("-framerate").arg(input_framerate.to_string());

        // Input device (camera index:audio)
        // For dual recording, webcam has NO audio (mic goes to screen)
        let device_input = format!("{}:none", webcam_id);
        info!("Spawning webcam recording with device ID: {} at {}fps input -> {}fps output",
            webcam_id, input_framerate, config.fps);
        eprintln!("[RECORDING] Spawning webcam recording process for device {} at {}fps input, {}fps output",
            webcam_id, input_framerate, config.fps);
        cmd.arg("-i").arg(&device_input);

        // Video codec
        cmd.arg("-c:v").arg("libx264");
        cmd.arg("-preset").arg("ultrafast"); // Real-time encoding

        // Quality (CRF scale: lower = better quality, 18=visually lossless, 23=good, 28=acceptable)
        // Map quality 10 (best) -> CRF 18, quality 5 (medium) -> CRF 23, quality 1 (low) -> CRF 28
        let crf = 18 + ((10 - config.quality) * 1);
        cmd.arg("-crf").arg(crf.to_string());

        // Pixel format for compatibility
        cmd.arg("-pix_fmt").arg("yuv420p");

        // Output framerate - convert to user's desired framerate if different from input
        // This allows capturing at webcam's native framerate while outputting at user's choice
        if input_framerate != config.fps as u32 {
            cmd.arg("-r").arg(config.fps.to_string());
            info!("Applying framerate conversion: {}fps input -> {}fps output", input_framerate, config.fps);
        }

        // Overwrite output file
        cmd.arg("-y");

        // Output path
        cmd.arg(config.output_path.to_str().unwrap());

        // Redirect stderr to pipe (for error capture)
        cmd.stderr(Stdio::piped());
        cmd.stdout(Stdio::null());

        // Log the full command for debugging
        let webcam_args: Vec<_> = cmd.get_args().collect();
        info!("FFmpeg webcam command: {:?} {:?}", self.ffmpeg_path, webcam_args);
        eprintln!("[RECORDING] FFmpeg webcam command: {:?} {:?}", self.ffmpeg_path, webcam_args);

        // Spawn process
        let result = cmd.spawn()
            .map_err(|e| {
                eprintln!("[RECORDING ERROR] Failed to spawn webcam FFmpeg: {}", e);
                RecordingError::RecordingFailed(format!("Failed to start webcam: {}", e))
            });

        eprintln!("[RECORDING] Webcam spawn result: {:?}", result.is_ok());
        result
    }
}


#[async_trait::async_trait]
impl ScreenRecorder for MacOSRecorder {
    async fn list_sources(&self, filter: SourceTypeFilter) -> Result<Vec<RecordingSource>, RecordingError> {
        info!("Listing macOS recording sources (filter: {:?})", filter);

        // Initialize preview generator
        let preview_generator = match crate::screen_preview::ScreenPreviewGenerator::new() {
            Ok(gen) => Some(gen),
            Err(e) => {
                warn!("Failed to initialize screen preview generator: {:?}", e);
                None
            }
        };

        let mut all_sources = Vec::new();

        // Always probe screen devices regardless of filter
        // This ensures we have full device availability information
        let screen_devices = Self::get_screen_devices(&self.ffmpeg_path)?;
        info!("Found {} screen device(s)", screen_devices.len());

        for (id, name) in screen_devices {
            // Generate preview thumbnail for this screen
            let preview_path = if let Some(ref generator) = preview_generator {
                match generator.capture_screen_preview(&id, "screen").await {
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
            all_sources.push(RecordingSource::Screen {
                id: id.clone(),
                name: name.clone(),
                width: 1920,  // Default width
                height: 1080, // Default height
                preview_path,
            });
        }

        // If no screen devices found, add at least one default screen
        if all_sources.is_empty() {
            warn!("No screen devices detected, adding default screen");
            all_sources.push(RecordingSource::Screen {
                id: "5".to_string(),
                name: "Capture screen 0".to_string(),
                width: 1920,
                height: 1080,
                preview_path: None,
            });
        }

        // Always probe webcam devices regardless of filter
        // This ensures frontend knows webcam availability for button states
        let webcam_devices = Self::get_webcam_devices(&self.ffmpeg_path)
            .unwrap_or_else(|e| {
                warn!("Failed to enumerate webcam devices: {:?}", e);
                Vec::new() // Return empty list on error, don't fail
            });

        info!("Found {} webcam device(s)", webcam_devices.len());

        for (id, name) in webcam_devices {
            // Generate preview thumbnail for this webcam
            let preview_path = if let Some(ref generator) = preview_generator {
                match generator.capture_screen_preview(&id, "webcam").await {
                    Ok(path) => {
                        info!("Generated preview for webcam {}: {:?}", id, path);
                        path.to_str().map(|s| s.to_string())
                    }
                    Err(e) => {
                        warn!("Failed to generate preview for webcam {}: {:?}", id, e);
                        None
                    }
                }
            } else {
                None
            };

            all_sources.push(RecordingSource::Webcam {
                id: id.clone(),
                name: name.clone(),
                preview_path,
            });
        }

        // Now filter the sources based on the requested filter
        let sources = match filter {
            SourceTypeFilter::Screen => {
                all_sources.into_iter()
                    .filter(|s| matches!(s, RecordingSource::Screen { .. }))
                    .collect()
            }
            SourceTypeFilter::Webcam => {
                all_sources.into_iter()
                    .filter(|s| matches!(s, RecordingSource::Webcam { .. }))
                    .collect()
            }
            SourceTypeFilter::All => all_sources,
            _ => all_sources, // Window filter not yet implemented, return all
        };

        info!("Found {} recording sources (filter: {:?})", sources.len(), filter);
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

        // Determine recording mode
        match config.recording_mode {
            RecordingMode::ScreenOnly => {
                info!("Starting screen-only recording");

                // Existing screen recording logic
                let mut cmd = Command::new(&self.ffmpeg_path);
                cmd.arg("-f").arg("avfoundation");

                if config.show_cursor {
                    cmd.arg("-capture_cursor").arg("1");
                }

                cmd.arg("-framerate").arg(config.fps.to_string());

                // Use device ID directly (e.g., "6"), not the device name (e.g., "Capture screen 6")
                let mut use_voice = false;
                let device_input = match config.audio_input {
                    AudioInputType::None => format!("{}:none", source.id()),
                    AudioInputType::Microphone => {
                        // Record screen without audio and capture mic via VoiceProcessingIO
                        use_voice = true;
                        format!("{}:none", source.id())
                    }
                    AudioInputType::SystemAudio => {
                        warn!("System audio requires BlackHole setup");
                        format!("{}:0", source.id())
                    }
                    AudioInputType::Both => {
                        warn!("Both audio sources require mixing setup");
                        format!("{}:0", source.id())
                    }
                };

                // Add thread queue size and probe settings for better real-time capture
                cmd.arg("-thread_queue_size").arg("4096");  // Increased for better audio buffering
                cmd.arg("-probesize").arg("10M");
                cmd.arg("-analyzeduration").arg("0");

                cmd.arg("-i").arg(&device_input);
                cmd.arg("-c:v").arg("libx264");
                cmd.arg("-preset").arg("ultrafast");  // Fastest encoding for real-time recording

                cmd.arg("-crf").arg("23");  // Standard quality, same as composite

                if let Some(crop) = &config.crop_region {
                    let crop_filter = format!("crop={}:{}:{}:{}", crop.width, crop.height, crop.x, crop.y);
                    cmd.arg("-vf").arg(crop_filter);
                }

                if !use_voice && config.audio_input != AudioInputType::None {
                    cmd.arg("-ar").arg("44100");
                    cmd.arg("-ac").arg("2");
                    cmd.arg("-c:a").arg("aac_at");
                    cmd.arg("-b:a").arg("192k");
                }

                cmd.arg("-pix_fmt").arg("yuv420p");
                cmd.arg("-y");
                let final_out = config.output_path.clone();
                let video_out = if use_voice {
                    let mut tmp = final_out.clone();
                    let stem = tmp.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
                    tmp.set_file_name(format!("{}-noaudio.mp4", stem));
                    cmd.arg(tmp.to_str().unwrap());
                    tmp
                } else {
                    cmd.arg(final_out.to_str().unwrap());
                    final_out.clone()
                };
                cmd.stderr(Stdio::piped());
                cmd.stdout(Stdio::null());

                // Log the full command for debugging
                info!("FFmpeg screen command: {:?} {:?}", self.ffmpeg_path, cmd.get_args().collect::<Vec<_>>());

                // Start VoiceProcessing capture if requested
                if use_voice {
                    let mut wav = video_out.clone();
                    wav.set_file_name("mic-voiceproc.wav");
                    if let Ok(h) = start_voice_processing_capture(wav.clone(), 44100.0, 1) {
                        state.temp_audio_path = Some(wav);
                        state.voice_handle = Some(h);
                    } else {
                        warn!("Failed to start VoiceProcessing capture; continuing without it");
                    }
                }

                let child = cmd.spawn()
                    .map_err(|e| RecordingError::RecordingFailed(format!("Failed to start screen recording: {}", e)))?;

                state.process = Some(child);
                if use_voice {
                    state.temp_video_path = Some(video_out);
                    state.output_path = Some(final_out);
                } else {
                    state.output_path = Some(final_out);
                }
            }

            RecordingMode::WebcamOnly => {
                info!("Starting webcam-only recording");

                let webcam_source = config.webcam_source.as_ref()
                    .ok_or(RecordingError::InvalidConfig("Webcam source required for webcam mode".into()))?;

                let webcam_id = webcam_source.id();

                // Try common webcam framerates
                let desired_fps = config.fps as u32;
                let framerates_to_try = vec![desired_fps, 60, 30, 15];

                let mut child = None;
                for &fps in &framerates_to_try {
                    if let Ok(c) = self.spawn_webcam_recording(webcam_id, &config, fps) {
                        child = Some(c);
                        break;
                    }
                }

                let child = child.ok_or_else(|| {
                    RecordingError::RecordingFailed(format!(
                        "Failed to start webcam recording with any framerate"
                    ))
                })?;

                state.process = Some(child);
                state.output_path = Some(config.output_path.clone());
            }

            RecordingMode::ScreenAndWebcam => {
                info!("Starting dual recording (screen + webcam)");

                // Validate webcam configuration
                let webcam_source = config.webcam_source.as_ref()
                    .ok_or(RecordingError::InvalidConfig("Webcam source required for dual mode".into()))?;

                let webcam_path = config.webcam_output_path.as_ref()
                    .ok_or(RecordingError::InvalidConfig("Webcam output path required for dual mode".into()))?;

                // Create parent directory for webcam file
                if let Some(parent) = webcam_path.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        RecordingError::SystemError(format!("Failed to create webcam output directory: {}", e))
                    })?;
                }

                // Spawn screen recording process (with mic audio)
                // Use device ID directly (e.g., "6"), not the device name (e.g., "Capture screen 6")
                let mut screen_cmd = Command::new(&self.ffmpeg_path);
                screen_cmd.arg("-f").arg("avfoundation");

                if config.show_cursor {
                    screen_cmd.arg("-capture_cursor").arg("1");
                }

                screen_cmd.arg("-framerate").arg(config.fps.to_string());

                let mut use_voice_dual = false;
                let device_input = match config.audio_input {
                    AudioInputType::Microphone => {
                        use_voice_dual = true;
                        format!("{}:none", source.id())
                    }
                    _ => format!("{}:none", source.id()), // Default to no audio if not mic
                };

                // Add thread queue size and probe settings for better real-time capture
                screen_cmd.arg("-thread_queue_size").arg("4096");  // Increased for better audio buffering
                screen_cmd.arg("-probesize").arg("10M");
                screen_cmd.arg("-analyzeduration").arg("0");

                screen_cmd.arg("-i").arg(&device_input);
                screen_cmd.arg("-c:v").arg("libx264");
                screen_cmd.arg("-preset").arg("ultrafast");  // Fastest encoding for real-time recording

                screen_cmd.arg("-crf").arg("23");  // Standard quality, same as composite

                if !use_voice_dual && config.audio_input == AudioInputType::Microphone {
                    screen_cmd.arg("-ar").arg("44100");
                    screen_cmd.arg("-ac").arg("2");
                    screen_cmd.arg("-c:a").arg("aac_at");
                    screen_cmd.arg("-b:a").arg("192k");
                }

                screen_cmd.arg("-pix_fmt").arg("yuv420p");
                screen_cmd.arg("-y");
                let final_out = config.output_path.clone();
                let video_out = if use_voice_dual {
                    let mut tmp = final_out.clone();
                    let stem = tmp.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
                    tmp.set_file_name(format!("{}-noaudio.mp4", stem));
                    screen_cmd.arg(tmp.to_str().unwrap());
                    tmp
                } else {
                    screen_cmd.arg(final_out.to_str().unwrap());
                    final_out.clone()
                };
                screen_cmd.stderr(Stdio::piped());
                screen_cmd.stdout(Stdio::null());

                // Log the full command for debugging
                let screen_args: Vec<_> = screen_cmd.get_args().collect();
                info!("FFmpeg screen command (dual mode): {:?} {:?}", self.ffmpeg_path, screen_args);
                eprintln!("[RECORDING] FFmpeg screen command (dual mode): {:?} {:?}", self.ffmpeg_path, screen_args);

                if use_voice_dual {
                    let mut wav = video_out.clone();
                    wav.set_file_name("mic-voiceproc.wav");
                    if let Ok(h) = start_voice_processing_capture(wav.clone(), 44100.0, 1) {
                        state.temp_audio_path = Some(wav);
                        state.voice_handle = Some(h);
                    } else {
                        warn!("Failed to start VoiceProcessing capture; continuing without it");
                    }
                }

                let screen_child = screen_cmd.spawn()
                    .map_err(|e| {
                        eprintln!("[RECORDING ERROR] Failed to spawn screen FFmpeg: {}", e);
                        RecordingError::RecordingFailed(format!("Failed to start screen: {}", e))
                    })?;

                // Spawn webcam recording process (NO audio)
                let mut webcam_config = config.clone();
                webcam_config.output_path = webcam_path.clone();
                webcam_config.audio_input = AudioInputType::None; // Force no audio on webcam

                // Validate webcam output directory exists before spawning process
                if let Some(parent_dir) = webcam_path.parent() {
                    if !parent_dir.exists() {
                        info!("Creating webcam output directory: {:?}", parent_dir);
                        std::fs::create_dir_all(parent_dir)
                            .map_err(|e| {
                                let error_msg = format!("Failed to create webcam output directory {:?}: {}", parent_dir, e);
                                error!("{}", error_msg);
                                eprintln!("[WEBCAM ERROR] {}", error_msg);
                                RecordingError::RecordingFailed(error_msg)
                            })?;
                        info!("Webcam output directory created successfully: {:?}", parent_dir);
                    } else {
                        info!("Webcam output directory exists: {:?}", parent_dir);
                    }
                } else {
                    let error_msg = format!("Invalid webcam output path (no parent directory): {:?}", webcam_path);
                    error!("{}", error_msg);
                    eprintln!("[WEBCAM ERROR] {}", error_msg);
                    return Err(RecordingError::RecordingFailed(error_msg));
                }

                info!("Starting webcam recording: device_id={}, output={}", webcam_source.id(), webcam_path.display());
                eprintln!("[WEBCAM] Starting webcam recording: device_id={}, output={}", webcam_source.id(), webcam_path.display());

                // Try multiple framerates with health check fallback
                // Try user's requested framerate first, then common webcam framerates (60, 30, 15)
                let desired_fps = config.fps as u32;
                let mut framerates_to_try = vec![desired_fps];

                // Add fallback framerates if they're different from desired
                for fps in [60, 30, 15] {
                    if fps != desired_fps && !framerates_to_try.contains(&fps) {
                        framerates_to_try.push(fps);
                    }
                }

                let mut webcam_child = None;
                let mut last_error = String::new();

                for (attempt, &input_framerate) in framerates_to_try.iter().enumerate() {
                    info!("Webcam recording attempt {} with {}fps input", attempt + 1, input_framerate);
                    eprintln!("[WEBCAM] Attempt {} with {}fps input", attempt + 1, input_framerate);

                    match self.spawn_webcam_recording(webcam_source.id(), &webcam_config, input_framerate) {
                        Ok(mut child) => {
                            info!("Webcam process spawned successfully (PID: {:?})", child.id());
                            eprintln!("[WEBCAM] Process spawned successfully (PID: {:?})", child.id());

                            // Health check: Wait briefly and verify the process is still alive
                            std::thread::sleep(std::time::Duration::from_millis(500));
                            match child.try_wait() {
                                Ok(Some(status)) => {
                                    // Process has already exited - capture stderr
                                    let mut stderr_output = String::new();
                                    if let Some(mut stderr) = child.stderr.take() {
                                        use std::io::Read;
                                        let _ = stderr.read_to_string(&mut stderr_output);
                                    }

                                    // Check if this is a framerate error
                                    let is_framerate_error = stderr_output.contains("framerate")
                                        || stderr_output.contains("Supported modes");

                                    last_error = format!(
                                        "{}fps: Process crashed (status: {}). FFmpeg stderr: {}",
                                        input_framerate,
                                        status,
                                        if stderr_output.is_empty() { "No output" } else { &stderr_output }
                                    );

                                    if is_framerate_error && attempt < framerates_to_try.len() - 1 {
                                        warn!("Webcam doesn't support {}fps, trying next framerate", input_framerate);
                                        eprintln!("[WEBCAM] {}fps not supported, trying next framerate", input_framerate);
                                        continue; // Try next framerate
                                    } else {
                                        error!("Webcam process failed: {}", last_error);
                                        eprintln!("[WEBCAM ERROR] {}", last_error);
                                        if attempt == framerates_to_try.len() - 1 {
                                            break; // All framerates failed, will return error below
                                        }
                                    }
                                }
                                Ok(None) => {
                                    // Process is still running - healthy!
                                    info!("Webcam health check passed at {}fps - process is running", input_framerate);
                                    eprintln!("[WEBCAM] Health check passed at {}fps", input_framerate);
                                    webcam_child = Some(child);
                                    break; // Success!
                                }
                                Err(e) => {
                                    // Error checking process status
                                    warn!("Failed to check webcam process status at {}fps: {}", input_framerate, e);
                                    eprintln!("[WEBCAM WARNING] Status check failed: {}", e);
                                    webcam_child = Some(child);
                                    break; // Proceed anyway if we can't check status
                                }
                            }
                        }
                        Err(e) => {
                            last_error = format!("{}fps: Failed to spawn: {}", input_framerate, e);
                            warn!("Failed to spawn webcam process at {}fps: {}", input_framerate, e);
                            eprintln!("[WEBCAM ERROR] Spawn failed at {}fps: {}", input_framerate, e);
                            if attempt == framerates_to_try.len() - 1 {
                                break; // Last attempt failed
                            }
                        }
                    }
                }

                // Check if we successfully started the webcam
                let mut webcam_child = match webcam_child {
                    Some(child) => child,
                    None => {
                        let error_msg = format!(
                            "Failed to start webcam recording after trying {} framerate(s). Last error: {}",
                            framerates_to_try.len(),
                            last_error
                        );
                        error!("{}", error_msg);
                        eprintln!("[WEBCAM ERROR] {}", error_msg);
                        return Err(RecordingError::RecordingFailed(error_msg));
                    }
                };

                state.process = Some(screen_child);
                state.webcam_process = Some(webcam_child);
                state.output_path = Some(config.output_path.clone());
                if use_voice_dual { state.temp_video_path = Some(video_out); }
                state.webcam_output_path = Some(webcam_path.clone());

                eprintln!("[RECORDING] Dual recording fully initialized - screen: {:?}, webcam: {:?}",
                    state.process.as_ref().map(|p| p.id()),
                    state.webcam_process.as_ref().map(|p| p.id()));
            }
        }

        state.start_time = Some(Instant::now());
        state.state = RecordingState::Recording;

        info!("Recording started successfully");
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

        // Get webcam process if it exists (for dual recording)
        let webcam_process = state.webcam_process.take();
        let webcam_output_path = state.webcam_output_path.take();

        // VoiceProcessing temp artifacts (if used)
        let temp_video_path = state.temp_video_path.take();
        let temp_audio_path = state.temp_audio_path.take();
        let voice_handle = state.voice_handle.take();

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

        // Log stderr BEFORE checking file (helps debug immediate failures)
        if !stderr_output.is_empty() {
            info!("FFmpeg stderr output ({} bytes): {}", stderr_output.len(), stderr_output);
        } else {
            warn!("FFmpeg produced no stderr output (this may indicate immediate failure)");
        }

        match wait_result {
            Ok(status) => {
                info!("Recording stopped, exit status: {}", status);
            }
            Err(e) => {
                warn!("Error waiting for recording process: {}", e);
            }
        }

        // Stop webcam process if exists
        if let Some(mut webcam_process) = webcam_process {
            info!("Stopping webcam recording process");
            eprintln!("[WEBCAM] Stopping webcam recording process");

            #[cfg(unix)]
            unsafe {
                libc::kill(webcam_process.id() as i32, libc::SIGINT);
            }

            // Read stderr for debugging - ALWAYS capture and log it
            let mut webcam_stderr = String::new();
            if let Some(mut stderr) = webcam_process.stderr.take() {
                use std::io::Read;
                let _ = stderr.read_to_string(&mut webcam_stderr);
            }

            // Wait for process to exit and check status
            // Note: FFmpeg returns exit code 255 when gracefully stopped via SIGINT - this is NORMAL
            let webcam_status = match webcam_process.wait() {
                Ok(status) => {
                    // Check if this is a graceful shutdown (exit code 255 after SIGINT)
                    let is_graceful_shutdown = status.code() == Some(255);

                    if status.success() || is_graceful_shutdown {
                        info!("Webcam process exited successfully: {:?}", status);
                        eprintln!("[WEBCAM] Process exited successfully: {:?}", status);
                    } else {
                        error!("Webcam process exited with unexpected error status: {}. FFmpeg stderr: {}",
                            status,
                            if webcam_stderr.is_empty() { "No output" } else { &webcam_stderr });
                        eprintln!("[WEBCAM ERROR] Process failed with status: {}. FFmpeg stderr: {}",
                            status,
                            if webcam_stderr.is_empty() { "No output" } else { &webcam_stderr });
                    }
                    Some(status)
                }
                Err(e) => {
                    error!("Error waiting for webcam process: {}", e);
                    eprintln!("[WEBCAM ERROR] Failed to wait for process: {}", e);
                    None
                }
            };

            // Always log stderr if present, even on success (may contain warnings)
            if !webcam_stderr.is_empty() {
                warn!("Webcam FFmpeg stderr output ({} bytes): {}",
                    webcam_stderr.len(),
                    webcam_stderr);
                eprintln!("[WEBCAM] FFmpeg stderr ({} bytes): {}",
                    webcam_stderr.len(),
                    webcam_stderr);
            } else if let Some(status) = webcam_status {
                // Check if process failed unexpectedly (not exit code 255)
                let is_graceful_shutdown = status.code() == Some(255);
                if !status.success() && !is_graceful_shutdown {
                    error!("Webcam process failed but produced no stderr output (process may have crashed)");
                    eprintln!("[WEBCAM ERROR] Process failed with no stderr (possible crash)");
                }
            }
        }

        // Verify webcam file if it was being recorded
        if let Some(webcam_path) = webcam_output_path {
            for attempt in 0..5 {
                if webcam_path.exists() && webcam_path.metadata().map(|m| m.len() > 0).unwrap_or(false) {
                    info!("Webcam recording file verified: {:?}", webcam_path);
                    break;
                }

                if attempt < 4 {
                    warn!("Webcam file not ready, waiting... (attempt {})", attempt + 1);
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }

        // Stop VoiceProcessing and mux audio if we recorded via VPIO
        if let Some(h) = voice_handle {
            h.stop();
            if let (Some(video_no_audio), Some(wav_path)) = (temp_video_path.as_ref(), temp_audio_path.as_ref()) {
                // Mux: copy video, encode audio to AAC (Apple AAC)
                let mut mux = Command::new(&self.ffmpeg_path);
                mux.args([
                    "-i", video_no_audio.to_str().unwrap(),
                    "-i", wav_path.to_str().unwrap(),
                    "-map", "0:v:0",
                    "-map", "1:a:0",
                    "-c:v", "copy",
                    "-c:a", "aac_at",
                    "-b:a", "192k",
                    "-y",
                    output_path.to_str().unwrap(),
                ]);
                let _ = mux.status();
                // Clean up temp files
                let _ = std::fs::remove_file(video_no_audio);
                let _ = std::fs::remove_file(wav_path);
            }
        }

        // State was already set to Idle earlier, no need to update again

        // Verify the output file was actually created by FFmpeg or mux
        // FFmpeg needs time to finalize the MP4 moov atom after receiving SIGINT
        // Retry up to 20 times with 100ms delays (total 2000ms max wait)
        let mut file_exists = false;
        for attempt in 0..20 {
            if output_path.exists() {
                file_exists = true;
                info!("Output file found after {}ms (attempt {})", attempt * 100, attempt + 1);
                break;
            }
            if attempt < 19 {
                if attempt < 5 {
                    // Log first 5 attempts at INFO level
                    info!("Output file not found yet, waiting 100ms (attempt {}/20)", attempt + 1);
                } else if attempt % 5 == 0 {
                    // Log every 5th attempt after that
                    warn!("Output file still not found after {}ms (attempt {}/20)", attempt * 100, attempt + 1);
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }

        if !file_exists {
            error!("Recording file was not created by FFmpeg after 2000ms: {}", output_path.display());
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
