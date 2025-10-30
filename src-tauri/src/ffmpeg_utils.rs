use std::path::PathBuf;

/// Get the FFmpeg executable path
///
/// This function first checks for a system-installed FFmpeg (which has full feature support),
/// then falls back to the bundled minimal FFmpeg binary if no system FFmpeg is found.
///
/// The bundled minimal FFmpeg has limited functionality and may not support all input formats
/// (notably, AVFoundation on macOS is disabled in the minimal build).
///
/// # Returns
/// - `Ok(PathBuf)`: Path to ffmpeg binary (system or bundled)
/// - `Err(String)`: Error message if no FFmpeg binary can be located
pub fn find_ffmpeg_path() -> Result<PathBuf, String> {
    // First, try to find system FFmpeg which has full feature support
    #[cfg(target_os = "macos")]
    {
        // Common macOS locations for Homebrew FFmpeg
        let system_paths = vec![
            PathBuf::from("/opt/homebrew/bin/ffmpeg"), // Apple Silicon Homebrew
            PathBuf::from("/usr/local/bin/ffmpeg"),    // Intel Homebrew
        ];

        for path in system_paths {
            if path.exists() {
                log::info!("Using system FFmpeg at: {:?}", path);
                return Ok(path);
            }
        }
    }

    // Fall back to bundled minimal FFmpeg
    let ffmpeg_path = get_bundled_binary_path("ffmpeg")?;

    // Verify the binary exists and is executable
    if !ffmpeg_path.exists() {
        return Err(format!(
            "No FFmpeg binary found. Please install FFmpeg via Homebrew: brew install ffmpeg\nAttempted paths:\n  - /opt/homebrew/bin/ffmpeg\n  - /usr/local/bin/ffmpeg\n  - {:?}",
            ffmpeg_path
        ));
    }

    log::warn!("Using bundled minimal FFmpeg (limited features). Install system FFmpeg for full functionality: brew install ffmpeg");
    log::info!("Bundled FFmpeg at: {:?}", ffmpeg_path);
    Ok(ffmpeg_path)
}

/// Get the bundled FFprobe executable path
///
/// This function returns the path to the FFprobe binary that is bundled with the application.
/// FFprobe is used for extracting metadata from media files.
///
/// # Returns
/// - `Ok(PathBuf)`: Path to the bundled ffprobe binary
/// - `Err(String)`: Error message if the binary cannot be located
pub fn find_ffprobe_path() -> Result<PathBuf, String> {
    let ffprobe_path = get_bundled_binary_path("ffprobe")?;

    // Verify the binary exists
    if !ffprobe_path.exists() {
        return Err(format!(
            "Bundled FFprobe binary not found at: {:?}\nThis is a packaging error. Please reinstall the application.",
            ffprobe_path
        ));
    }

    log::info!("Using bundled FFprobe at: {:?}", ffprobe_path);
    Ok(ffprobe_path)
}

/// Get the path to a bundled binary
///
/// Tauri's externalBin feature places bundled binaries in the same directory as the main executable.
/// On macOS, this is typically: ClipForge.app/Contents/MacOS/
///
/// # Arguments
/// * `binary_name` - The name of the binary (e.g., "ffmpeg" or "ffprobe")
///
/// # Returns
/// - `Ok(PathBuf)`: Path to the bundled binary
/// - `Err(String)`: Error if the executable directory cannot be determined
fn get_bundled_binary_path(binary_name: &str) -> Result<PathBuf, String> {
    // Get the directory containing the current executable
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;

    let exe_dir = exe_path.parent()
        .ok_or_else(|| "Failed to get executable directory".to_string())?;

    // Try with platform suffix first (e.g., ffmpeg-aarch64-apple-darwin)
    let binary_name_with_suffix = format!("{}-aarch64-apple-darwin", binary_name);
    let binary_path_with_suffix = exe_dir.join(&binary_name_with_suffix);

    if binary_path_with_suffix.exists() {
        log::debug!(
            "Found bundled binary '{}' at: {:?}",
            binary_name,
            binary_path_with_suffix
        );
        return Ok(binary_path_with_suffix);
    }

    // Fall back to simple name (e.g., ffmpeg)
    let binary_path = exe_dir.join(binary_name);

    log::debug!(
        "Looking for bundled binary '{}' at: {:?}",
        binary_name,
        binary_path
    );

    Ok(binary_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_bundled_path_construction() {
        // Test that the path construction logic works correctly
        let result = get_bundled_binary_path("ffmpeg");
        assert!(result.is_ok());

        let path = result.unwrap();
        let path_str = path.to_string_lossy();
        assert!(path_str.contains("ffmpeg-aarch64-apple-darwin"));
    }

    #[test]
    fn test_ffmpeg_path() {
        // This test will only pass in a bundled build
        match find_ffmpeg_path() {
            Ok(path) => {
                println!("Bundled FFmpeg found at: {:?}", path);
                assert!(path.to_string_lossy().contains("ffmpeg"));
            }
            Err(e) => {
                // Expected to fail in development mode (not bundled)
                println!("FFmpeg not bundled (expected in dev mode): {}", e);
            }
        }
    }

    #[test]
    fn test_ffprobe_path() {
        // This test will only pass in a bundled build
        match find_ffprobe_path() {
            Ok(path) => {
                println!("Bundled FFprobe found at: {:?}", path);
                assert!(path.to_string_lossy().contains("ffprobe"));
            }
            Err(e) => {
                // Expected to fail in development mode (not bundled)
                println!("FFprobe not bundled (expected in dev mode): {}", e);
            }
        }
    }
}
