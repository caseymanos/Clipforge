use std::path::PathBuf;
use tauri::State;
use crate::file_service::FileService;
use crate::metadata::extract_metadata;
use crate::thumbnail::ThumbnailGenerator;
use crate::models::{MediaFile, FileMetadata};

/// Import a media file into the library
#[tauri::command]
pub async fn import_media_file(
    path: String,
    file_service: State<'_, FileService>
) -> Result<MediaFile, String> {
    log::info!("Command: import_media_file({})", path);
    file_service
        .import_file(PathBuf::from(path))
        .await
        .map_err(|e| {
            log::error!("Import failed: {}", e);
            e.to_string()
        })
}

/// Get all media files in the library
#[tauri::command]
pub async fn get_media_library(
    file_service: State<'_, FileService>
) -> Result<Vec<MediaFile>, String> {
    log::info!("Command: get_media_library()");
    file_service
        .get_all_media()
        .await
        .map_err(|e| {
            log::error!("Failed to get media library: {}", e);
            e.to_string()
        })
}

/// Get a specific media file by ID
#[tauri::command]
pub async fn get_media_file(
    id: String,
    file_service: State<'_, FileService>
) -> Result<Option<MediaFile>, String> {
    log::info!("Command: get_media_file({})", id);
    file_service
        .get_by_id(&id)
        .await
        .map_err(|e| {
            log::error!("Failed to get media file: {}", e);
            e.to_string()
        })
}

/// Delete a media file from the library
#[tauri::command]
pub async fn delete_media_file(
    id: String,
    file_service: State<'_, FileService>
) -> Result<(), String> {
    log::info!("Command: delete_media_file({})", id);
    file_service
        .delete_media(id)
        .await
        .map_err(|e| {
            log::error!("Delete failed: {}", e);
            e.to_string()
        })
}

/// Extract metadata from a video file
#[tauri::command]
pub async fn get_file_metadata(
    path: String
) -> Result<FileMetadata, String> {
    log::info!("Command: get_file_metadata({})", path);
    extract_metadata(&PathBuf::from(path))
        .await
        .map_err(|e| {
            log::error!("Metadata extraction failed: {}", e);
            e.to_string()
        })
}

/// Generate a thumbnail for a video file at a specific timestamp
#[tauri::command]
pub async fn generate_thumbnail(
    video_path: String,
    timestamp: f64,
    file_service: State<'_, FileService>
) -> Result<String, String> {
    log::info!("Command: generate_thumbnail({}, {}s)", video_path, timestamp);
    let path = file_service
        .thumbnail_generator()
        .generate(&PathBuf::from(video_path), timestamp)
        .await
        .map_err(|e| {
            log::error!("Thumbnail generation failed: {}", e);
            e.to_string()
        })?;

    Ok(path.to_string_lossy().to_string())
}

/// Generate a sequence of thumbnails for timeline preview
#[tauri::command]
pub async fn generate_thumbnail_sequence(
    video_path: String,
    duration: f64,
    count: usize,
    file_service: State<'_, FileService>
) -> Result<Vec<String>, String> {
    log::info!("Command: generate_thumbnail_sequence({}, duration={:.2}s, count={})",
              video_path, duration, count);
    let paths = file_service
        .thumbnail_generator()
        .generate_sequence(&PathBuf::from(video_path), duration, count)
        .await
        .map_err(|e| {
            log::error!("Thumbnail sequence generation failed: {}", e);
            e.to_string()
        })?;

    Ok(paths.iter().map(|p| p.to_string_lossy().to_string()).collect())
}
