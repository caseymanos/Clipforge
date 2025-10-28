use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;
use crate::models::ThumbnailError;

/// Service for generating video thumbnails
pub struct ThumbnailGenerator {
    cache_dir: PathBuf,
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

        log::info!("Thumbnail cache directory: {:?}", cache_dir);

        Ok(Self { cache_dir })
    }

    /// Generate a single thumbnail at the specified timestamp
    pub async fn generate(
        &self,
        video_path: &Path,
        timestamp: f64
    ) -> Result<PathBuf, ThumbnailError> {
        let thumb_filename = format!("{}.jpg", Uuid::new_v4());
        let output_path = self.cache_dir.join(&thumb_filename);

        log::debug!("Generating thumbnail for {:?} at {}s", video_path, timestamp);

        let status = Command::new("ffmpeg")
            .args(&[
                "-ss", &timestamp.to_string(),
                "-i", video_path.to_str().unwrap(),
                "-vframes", "1",
                "-vf", "scale=320:-1",  // Width 320px, height auto
                "-q:v", "2",             // High quality JPEG
                "-y",                    // Overwrite if exists
                output_path.to_str().unwrap(),
            ])
            .output()
            .map_err(|e| ThumbnailError::IoError(e))?;

        if !status.status.success() {
            log::error!("FFmpeg thumbnail generation failed for {:?}", video_path);
            return Err(ThumbnailError::GenerationFailed);
        }

        log::info!("Thumbnail generated: {:?}", output_path);
        Ok(output_path)
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
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Clear all cached thumbnails
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
