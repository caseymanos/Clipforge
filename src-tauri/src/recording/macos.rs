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
    webcam_overlay_config: Option<super::WebcamOverlayConfig>, // Configuration for compositing
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
                webcam_overlay_config: None,
                start_time: None,
            })),
            ffmpeg_path,
        }
    }

    /// Get available screen capture devices via FFmpeg device list
    fn get_screen_devices(ffmpeg_path: &PathBuf) -> Result<Vec<(String, String)>, RecordingError> {
        info!("Discovering screen devices via FFmpeg device list...");

        // First, get actual display names from system_profiler
        let display_names = Self::get_display_names();

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
        // [AVFoundation indev @ ...] [4] Capture screen 0
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
                            // Extract screen index from "Capture screen 0" -> 0
                            let screen_idx = name.trim_start_matches("Capture screen")
                                .trim()
                                .parse::<usize>()
                                .unwrap_or(0);

                            // Map to actual display name
                            let display_name = display_names.get(screen_idx)
                                .map(|s| s.as_str())
                                .unwrap_or(name);

                            info!("Found screen device: [{}] {} -> {}", id, name, display_name);
                            devices.push((id.to_string(), display_name.to_string()));
                        }
                    }
                }
            }
        }

        if devices.is_empty() {
            warn!("No 'Capture screen' devices detected via FFmpeg; falling back to default");
            return Ok(vec![("4".to_string(), "Main Display".to_string())]);
        }

        Ok(devices)
    }

    /// Get display names from system_profiler
    fn get_display_names() -> Vec<String> {
        let output = Command::new("system_profiler")
            .arg("SPDisplaysDataType")
            .arg("-json")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();

        if let Ok(output) = output {
            if let Ok(json_str) = String::from_utf8(output.stdout) {
                // Parse JSON to extract display names
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_str) {
                    if let Some(displays) = json["SPDisplaysDataType"].as_array() {
                        if let Some(first_gpu) = displays.first() {
                            if let Some(ndrvs) = first_gpu["spdisplays_ndrvs"].as_array() {
                                return ndrvs.iter()
                                    .filter_map(|display| {
                                        let name = display["_name"].as_str()?;
                                        let resolution = display["_spdisplays_pixels"].as_str().unwrap_or("");
                                        Some(format!("{} ({})", name, resolution))
                                    })
                                    .collect();
                            }
                        }
                    }
                }
            }
        }

        // Fallback
        vec![]
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

                        // Filter out OBS Virtual Camera - prefer real cameras
                        // OBS Virtual Camera can interfere with real camera access
                        if device_name.contains("OBS") && device_name.contains("Virtual") {
                            info!("Skipping OBS Virtual Camera: [{}] {}", device_id, device_name);
                            continue;
                        }

                        // Add real webcam devices only
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
    /// - audio_device_id: Optional audio device ID (None for no audio, Some("0") for microphone, etc.)
    fn spawn_webcam_recording(
        &self,
        webcam_id: &str,
        config: &RecordingConfig,
        input_framerate: u32,
        audio_device_id: Option<&str>,
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
        // Format: "webcam_id:audio_id" or "webcam_id:none"
        let device_input = if let Some(audio_id) = audio_device_id {
            format!("{}:{}", webcam_id, audio_id)
        } else {
            format!("{}:none", webcam_id)
        };
        info!("Spawning webcam recording with device ID: {} at {}fps input -> {}fps output",
            device_input, input_framerate, config.fps);
        eprintln!("[RECORDING] Spawning webcam recording with device: {} at {}fps input -> {}fps output",
            device_input, input_framerate, config.fps);
        cmd.arg("-i").arg(&device_input);

        // Video codec - use VideoToolbox hardware encoder
        cmd.arg("-c:v").arg("h264_videotoolbox");

        // Bitrate for VideoToolbox (quality 10=best, 5=medium, 1=low)
        // Map quality 10 -> 5 Mbps, quality 5 -> 3.5 Mbps, quality 1 -> 2 Mbps
        let bitrate: u32 = 2000 + (config.quality as u32 * 300);
        cmd.arg("-b:v").arg(format!("{}k", bitrate));

        // Quality setting for VideoToolbox (0-100, higher = better)
        // Lower quality = smaller file size. Quality 60-70 is good for webcam
        let quality: u32 = 40 + (config.quality as u32 * 3);  // Quality 10 -> 70, quality 5 -> 55, quality 1 -> 43
        cmd.arg("-q:v").arg(quality.to_string());

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

        // Use inherit for stderr to prevent pipe buffer issues
        cmd.stderr(Stdio::inherit());
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

                // Don't specify input framerate for screen capture - let it use native rate

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

                // Set output framerate (re-encode from native capture rate to desired FPS)
                cmd.arg("-r").arg(config.fps.to_string());

                cmd.arg("-c:v").arg("h264_videotoolbox");
                cmd.arg("-b:v").arg("8000k");  // 8 Mbps target bitrate for VideoToolbox

                // Quality setting for VideoToolbox (0-100, higher = better)
                // Lower quality = smaller file size. Quality 50 gives ~6-8 Mbps for screen content
                cmd.arg("-q:v").arg("50");  // Balanced quality for screen recording

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
                // Use inherit for stderr to prevent pipe buffer from filling up and blocking the process
                cmd.stderr(Stdio::inherit());
                cmd.stdout(Stdio::null());

                // Log the full command for debugging
                info!("FFmpeg screen command: {:?} {:?}", self.ffmpeg_path, cmd.get_args().collect::<Vec<_>>());

                // Start VoiceProcessing capture if requested
                if use_voice {
                    let mut wav = video_out.clone();
                    wav.set_file_name("mic-voiceproc.wav");
                    info!("Starting Voice Processing audio capture to: {:?}", wav);
                    match start_voice_processing_capture(wav.clone(), 44100.0, 1) {
                        Ok(h) => {
                            info!("✓ Voice Processing capture started successfully");
                            state.temp_audio_path = Some(wav);
                            state.voice_handle = Some(h);
                        }
                        Err(e) => {
                            error!("Failed to start VoiceProcessing capture: {}; continuing without it", e);
                        }
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
                    if let Ok(c) = self.spawn_webcam_recording(webcam_id, &config, fps, None) {
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

                info!("[DUAL MODE] Webcam source received: id={}, name={}", webcam_source.id(), webcam_source.name());
                eprintln!("[DUAL MODE] Webcam source received: id={}, name={}", webcam_source.id(), webcam_source.name());

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

                // Don't specify input framerate for screen capture - let it use native rate

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

                // Set output framerate (re-encode from native capture rate to desired FPS)
                screen_cmd.arg("-r").arg(config.fps.to_string());

                screen_cmd.arg("-c:v").arg("h264_videotoolbox");
                screen_cmd.arg("-b:v").arg("8000k");  // 8 Mbps target bitrate for VideoToolbox

                // Quality setting for VideoToolbox (0-100, higher = better)
                // Lower quality = smaller file size. Quality 50 gives ~6-8 Mbps for screen content
                screen_cmd.arg("-q:v").arg("50");  // Balanced quality for screen recording

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
                // Use inherit for stderr so FFmpeg output goes to parent naturally
                screen_cmd.stderr(Stdio::inherit());
                screen_cmd.stdout(Stdio::null());

                // Log the full command for debugging
                let screen_args: Vec<_> = screen_cmd.get_args().collect();
                info!("FFmpeg screen command (dual mode): {:?} {:?}", self.ffmpeg_path, screen_args);
                eprintln!("[RECORDING] FFmpeg screen command (dual mode): {:?} {:?}", self.ffmpeg_path, screen_args);

                if use_voice_dual {
                    let mut wav = video_out.clone();
                    wav.set_file_name("mic-voiceproc.wav");
                    info!("[DUAL MODE] Starting Voice Processing audio capture to: {:?}", wav);
                    match start_voice_processing_capture(wav.clone(), 44100.0, 1) {
                        Ok(h) => {
                            info!("✓ [DUAL MODE] Voice Processing capture started successfully");
                            state.temp_audio_path = Some(wav);
                            state.voice_handle = Some(h);
                        }
                        Err(e) => {
                            error!("[DUAL MODE] Failed to start VoiceProcessing capture: {}; continuing without it", e);
                        }
                    }
                }

                let mut screen_child = screen_cmd.spawn()
                    .map_err(|e| {
                        eprintln!("[RECORDING ERROR] Failed to spawn screen FFmpeg: {}", e);
                        RecordingError::RecordingFailed(format!("Failed to start screen: {}", e))
                    })?;

                // Health check: verify FFmpeg is still running after a brief moment
                std::thread::sleep(std::time::Duration::from_millis(100));
                match screen_child.try_wait() {
                    Ok(Some(status)) => {
                        let error_msg = format!("Screen FFmpeg exited immediately with status: {}", status);
                        error!("{}", error_msg);
                        eprintln!("[RECORDING ERROR] {}", error_msg);
                        return Err(RecordingError::RecordingFailed(error_msg));
                    }
                    Ok(None) => {
                        info!("✓ Screen FFmpeg health check passed");
                        eprintln!("[RECORDING] ✓ Screen FFmpeg health check passed");
                    }
                    Err(e) => {
                        warn!("Failed to check screen FFmpeg status: {}", e);
                    }
                }

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

                // Wait for any preview capture processes to fully release the camera
                // This prevents "Configuration of video device failed" errors
                info!("Waiting 800ms for camera to be released from preview processes...");
                std::thread::sleep(std::time::Duration::from_millis(800));

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
                    // Add delay between retry attempts to allow camera to reset
                    if attempt > 0 {
                        info!("Waiting 1000ms before retry attempt {}...", attempt + 1);
                        eprintln!("[WEBCAM] Waiting 1000ms before retry attempt {}...", attempt + 1);
                        std::thread::sleep(std::time::Duration::from_millis(1000));
                    }

                    info!("Webcam recording attempt {} with {}fps input", attempt + 1, input_framerate);
                    eprintln!("[WEBCAM] Attempt {} with {}fps input", attempt + 1, input_framerate);

                    match self.spawn_webcam_recording(webcam_source.id(), &webcam_config, input_framerate, None) {
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
                        // Provide user-friendly error message with actionable advice
                        let user_msg = if last_error.contains("Configuration of video device failed") {
                            "Failed to access webcam - it may be in use by another application. Please close other apps using the camera and try again."
                        } else if last_error.contains("framerate") {
                            "Webcam doesn't support the requested framerate. Please try a different quality setting."
                        } else {
                            "Failed to start webcam recording. Please check that the camera is connected and not in use by another app."
                        };

                        let detailed_error = format!(
                            "{}. Technical details: {} (tried {} framerate(s))",
                            user_msg,
                            last_error,
                            framerates_to_try.len()
                        );
                        error!("{}", detailed_error);
                        eprintln!("[WEBCAM ERROR] {}", detailed_error);
                        return Err(RecordingError::RecordingFailed(detailed_error));
                    }
                };

                state.process = Some(screen_child);
                state.webcam_process = Some(webcam_child);
                state.output_path = Some(config.output_path.clone());
                if use_voice_dual { state.temp_video_path = Some(video_out); }
                state.webcam_output_path = Some(webcam_path.clone());
                state.webcam_overlay_config = config.webcam_overlay_config.clone();

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
        let webcam_overlay_config = state.webcam_overlay_config.take();

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

        // Wait for process to exit with timeout (max 5 seconds)
        let mut waited = 0;
        loop {
            match process.try_wait() {
                Ok(Some(status)) => {
                    info!("Recording stopped, exit status: {}", status);
                    break;
                }
                Ok(None) => {
                    // Process still running
                    if waited >= 50 {  // 50 * 100ms = 5 seconds
                        warn!("Screen recording process did not exit after 5s, force killing");
                        let _ = process.kill();
                        let _ = process.wait();  // Clean up zombie
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    waited += 1;
                }
                Err(e) => {
                    warn!("Error waiting for recording process: {}", e);
                    break;
                }
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

            // Wait for process to exit with timeout (max 5 seconds)
            // Note: FFmpeg returns exit code 255 when gracefully stopped via SIGINT - this is NORMAL
            let mut webcam_waited = 0;
            loop {
                match webcam_process.try_wait() {
                    Ok(Some(status)) => {
                        let is_graceful_shutdown = status.code() == Some(255);
                        if status.success() || is_graceful_shutdown {
                            info!("Webcam process exited successfully: {:?}", status);
                            eprintln!("[WEBCAM] Process exited successfully: {:?}", status);
                        } else {
                            error!("Webcam process exited with unexpected error status: {}", status);
                            eprintln!("[WEBCAM ERROR] Process failed with status: {}", status);
                        }
                        break;
                    }
                    Ok(None) => {
                        // Process still running
                        if webcam_waited >= 50 {  // 50 * 100ms = 5 seconds
                            warn!("Webcam process did not exit after 5s, force killing");
                            eprintln!("[WEBCAM] Process did not exit after 5s, force killing");
                            let _ = webcam_process.kill();
                            let _ = webcam_process.wait();  // Clean up zombie
                            break;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        webcam_waited += 1;
                    }
                    Err(e) => {
                        error!("Error waiting for webcam process: {}", e);
                        eprintln!("[WEBCAM ERROR] Failed to wait for process: {}", e);
                        break;
                    }
                }
            }
        }

        // Verify webcam file if it was being recorded
        if let Some(ref webcam_path) = webcam_output_path {
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
                // Verify audio file exists and has content
                if let Ok(metadata) = std::fs::metadata(wav_path) {
                    log::info!("Voice Processing WAV file size: {} bytes", metadata.len());
                    if metadata.len() == 0 {
                        log::error!("Voice Processing WAV file is empty!");
                    }
                } else {
                    log::error!("Voice Processing WAV file not found at: {:?}", wav_path);
                }

                // Mux: copy video, encode audio to AAC (Apple AAC)
                log::info!("Muxing video with Voice Processing audio...");
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
                mux.stderr(std::process::Stdio::piped());

                match mux.output() {
                    Ok(output) => {
                        if output.status.success() {
                            log::info!("Successfully muxed video with audio");
                        } else {
                            log::error!("FFmpeg muxing failed with status: {:?}", output.status);
                            log::error!("FFmpeg stderr: {}", String::from_utf8_lossy(&output.stderr));
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to execute FFmpeg muxing command: {}", e);
                    }
                }

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

        // If we have both screen and webcam recordings, composite them
        if let (Some(webcam_path), Some(config)) = (webcam_output_path.as_ref(), webcam_overlay_config.as_ref()) {
            info!("Compositing screen and webcam recordings...");
            info!("  Screen: {:?}", output_path);
            info!("  Webcam: {:?}", webcam_path);

            // Verify webcam file exists and has content
            match std::fs::metadata(webcam_path) {
                Ok(metadata) if metadata.len() > 0 => {
                    info!("Webcam file verified: {} bytes", metadata.len());

                    // Create composite output path (replace screen recording with composite)
                    let composite_path = output_path.with_file_name(
                        format!("{}-composite.mp4",
                            output_path.file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("recording"))
                    );

                    // Import FFmpegService to use composite_webcam
                    use crate::ffmpeg::FFmpegService;
                    let ffmpeg_service = match FFmpegService::new() {
                        Ok(service) => service,
                        Err(e) => {
                            error!("Failed to create FFmpegService: {}", e);
                            warn!("Returning screen recording without composite");
                            return Ok(output_path);
                        }
                    };

                    // Call composite_webcam
                    match tokio::runtime::Runtime::new()
                        .unwrap()
                        .block_on(ffmpeg_service.composite_webcam(
                            &output_path,
                            webcam_path,
                            &composite_path,
                            config,
                            None, // No progress callback during stop
                            temp_audio_path.as_deref(), // Pass audio path if it exists
                        )) {
                        Ok(_) => {
                            info!("Composite created successfully: {:?}", composite_path);

                            // Clean up original screen and webcam files
                            if let Err(e) = std::fs::remove_file(&output_path) {
                                warn!("Failed to remove screen recording: {}", e);
                            }
                            if let Err(e) = std::fs::remove_file(webcam_path) {
                                warn!("Failed to remove webcam recording: {}", e);
                            }

                            // Return composite path
                            return Ok(composite_path);
                        }
                        Err(e) => {
                            error!("Failed to composite recordings: {}", e);
                            warn!("Returning screen recording without composite");
                            // Fall through to return screen recording
                        }
                    }
                }
                Ok(_) => {
                    warn!("Webcam file is empty, skipping composite");
                }
                Err(e) => {
                    warn!("Webcam file not found or inaccessible: {}", e);
                }
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
