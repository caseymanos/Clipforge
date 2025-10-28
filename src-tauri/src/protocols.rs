use tauri::App;
use std::path::Path;

/// Register custom stream:// protocol for efficient video file access
pub fn register_stream_protocol(_app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // Note: In Tauri v2, the asset protocol is configured via tauri.conf.json
    // The 'asset' protocol is available by default for local file access
    // Frontend can use convertFileSrc() from @tauri-apps/api/core to convert paths

    // Protocol scope is configured in tauri.conf.json under security settings
    // We don't need to programmatically set it here

    log::info!("Protocol registration skipped - using default asset protocol");

    Ok(())
}

/// Check if a path is within allowed directories
///
/// Canonicalizes paths to prevent traversal attacks (e.g., ../)
#[allow(dead_code)]
fn is_path_allowed(path: &str) -> bool {
    // Canonicalize to resolve .. and symlinks, preventing traversal attacks
    let canonical_path = match Path::new(path).canonicalize() {
        Ok(p) => p,
        Err(_) => {
            log::warn!("Failed to canonicalize path: {}", path);
            return false;
        }
    };

    // Only allow paths in user's home or app data directory
    if let Some(home_dir) = dirs::home_dir() {
        if let Ok(canonical_home) = home_dir.canonicalize() {
            if canonical_path.starts_with(canonical_home) {
                return true;
            }
        }
    }

    if let Some(data_dir) = dirs::data_local_dir() {
        if let Ok(canonical_data) = data_dir.canonicalize() {
            if canonical_path.starts_with(canonical_data) {
                return true;
            }
        }
    }

    log::warn!("Path not allowed: {:?}", canonical_path);
    false
}

/// Get MIME type from file extension
#[allow(dead_code)]
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
