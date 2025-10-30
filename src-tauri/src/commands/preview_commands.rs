// Module 8: Video Preview Commands

use crate::models::Timeline;
use crate::preview_service::PreviewService;
use tauri::State;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Render a preview frame from the timeline at the specified time
#[tauri::command]
pub async fn render_preview_frame(
    service: State<'_, Arc<Mutex<PreviewService>>>,
    timeline: Timeline,
    time: f64,
    media_files: HashMap<String, PathBuf>,
) -> Result<String, String> {
    let service = service.lock().await;

    service
        .render_preview_frame(&timeline, time, &media_files)
        .await
        .map_err(|e| format!("Failed to render preview frame: {}", e))
}

/// Clear the preview frame cache
#[tauri::command]
pub async fn clear_preview_cache(
    service: State<'_, Arc<Mutex<PreviewService>>>,
) -> Result<(), String> {
    let service = service.lock().await;
    service.clear_cache().await;
    Ok(())
}

/// Get cache statistics
#[tauri::command]
pub async fn get_cache_stats(
    service: State<'_, Arc<Mutex<PreviewService>>>,
) -> Result<CacheStatsResponse, String> {
    let service = service.lock().await;
    let stats = service.cache_stats().await;

    Ok(CacheStatsResponse {
        capacity: stats.capacity,
        current_size: stats.current_size,
        hit_rate: stats.hit_rate,
    })
}

#[derive(serde::Serialize)]
pub struct CacheStatsResponse {
    pub capacity: usize,
    pub current_size: usize,
    pub hit_rate: f64,
}
