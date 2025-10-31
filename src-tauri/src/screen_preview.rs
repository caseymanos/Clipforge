use std::path::{Path, PathBuf};
use std::process::Command;
use crate::models::ThumbnailError;
use crate::ffmpeg_utils;

/// Service for generating screen preview thumbnails
pub struct ScreenPreviewGenerator {
    cache_dir: PathBuf,
    ffmpeg_path: PathBuf,
}

impl ScreenPreviewGenerator {
    /// Create a new screen preview generator
    pub fn new() -> Result<Self, ThumbnailError> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| {
                ThumbnailError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Cache directory not found"
                ))
            })?
            .join("clipforge")
            .join("screen_previews");

        std::fs::create_dir_all(&cache_dir)?;

        // Find FFmpeg path
        let ffmpeg_path = ffmpeg_utils::find_ffmpeg_path()
            .map_err(|e| ThumbnailError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("FFmpeg not found: {}", e)
            )))?;

        log::info!("Screen preview cache directory: {:?}", cache_dir);
        log::info!("ScreenPreviewGenerator using FFmpeg at: {:?}", ffmpeg_path);

        Ok(Self { cache_dir, ffmpeg_path })
    }

    /// Generate a screen preview thumbnail for a given device ID
    ///
    /// Uses FFmpeg to capture a single frame from the screen device
    /// device_type should be one of: "screen", "window", or "webcam"
    #[cfg(target_os = "macos")]
    pub async fn capture_screen_preview(
        &self,
        device_id: &str,
        device_type: &str,
    ) -> Result<PathBuf, ThumbnailError> {
        let preview_filename = format!("screen-{}.jpg", device_id);
        let output_path = self.cache_dir.join(&preview_filename);

        let output_path_str = output_path.to_str()
            .ok_or_else(|| ThumbnailError::IoError(
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Output path contains invalid UTF-8 characters"
                )
            ))?;

        log::debug!("Capturing screen preview for device {} (type: {})", device_id, device_type);

        // Determine if this is a webcam device based on passed device_type
        let is_webcam = device_type == "webcam";

        // Format device name correctly for AVFoundation
        // Webcams use numeric IDs directly, screens/windows need "Capture screen {index}" format
        let input_device = if is_webcam {
            format!("{}:none", device_id)
        } else {
            // For screens/windows, use the device ID directly (it's already the screen index)
            format!("Capture screen {}:none", device_id)
        };

        // Build FFmpeg command - webcams need different parameters than screens
        // Try multiple framerates for webcams since different models support different rates
        let framerates = if is_webcam {
            vec![60, 30, 15]  // Common webcam framerates
        } else {
            vec![30]  // Screens typically support 30fps
        };

        let mut last_error = String::new();
        let mut success = false;

        for framerate in framerates {
            log::debug!("Trying framerate {}fps for device {}", framerate, device_id);
            let mut cmd = Command::new(&self.ffmpeg_path);
            cmd.arg("-f").arg("avfoundation");

            if is_webcam {
                // For cameras: specify explicit framerate and capture time
                cmd.arg("-framerate").arg(framerate.to_string());
                cmd.arg("-t").arg("1");  // Capture for 1 second
            } else {
                // For screens: use short capture timeout
                cmd.arg("-t").arg("0.1");
            }

            cmd.arg("-i").arg(&input_device);
            cmd.arg("-vframes").arg("1");
            cmd.arg("-vf").arg("scale=320:-1");  // Width 320px, height auto
            cmd.arg("-q:v").arg("2");             // High quality JPEG
            cmd.arg("-y");                        // Overwrite if exists
            cmd.arg(output_path_str);

            let status = cmd.output().map_err(ThumbnailError::IoError)?;

            if status.status.success() {
                success = true;
                log::info!("Screen preview generated at {}fps: {:?}", framerate, output_path);
                break;
            } else {
                let stderr = String::from_utf8_lossy(&status.stderr);
                last_error = stderr.to_string();
                log::warn!("Preview capture failed at {}fps for device {} (type: {}). FFmpeg error: {}",
                    framerate, device_id, device_type,
                    last_error.lines().take(3).collect::<Vec<&str>>().join(" | "));
            }
        }

        if !success {
            log::error!("FFmpeg screen preview failed for device {} (type: {}) after trying all framerates. Input device: {}. Full error: {}",
                device_id, device_type, input_device, last_error);
            return Err(ThumbnailError::GenerationFailed);
        }

        Ok(output_path)
    }

    /// Generate a window preview thumbnail (placeholder for future implementation)
    #[cfg(target_os = "macos")]
    pub async fn capture_window_preview(
        &self,
        window_id: &str,
    ) -> Result<PathBuf, ThumbnailError> {
        let preview_filename = format!("window-{}.jpg", window_id);
        let output_path = self.cache_dir.join(&preview_filename);

        log::warn!("Window preview capture not yet implemented for window {}", window_id);

        // TODO: Implement window-specific capture
        // For now, return an error to gracefully fallback
        Err(ThumbnailError::GenerationFailed)
    }

    /// Get the cache directory path
    #[allow(dead_code)]
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Clear all cached screen previews
    #[allow(dead_code)]
    pub fn clear_cache(&self) -> Result<(), ThumbnailError> {
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)?;
            std::fs::create_dir_all(&self.cache_dir)?;
            log::info!("Screen preview cache cleared");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_preview_generator_creation() {
        let generator = ScreenPreviewGenerator::new();
        assert!(generator.is_ok());
    }
}
