// Integration helpers for recording (e.g., auto-import to media library)

use crate::file_service::FileService;
use std::path::Path;
use log::{info, error};

/// Auto-import a recorded video file to the media library
///
/// This is called after a recording is completed to automatically
/// add the recorded file to the user's media library.
pub async fn auto_import_recording(
    file_service: &FileService,
    recording_path: &Path,
) -> Result<String, String> {
    info!("Auto-importing recording: {}", recording_path.display());

    match file_service.import_file(recording_path.to_path_buf()).await {
        Ok(media_file) => {
            info!("Successfully imported recording with ID: {}", media_file.id);
            Ok(media_file.id)
        }
        Err(e) => {
            error!("Failed to auto-import recording: {}", e);
            Err(format!("Failed to import recording: {}", e))
        }
    }
}
