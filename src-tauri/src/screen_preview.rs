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

        // Determine if this is a camera device (numeric ID) or screen device (starts with "Capture screen")
        let is_camera = device_id.parse::<u32>().is_ok();

        // Format device name correctly for AVFoundation
        // Cameras use numeric IDs, screens need "Capture screen {id}" format
        let input_device = if is_camera {
            format!("{}:none", device_id)
        } else {
            format!("Capture screen {}:none", device_id)
        };

        // Build FFmpeg command - cameras need different parameters than screens
        let mut cmd = Command::new(&self.ffmpeg_path);
        cmd.arg("-f").arg("avfoundation");

        if is_camera {
            // For cameras: specify explicit framerate that most cameras support
            cmd.arg("-framerate").arg("30");
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
