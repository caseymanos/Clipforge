use crate::models::{Timeline, Clip, TrackType, Resolution};
use crate::timeline::TimelineService;
use tauri::State;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Shared timeline service state
pub struct TimelineServiceState {
    pub service: Arc<Mutex<TimelineService>>,
}

/// Create a new timeline
#[tauri::command]
pub async fn create_timeline(
    name: String,
    framerate: f64,
    width: u32,
    height: u32,
    state: State<'_, TimelineServiceState>,
) -> Result<Timeline, String> {
    let mut service = state.service.lock().await;
    let resolution = Resolution { width, height };

    service.create_timeline(name, framerate, resolution)
        .map_err(|e| e.to_string())
}

/// Get current timeline
#[tauri::command]
pub async fn get_current_timeline(
    state: State<'_, TimelineServiceState>,
) -> Result<Timeline, String> {
    let service = state.service.lock().await;
    service.get_timeline()
        .map(|t| t.clone())
        .map_err(|e| e.to_string())
}

/// Add a new track to the timeline
#[tauri::command]
pub async fn add_track(
    track_type: String,
    state: State<'_, TimelineServiceState>,
) -> Result<String, String> {
    let mut service = state.service.lock().await;

    let track_type = match track_type.as_str() {
        "Video" => TrackType::Video,
        "Audio" => TrackType::Audio,
        "Overlay" => TrackType::Overlay,
        _ => return Err(format!("Invalid track type: {}", track_type)),
    };

    service.add_track(track_type)
        .map_err(|e| e.to_string())
}

/// Remove a track from the timeline
#[tauri::command]
pub async fn remove_track(
    track_id: String,
    state: State<'_, TimelineServiceState>,
) -> Result<(), String> {
    let mut service = state.service.lock().await;
    service.remove_track(&track_id)
        .map_err(|e| e.to_string())
}

/// Add a clip to a track
#[tauri::command]
pub async fn add_clip_to_timeline(
    track_id: String,
    clip: Clip,
    state: State<'_, TimelineServiceState>,
) -> Result<(), String> {
    let mut service = state.service.lock().await;
    service.add_clip(&track_id, clip)
        .map_err(|e| e.to_string())
}

/// Remove a clip from the timeline
#[tauri::command]
pub async fn remove_clip_from_timeline(
    clip_id: String,
    state: State<'_, TimelineServiceState>,
) -> Result<(), String> {
    let mut service = state.service.lock().await;
    service.remove_clip(&clip_id)
        .map_err(|e| e.to_string())
}

/// Move a clip to a new position or track
#[tauri::command]
pub async fn move_clip_on_timeline(
    clip_id: String,
    new_track_id: String,
    new_position: f64,
    state: State<'_, TimelineServiceState>,
) -> Result<(), String> {
    let mut service = state.service.lock().await;
    service.move_clip(&clip_id, &new_track_id, new_position)
        .map_err(|e| e.to_string())
}

/// Trim a clip (adjust in/out points)
#[tauri::command]
pub async fn trim_clip_on_timeline(
    clip_id: String,
    trim_start: Option<f64>,
    trim_end: Option<f64>,
    state: State<'_, TimelineServiceState>,
) -> Result<(), String> {
    let mut service = state.service.lock().await;
    service.trim_clip(&clip_id, trim_start, trim_end)
        .map_err(|e| e.to_string())
}

/// Split a clip at a given time
#[tauri::command]
pub async fn split_clip_at_time(
    clip_id: String,
    split_time: f64,
    state: State<'_, TimelineServiceState>,
) -> Result<(String, String), String> {
    let mut service = state.service.lock().await;
    service.split_clip(&clip_id, split_time)
        .map_err(|e| e.to_string())
}

/// Get clips at a specific time (playhead position)
#[tauri::command]
pub async fn get_clips_at_playhead(
    time: f64,
    state: State<'_, TimelineServiceState>,
) -> Result<Vec<Clip>, String> {
    let service = state.service.lock().await;
    service.get_clips_at_time(time)
        .map_err(|e| e.to_string())
}

/// Save timeline to project file
#[tauri::command]
pub async fn save_timeline_project(
    path: String,
    state: State<'_, TimelineServiceState>,
) -> Result<(), String> {
    let mut service = state.service.lock().await;
    let path = PathBuf::from(path);

    service.save_project(path)
        .map_err(|e| e.to_string())
}

/// Load timeline from project file
#[tauri::command]
pub async fn load_timeline_project(
    path: String,
    state: State<'_, TimelineServiceState>,
) -> Result<Timeline, String> {
    let mut service = state.service.lock().await;
    let path = PathBuf::from(path);

    service.load_project(path)
        .map_err(|e| e.to_string())
}
