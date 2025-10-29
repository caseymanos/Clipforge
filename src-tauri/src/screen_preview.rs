use std::path::{Path, PathBuf};
use std::process::Command;
use crate::models::ThumbnailError;

/// Service for generating screen preview thumbnails
pub struct ScreenPreviewGenerator {
    cache_dir: PathBuf,
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

        log::info!("Screen preview cache directory: {:?}", cache_dir);

        Ok(Self { cache_dir })
    }

    /// Generate a screen preview thumbnail for a given device ID
    ///
    /// Uses FFmpeg to capture a single frame from the screen device
    #[cfg(target_os = "macos")]
    pub async fn capture_screen_preview(
        &self,
        device_id: &str,
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

        log::debug!("Capturing screen preview for device {}", device_id);

        // FFmpeg command to capture single frame from AVFoundation device
        // -f avfoundation: Use macOS screen capture
        // -i "{device_id}:none": Video device ID with no audio
        // -vframes 1: Capture only 1 frame
        // -vf "scale=320:-1": Scale to 320px width, preserve aspect ratio
        // -q:v 2: High quality JPEG (1-31 scale, lower is better)
        // -y: Overwrite if exists
        let input_device = format!("{}:none", device_id);

        let status = Command::new("ffmpeg")
            .args([
                "-f", "avfoundation",
                "-i", &input_device,
                "-vframes", "1",
                "-vf", "scale=320:-1",  // Width 320px, height auto
                "-q:v", "2",             // High quality JPEG
                "-y",                    // Overwrite if exists
                output_path_str,
            ])
            .output()
            .map_err(ThumbnailError::IoError)?;

        if !status.status.success() {
            let stderr = String::from_utf8_lossy(&status.stderr);
            log::error!("FFmpeg screen preview failed for device {}: {}", device_id, stderr);
            return Err(ThumbnailError::GenerationFailed);
        }

        log::info!("Screen preview generated: {:?}", output_path);
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
