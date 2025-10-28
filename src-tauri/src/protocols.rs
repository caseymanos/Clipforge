use tauri::{App, AppHandle};
use std::fs;
use std::path::Path;

/// Register custom stream:// protocol for efficient video file access
pub fn register_stream_protocol(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.app_handle();

    app_handle.asset_protocol_scope().allow_directory(
        dirs::home_dir().unwrap_or_default(),
        true,
    )?;

    // Register the stream protocol
    app.handle().register_asynchronous_uri_scheme_protocol("stream", move |_app, request, responder| {
        let path = request.uri().path();

        // Security: Validate path is within allowed directories
        if !is_path_allowed(path) {
            log::warn!("Access denied to path: {}", path);
            responder.respond(
                tauri::http::Response::builder()
                    .status(403)
                    .body(vec![])
                    .unwrap()
            );
            return;
        }

        // Read video file
        match fs::read(path) {
            Ok(data) => {
                // Determine MIME type from extension
                let mime_type = get_mime_type(path);

                responder.respond(
                    tauri::http::Response::builder()
                        .status(200)
                        .header("Content-Type", mime_type)
                        .header("Accept-Ranges", "bytes")
                        .body(data)
                        .unwrap()
                );
            }
            Err(e) => {
                log::error!("Failed to read file {}: {}", path, e);
                responder.respond(
                    tauri::http::Response::builder()
                        .status(404)
                        .body(vec![])
                        .unwrap()
                );
            }
        }
    });

    Ok(())
}

/// Check if a path is within allowed directories
fn is_path_allowed(path: &str) -> bool {
    let path = Path::new(path);

    // Only allow paths in user's home or app data directory
    if let Some(home_dir) = dirs::home_dir() {
        if path.starts_with(home_dir) {
            return true;
        }
    }

    if let Some(data_dir) = dirs::data_local_dir() {
        if path.starts_with(data_dir) {
            return true;
        }
    }

    false
}

/// Get MIME type from file extension
fn get_mime_type(path: &str) -> &'static str {
    match Path::new(path).extension().and_then(|e| e.to_str()) {
        Some("mp4") => "video/mp4",
        Some("mov") => "video/quicktime",
        Some("webm") => "video/webm",
        Some("avi") => "video/x-msvideo",
        Some("mkv") => "video/x-matroska",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mime_type_detection() {
        assert_eq!(get_mime_type("video.mp4"), "video/mp4");
        assert_eq!(get_mime_type("clip.mov"), "video/quicktime");
        assert_eq!(get_mime_type("test.webm"), "video/webm");
        assert_eq!(get_mime_type("unknown.xyz"), "application/octet-stream");
    }

    #[test]
    fn test_path_validation() {
        // Test path security checks
        assert!(!is_path_allowed("/etc/passwd"));
        assert!(!is_path_allowed("/System/Library/"));
    }
}
