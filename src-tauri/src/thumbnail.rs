use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;
use crate::models::ThumbnailError;
use crate::ffmpeg_utils;
use std::time::Duration;

/// Service for generating video thumbnails
pub struct ThumbnailGenerator {
    cache_dir: PathBuf,
    ffmpeg_path: PathBuf,
}

impl ThumbnailGenerator {
    /// Create a new thumbnail generator
    pub fn new() -> Result<Self, ThumbnailError> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| {
                ThumbnailError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Cache directory not found"
                ))
            })?
            .join("clipforge")
            .join("thumbnails");

        std::fs::create_dir_all(&cache_dir)?;

        // Find FFmpeg path
        let ffmpeg_path = ffmpeg_utils::find_ffmpeg_path()
            .map_err(|e| ThumbnailError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("FFmpeg not found: {}", e)
            )))?;

        log::info!("Thumbnail cache directory: {:?}", cache_dir);
        log::info!("ThumbnailGenerator using FFmpeg at: {:?}", ffmpeg_path);

        Ok(Self { cache_dir, ffmpeg_path })
    }

    /// Generate a single thumbnail at the specified timestamp
    pub async fn generate(
        &self,
        video_path: &Path,
        timestamp: f64
    ) -> Result<PathBuf, ThumbnailError> {
        let video_path_str = video_path.to_str()
            .ok_or_else(|| ThumbnailError::IoError(
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Video path contains invalid UTF-8 characters"
                )
            ))?;

        let thumb_filename = format!("{}.jpg", Uuid::new_v4());
        let output_path = self.cache_dir.join(&thumb_filename);

        let output_path_str = output_path.to_str()
            .ok_or_else(|| ThumbnailError::IoError(
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Output path contains invalid UTF-8 characters"
                )
            ))?;

        log::debug!("Generating thumbnail for {:?} at {}s", video_path, timestamp);

        // Verify file is readable before attempting
        if !video_path.exists() {
            log::error!("Video file does not exist: {:?}", video_path);
            return Err(ThumbnailError::IoError(
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Video file not found: {:?}", video_path)
                )
            ));
        }

        // Retry logic with exponential backoff: 3 attempts with 500ms, 1s, 2s delays
        const MAX_ATTEMPTS: u32 = 3;
        const RETRY_DELAYS_MS: [u64; 2] = [500, 1000]; // Delays after 1st and 2nd failures

        let mut last_error = None;

        for attempt in 0..MAX_ATTEMPTS {
            if attempt > 0 {
                let delay_ms = RETRY_DELAYS_MS[(attempt - 1) as usize];
                log::warn!("Thumbnail generation attempt {} failed, retrying in {}ms...", attempt, delay_ms);
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            }

            let status = Command::new(&self.ffmpeg_path)
                .args([
                    "-ss", &timestamp.to_string(),
                    "-i", video_path_str,
                    "-vframes", "1",
                    "-vf", "scale=320:-1",  // Width 320px, height auto
                    "-q:v", "2",             // High quality JPEG
                    "-y",                    // Overwrite if exists
                    output_path_str,
                ])
                .output()
                .map_err(ThumbnailError::IoError)?;

            if status.status.success() {
                log::info!("Thumbnail generated successfully: {:?}", output_path);
                return Ok(output_path);
            }

            last_error = Some(format!(
                "FFmpeg stderr: {}",
                String::from_utf8_lossy(&status.stderr)
            ));
            log::error!("FFmpeg thumbnail generation failed for {:?} (attempt {}): {:?}",
                video_path, attempt + 1, last_error);
        }

        log::error!("All {} thumbnail generation attempts failed for {:?}", MAX_ATTEMPTS, video_path);
        Err(ThumbnailError::GenerationFailed)
    }

    /// Generate a sequence of thumbnails across the video duration
    pub async fn generate_sequence(
        &self,
        video_path: &Path,
        duration: f64,
        count: usize
    ) -> Result<Vec<PathBuf>, ThumbnailError> {
        let mut thumbnails = Vec::new();
        let interval = duration / count as f64;

        for i in 0..count {
            let timestamp = i as f64 * interval;
            let thumb = self.generate(video_path, timestamp).await?;
            thumbnails.push(thumb);
        }

        Ok(thumbnails)
    }

    /// Get the cache directory path
    #[allow(dead_code)]
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Clear all cached thumbnails
    #[allow(dead_code)]
    pub fn clear_cache(&self) -> Result<(), ThumbnailError> {
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)?;
            std::fs::create_dir_all(&self.cache_dir)?;
            log::info!("Thumbnail cache cleared");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thumbnail_generator_creation() {
        let generator = ThumbnailGenerator::new();
        assert!(generator.is_ok());
    }
}
