// Tauri commands for screen recording

use crate::recording::{
    PlatformRecorder, RecordingConfig, RecordingSource, RecordingState, ScreenRecorder, SourceTypeFilter,
};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;
use log::{info, error};

/// Global recording service state
pub struct RecordingService {
    recorder: Arc<Mutex<PlatformRecorder>>,
}

impl RecordingService {
    pub fn new() -> Self {
        Self {
            recorder: Arc::new(Mutex::new(PlatformRecorder::new())),
        }
    }

    pub async fn get_recorder(&self) -> tokio::sync::MutexGuard<'_, PlatformRecorder> {
        self.recorder.lock().await
    }

    pub fn get_recorder_arc(&self) -> Arc<Mutex<PlatformRecorder>> {
        Arc::clone(&self.recorder)
    }
}

/// List available recording sources filtered by type
#[tauri::command]
pub async fn list_recording_sources(
    service: State<'_, RecordingService>,
    filter: Option<SourceTypeFilter>,
) -> Result<Vec<RecordingSource>, String> {
    let filter = filter.unwrap_or(SourceTypeFilter::All);
    info!("Command: list_recording_sources (filter: {:?})", filter);

    let recorder = service.get_recorder().await;

    recorder
        .list_sources(filter)
        .await
        .map_err(|e| {
            error!("Failed to list recording sources: {}", e);
            e.to_string()
        })
}

/// Check if we have screen recording permissions
#[tauri::command]
pub async fn check_recording_permissions(
    service: State<'_, RecordingService>,
) -> Result<bool, String> {
    info!("Command: check_recording_permissions");

    let recorder = service.get_recorder().await;

    recorder
        .check_permissions()
        .await
        .map_err(|e| {
            error!("Failed to check permissions: {}", e);
            e.to_string()
        })
}

/// Request screen recording permissions
#[tauri::command]
pub async fn request_recording_permissions(
    service: State<'_, RecordingService>,
) -> Result<bool, String> {
    info!("Command: request_recording_permissions");

    let recorder = service.get_recorder().await;

    recorder
        .request_permissions()
        .await
        .map_err(|e| {
            error!("Failed to request permissions: {}", e);
            e.to_string()
        })
}

/// Start screen recording
#[tauri::command]
pub async fn start_recording(
    app: AppHandle,
    service: State<'_, RecordingService>,
    source: RecordingSource,
    config: RecordingConfig,
) -> Result<(), String> {
    info!("Command: start_recording for source: {}", source.name());

    let mut recorder = service.get_recorder().await;

    recorder
        .start_recording(&source, config)
        .await
        .map_err(|e| {
            error!("Failed to start recording: {}", e);
            e.to_string()
        })?;

    // Emit event that recording started
    let _ = app.emit("recording:started", ());

    // Start duration update task
    let recorder_arc = service.get_recorder_arc();
    let app_clone = app.clone();

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            let recorder = recorder_arc.lock().await;
            let state = recorder.get_state();

            if !state.is_recording() {
                break;
            }

            let duration = recorder.get_duration();
            drop(recorder); // Release lock before emitting

            // Emit duration update
            let _ = app_clone.emit("recording:duration", serde_json::json!({
                "duration": duration
            }));
        }
    });

    Ok(())
}

/// Stop screen recording
#[tauri::command]
pub async fn stop_recording(
    app: AppHandle,
    service: State<'_, RecordingService>,
) -> Result<String, String> {
    info!("Command: stop_recording");

    let mut recorder = service.get_recorder().await;

    let output_path = recorder
        .stop_recording()
        .await
        .map_err(|e| {
            error!("Failed to stop recording: {}", e);
            e.to_string()
        })?;

    let output_path_str = output_path.to_string_lossy().to_string();

    // Emit event that recording stopped with file_path in payload
    let _ = app.emit("recording:stopped", serde_json::json!({
        "file_path": output_path_str.clone()
    }));

    info!("Recording saved to: {}", output_path_str);
    Ok(output_path_str)
}

/// Get current recording state
#[tauri::command]
pub async fn get_recording_state(
    service: State<'_, RecordingService>,
) -> Result<RecordingState, String> {
    let recorder = service.get_recorder().await;
    Ok(recorder.get_state())
}

/// Get current recording duration
#[tauri::command]
pub async fn get_recording_duration(
    service: State<'_, RecordingService>,
) -> Result<f64, String> {
    let recorder = service.get_recorder().await;
    Ok(recorder.get_duration())
}

/// List available audio input devices (macOS only for now)
/// Returns a list of (device_id, device_name) tuples
#[tauri::command]
pub async fn list_audio_devices() -> Result<Vec<(String, String)>, String> {
    info!("Command: list_audio_devices");

    #[cfg(target_os = "macos")]
    {
        use crate::recording::macos::MacOSRecorder;
        use crate::ffmpeg_utils;

        let ffmpeg_path = ffmpeg_utils::find_ffmpeg_path()
            .map_err(|e| {
                error!("Failed to find FFmpeg: {}", e);
                e.to_string()
            })?;

        MacOSRecorder::get_audio_devices(&ffmpeg_path)
            .map_err(|e| {
                error!("Failed to list audio devices: {}", e);
                e.to_string()
            })
    }

    #[cfg(not(target_os = "macos"))]
    {
        // For non-macOS platforms, return empty list for now
        // TODO: Implement for Windows and Linux
        warn!("Audio device listing not yet implemented for this platform");
        Ok(Vec::new())
    }
}

/// Check if a file exists at the given path
#[tauri::command]
pub fn file_exists(path: String) -> Result<bool, String> {
    use std::path::Path;
    Ok(Path::new(&path).exists())
}

/// Composite webcam overlay onto screen recording
///
/// Takes two separate video files (screen and webcam) and composites them
/// using the provided overlay configuration. Returns the path to the composite video.
#[tauri::command]
pub async fn composite_webcam_recording(
    app: AppHandle,
    ffmpeg: State<'_, crate::ffmpeg::FFmpegService>,
    screen_path: String,
    webcam_path: String,
    output_path: String,
    overlay_config: crate::recording::WebcamOverlayConfig,
) -> Result<String, String> {
    use std::path::PathBuf;

    info!("Command: composite_webcam_recording");
    info!("  Screen: {}", screen_path);
    info!("  Webcam: {}", webcam_path);
    info!("  Output: {}", output_path);
    info!("  Overlay config: {:?}", overlay_config);

    let screen_path = PathBuf::from(&screen_path);
    let webcam_path = PathBuf::from(&webcam_path);
    let output_path = PathBuf::from(&output_path);

    // Validate input files exist
    if !screen_path.exists() {
        let err_msg = format!("Screen recording file not found: {}", screen_path.display());
        error!("{}", err_msg);
        return Err(err_msg);
    }
    if !webcam_path.exists() {
        let err_msg = format!("Webcam recording file not found: {}", webcam_path.display());
        error!("{}", err_msg);
        return Err(err_msg);
    }

    // Log file sizes for debugging
    if let Ok(metadata) = std::fs::metadata(&screen_path) {
        info!("  Screen file size: {} bytes", metadata.len());
    }
    if let Ok(metadata) = std::fs::metadata(&webcam_path) {
        info!("  Webcam file size: {} bytes", metadata.len());
    }

    // Ensure output directory exists
    if let Some(parent) = output_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            let err_msg = format!("Failed to create output directory: {}", e);
            error!("{}", err_msg);
            return Err(err_msg);
        }
    }

    // Create progress callback that emits events
    let app_clone = app.clone();
    let progress_callback = Arc::new(move |progress: f64| {
        let _ = app_clone.emit("compositing:progress", serde_json::json!({
            "progress": progress
        }));
    });

    // Perform compositing
    info!("Starting FFmpeg composite operation...");
    ffmpeg
        .composite_webcam(
            &screen_path,
            &webcam_path,
            &output_path,
            &overlay_config,
            Some(progress_callback),
        )
        .await
        .map_err(|e| {
            error!("FFmpeg composite operation failed: {}", e);
            format!("Failed to composite webcam recording: {}", e)
        })?;

    // Validate output file was created
    if !output_path.exists() {
        let err_msg = format!("Composite file was not created: {}", output_path.display());
        error!("{}", err_msg);
        return Err(err_msg);
    }

    // Validate output file has content
    if let Ok(metadata) = std::fs::metadata(&output_path) {
        let file_size = metadata.len();
        info!("  Composite file size: {} bytes", file_size);
        if file_size == 0 {
            let err_msg = format!("Composite file is empty: {}", output_path.display());
            error!("{}", err_msg);
            return Err(err_msg);
        }
    }

    // Emit completion event
    let _ = app.emit("compositing:complete", serde_json::json!({
        "output_path": output_path.to_string_lossy().to_string()
    }));

    info!("Compositing complete: {}", output_path.display());
    Ok(output_path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recording_service_creation() {
        let service = RecordingService::new();
        // Test passes if no panic
        drop(service);
    }
}
