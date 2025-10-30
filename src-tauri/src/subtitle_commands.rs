use crate::models::{MediaFile, SubtitleSegment, SubtitleTrack, Timeline};
use crate::subtitle::SubtitleService;
use log::{info, error};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{State, Window};
use tokio::sync::Mutex;

/// Shared subtitle service state
pub struct SubtitleServiceState {
    pub service: Arc<Mutex<Option<SubtitleService>>>,
}

/// Set OpenAI API key
#[tauri::command]
pub async fn set_openai_api_key(
    api_key: String,
    state: State<'_, SubtitleServiceState>,
) -> Result<(), String> {
    info!("Setting OpenAI API key");

    let service = SubtitleService::new(api_key)
        .map_err(|e| format!("Failed to initialize subtitle service: {}", e))?;

    let mut service_lock = state.service.lock().await;
    *service_lock = Some(service);

    Ok(())
}

/// Check if subtitle service is available
#[tauri::command]
pub async fn check_subtitle_available(
    state: State<'_, SubtitleServiceState>,
) -> Result<bool, String> {
    let service_lock = state.service.lock().await;
    Ok(service_lock.is_some())
}

/// Transcribe timeline audio to generate subtitles
#[tauri::command]
pub async fn transcribe_timeline_audio(
    timeline_id: String,
    media_files: Vec<MediaFile>,
    language: Option<String>,
    window: Window,
    state: State<'_, SubtitleServiceState>,
) -> Result<SubtitleTrack, String> {
    info!("Transcribing timeline audio: {}", timeline_id);

    let service_lock = state.service.lock().await;
    let service = service_lock.as_ref()
        .ok_or_else(|| "Subtitle service not initialized. Please set API key first.".to_string())?;

    // For now, transcribe the first media file with audio
    // TODO: In future, merge all audio tracks
    let media_file = media_files.iter()
        .find(|f| f.codec.audio.is_some())
        .ok_or_else(|| "No audio track found in timeline".to_string())?;

    let track = service.transcribe_media_file(media_file, language, Some(window))
        .await
        .map_err(|e| format!("Transcription failed: {}", e))?;

    Ok(track)
}

/// Update a subtitle segment
#[tauri::command]
pub async fn update_subtitle_segment(
    _timeline_id: String,
    _segment_id: usize,
    _new_text: String,
    _new_start: Option<f64>,
    _new_end: Option<f64>,
) -> Result<(), String> {
    // Note: This command would need access to timeline service to persist changes
    // For now, we'll rely on the frontend to update the timeline and call save_timeline
    // This is consistent with the optimistic update pattern

    Ok(())
}

/// Toggle subtitles on/off for timeline
#[tauri::command]
pub async fn toggle_subtitles(
    _timeline_id: String,
    _enabled: bool,
) -> Result<(), String> {
    // Note: This would be handled by timeline service
    // Frontend will update timeline.subtitle_enabled and persist

    Ok(())
}

/// Export subtitles to SRT file
#[tauri::command]
pub async fn export_subtitles_srt(
    track: SubtitleTrack,
    output_path: String,
) -> Result<(), String> {
    info!("Exporting subtitles to SRT: {}", output_path);

    let path = PathBuf::from(output_path);
    SubtitleService::export_srt(&track, &path)
        .map_err(|e| format!("Failed to export SRT: {}", e))?;

    Ok(())
}

/// Import subtitles from SRT file
#[tauri::command]
pub async fn import_subtitles_srt(
    file_path: String,
    language: Option<String>,
) -> Result<SubtitleTrack, String> {
    info!("Importing subtitles from SRT: {}", file_path);

    let path = PathBuf::from(&file_path);
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read SRT file: {}", e))?;

    let segments = SubtitleService::parse_srt(&content)
        .map_err(|e| format!("Failed to parse SRT: {}", e))?;

    Ok(SubtitleTrack {
        segments,
        language: language.unwrap_or_else(|| "en".to_string()),
        source: crate::models::SubtitleSource::Imported {
            file_path: path,
        },
    })
}
