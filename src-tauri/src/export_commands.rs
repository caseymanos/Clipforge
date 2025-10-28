use crate::models::{Timeline, ExportSettings, ExportProgress, MediaFile};
use crate::export::ExportService;
use tauri::{State, Window};
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Shared export service state
pub struct ExportServiceState {
    pub service: Arc<Mutex<ExportService>>,
}

/// Export a timeline to a video file
#[tauri::command]
pub async fn export_timeline(
    timeline: Timeline,
    settings: ExportSettings,
    output_path: String,
    media_files_map: HashMap<String, MediaFile>,
    service_state: State<'_, ExportServiceState>,
    window: Window,
) -> Result<String, String> {
    let service = service_state.service.lock().await;

    let output = PathBuf::from(output_path);

    service.export_timeline(
        &timeline,
        &settings,
        output,
        &media_files_map,
        window,
    )
    .await
    .map(|p| p.to_string_lossy().to_string())
    .map_err(|e| e.to_string())
}

/// Cancel an ongoing export
#[tauri::command]
pub async fn cancel_export(
    service_state: State<'_, ExportServiceState>,
) -> Result<(), String> {
    let service = service_state.service.lock().await;
    service.cancel_export();
    Ok(())
}

/// Get available export presets
#[tauri::command]
pub async fn get_export_presets() -> Result<Vec<(String, ExportSettings)>, String> {
    Ok(ExportService::get_presets())
}

/// Validate timeline before export
#[tauri::command]
pub async fn validate_timeline_for_export(
    timeline: Timeline,
    media_files_map: HashMap<String, MediaFile>,
) -> Result<bool, String> {
    // Basic validation
    let has_video = timeline.tracks.iter()
        .any(|t| t.enabled && !t.clips.is_empty());

    if !has_video {
        return Err("Timeline must have at least one enabled track with clips".to_string());
    }

    // Check all clips have valid media files
    for track in &timeline.tracks {
        for clip in &track.clips {
            if !media_files_map.contains_key(&clip.media_file_id) {
                return Err(format!("Clip references missing media file: {}", clip.media_file_id));
            }
        }
    }

    Ok(true)
}
