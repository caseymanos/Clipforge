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
