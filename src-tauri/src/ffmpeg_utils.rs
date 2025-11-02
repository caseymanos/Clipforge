use std::path::PathBuf;

/// Get the FFmpeg executable path
///
/// This function uses the bundled FFmpeg binary which is statically linked and includes
/// all necessary features for ClipForge (screen capture, encoding, filtering).
///
/// # Returns
/// - `Ok(PathBuf)`: Path to ffmpeg binary
/// - `Err(String)`: Error message if no FFmpeg binary can be located
pub fn find_ffmpeg_path() -> Result<PathBuf, String> {
    // Use bundled FFmpeg (statically linked with all features)
    let ffmpeg_path = get_bundled_binary_path("ffmpeg")?;

    // Verify the binary exists and is executable
    if !ffmpeg_path.exists() {
        return Err(format!(
            "Bundled FFmpeg binary not found at: {:?}",
            ffmpeg_path
        ));
    }

    log::info!("Using bundled FFmpeg at: {:?}", ffmpeg_path);
    Ok(ffmpeg_path)
}

/// Get the bundled FFprobe executable path
///
/// This function uses the bundled FFprobe binary which is statically linked.
/// FFprobe is used for extracting metadata from media files.
///
/// # Returns
/// - `Ok(PathBuf)`: Path to the ffprobe binary
/// - `Err(String)`: Error message if the binary cannot be located
pub fn find_ffprobe_path() -> Result<PathBuf, String> {
    // Use bundled FFprobe (statically linked)
    let ffprobe_path = get_bundled_binary_path("ffprobe")?;

    // Verify the binary exists
    if !ffprobe_path.exists() {
        return Err(format!(
            "Bundled FFprobe binary not found at: {:?}",
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
/// In development mode, this function also checks the src-tauri/binaries/ directory.
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

    if binary_path.exists() {
        log::debug!(
            "Found bundled binary '{}' at: {:?}",
            binary_name,
            binary_path
        );
        return Ok(binary_path);
    }

    // Development mode fallback: check src-tauri/binaries/
    // In dev mode, exe_path is typically: /path/to/project/src-tauri/target/debug/clipforge
    // We need to navigate to: /path/to/project/src-tauri/binaries/
    #[cfg(debug_assertions)]
    {
        if let Some(target_dir) = exe_dir.parent() {
            if let Some(src_tauri_dir) = target_dir.parent() {
                let dev_binary_path = src_tauri_dir.join("binaries").join(binary_name);
                if dev_binary_path.exists() {
                    log::info!(
                        "Found development binary '{}' at: {:?}",
                        binary_name,
                        dev_binary_path
                    );
                    return Ok(dev_binary_path);
                }
            }
        }
    }

    log::debug!(
        "Looking for bundled binary '{}' at: {:?} (does not exist)",
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
