use crate::preview_cache::PreviewCache;
use crate::models::{Clip, Timeline, TrackType};
use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose, Engine as _};
use log::{debug, info, warn};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use tokio::fs;

/// Service for rendering video preview frames
pub struct PreviewService {
    /// Frame cache for fast scrubbing
    cache: Arc<PreviewCache>,
    /// Path to FFmpeg binary
    ffmpeg_path: PathBuf,
}

impl PreviewService {
    /// Create a new preview service
    pub fn new() -> Self {
        Self {
            cache: Arc::new(PreviewCache::new(500)), // Increased cache for better performance
            ffmpeg_path: PathBuf::from("ffmpeg"),
        }
    }

    /// Render a single frame from a timeline at the specified time
    ///
    /// # Arguments
    /// * `timeline` - The timeline to render
    /// * `time` - Time in seconds
    /// * `media_files` - Map of media file IDs to file paths
    ///
    /// # Returns
    /// Base64-encoded JPEG image
    pub async fn render_preview_frame(
        &self,
        timeline: &Timeline,
        time: f64,
        media_files: &HashMap<String, PathBuf>,
    ) -> Result<String> {
        // Check cache first
        if let Some(cached_frame) = self.cache.get(time).await {
            debug!("Using cached frame for time: {}s", time);
            return Ok(general_purpose::STANDARD.encode(&cached_frame));
        }

        info!(
            "Rendering preview frame for timeline '{}' at {}s",
            timeline.name, time
        );

        // Find the active clip(s) at this time
        let active_clips = self.find_active_clips(timeline, time);

        if active_clips.is_empty() {
            let frame_data = self.render_blank_frame(&timeline.resolution).await?;
            return Ok(general_purpose::STANDARD.encode(&frame_data));
        }

        // For single clip, render directly
        if active_clips.len() == 1 {
            let clip = &active_clips[0];
            let frame_data = self
                .render_single_clip_frame(clip, time, &timeline.resolution, media_files)
                .await?;

            // Cache the frame
            self.cache.put(time, frame_data.clone()).await;

            return Ok(general_purpose::STANDARD.encode(&frame_data));
        }

        // For multiple clips, composite them
        let frame_data = self
            .render_composite_frame(&active_clips, time, &timeline.resolution, media_files)
            .await?;

        // Cache the frame
        self.cache.put(time, frame_data.clone()).await;

        Ok(general_purpose::STANDARD.encode(&frame_data))
    }

    /// Find all clips that are active at the given time
    fn find_active_clips(&self, timeline: &Timeline, time: f64) -> Vec<Clip> {
        let mut active_clips = Vec::new();

        for track in &timeline.tracks {
            // Only process video and overlay tracks
            if !matches!(track.track_type, TrackType::Video | TrackType::Overlay) {
                continue;
            }

            if track.muted {
                continue;
            }

            for clip in &track.clips {
                let clip_start = clip.track_position;
                let clip_end = clip.track_position + clip.duration;

                if time >= clip_start && time < clip_end {
                    active_clips.push(clip.clone());
                }
            }
        }

        active_clips
    }

    /// Render a frame from a single clip
    async fn render_single_clip_frame(
        &self,
        clip: &Clip,
        timeline_time: f64,
        resolution: &crate::models::Resolution,
        media_files: &HashMap<String, PathBuf>,
    ) -> Result<Vec<u8>> {
        let media_path = media_files
            .get(&clip.media_file_id)
            .ok_or_else(|| anyhow!("Media file not found: {}", clip.media_file_id))?;

        // Calculate time within the clip
        let clip_time = (timeline_time - clip.track_position) / clip.speed as f64 + clip.trim_start;

        debug!(
            "Rendering clip '{}' at {}s (clip time: {}s)",
            clip.id, timeline_time, clip_time
        );

        self.extract_frame(media_path, clip_time, resolution).await
    }

    /// Render a composite frame from multiple clips
    async fn render_composite_frame(
        &self,
        clips: &[Clip],
        time: f64,
        resolution: &crate::models::Resolution,
        media_files: &HashMap<String, PathBuf>,
    ) -> Result<Vec<u8>> {
        info!("Compositing {} clips at {}s", clips.len(), time);

        // For MVP, just render the topmost clip
        // TODO: Implement proper layer compositing with FFmpeg filter_complex
        if let Some(clip) = clips.last() {
            self.render_single_clip_frame(clip, time, resolution, media_files)
                .await
        } else {
            self.render_blank_frame(resolution).await
        }
    }

    /// Extract a single frame from a video file
    async fn extract_frame(
        &self,
        video_path: &Path,
        time: f64,
        resolution: &crate::models::Resolution,
    ) -> Result<Vec<u8>> {
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(format!("clipforge_frame_{}.jpg", uuid::Uuid::new_v4()));

        debug!(
            "Extracting frame from {:?} at {}s to {:?}",
            video_path, time, temp_file
        );

        // Build FFmpeg command (optimized for preview performance)
        // Limit preview resolution to 1280px width for faster processing
        let preview_width = resolution.width.min(1280);
        let preview_height = (preview_width as f64 / resolution.width as f64 * resolution.height as f64) as u32;

        let output = Command::new(&self.ffmpeg_path)
            .arg("-hwaccel")
            .arg("auto") // Use hardware acceleration if available (videotoolbox on macOS, etc.)
            .arg("-ss") // Seek before input (faster)
            .arg(format!("{}", time))
            .arg("-i")
            .arg(video_path)
            .arg("-vframes")
            .arg("1") // Extract 1 frame
            .arg("-vf")
            .arg(format!("scale={}:{}", preview_width, preview_height))
            .arg("-q:v")
            .arg("5") // Balanced quality/speed (was 2 - highest quality but slower)
            .arg("-f")
            .arg("image2")
            .arg(&temp_file)
            .arg("-y") // Overwrite output
            .output()
            .context("Failed to spawn ffmpeg process")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("FFmpeg error: {}", stderr);
            return Err(anyhow!("FFmpeg failed to extract frame: {}", stderr));
        }

        // Read the generated image
        let frame_data = fs::read(&temp_file)
            .await
            .context("Failed to read extracted frame")?;

        // Clean up temp file
        let _ = fs::remove_file(&temp_file).await;

        debug!("Frame extracted successfully, size: {} bytes", frame_data.len());

        Ok(frame_data)
    }

    /// Render a blank frame (black screen) for empty timeline sections
    async fn render_blank_frame(
        &self,
        resolution: &crate::models::Resolution,
    ) -> Result<Vec<u8>> {
        debug!(
            "Rendering blank frame at {}x{}",
            resolution.width, resolution.height
        );

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(format!("clipforge_blank_{}.jpg", uuid::Uuid::new_v4()));

        // Generate black frame using FFmpeg
        let output = Command::new(&self.ffmpeg_path)
            .arg("-f")
            .arg("lavfi")
            .arg("-i")
            .arg(format!(
                "color=black:s={}x{}:d=0.1",
                resolution.width, resolution.height
            ))
            .arg("-frames:v")
            .arg("1")
            .arg("-q:v")
            .arg("2")
            .arg(&temp_file)
            .arg("-y")
            .output()
            .context("Failed to generate blank frame")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to generate blank frame: {}", stderr));
        }

        let frame_data = fs::read(&temp_file)
            .await
            .context("Failed to read blank frame")?;

        let _ = fs::remove_file(&temp_file).await;

        Ok(frame_data)
    }

    /// Clear the preview cache
    pub async fn clear_cache(&self) {
        self.cache.clear().await;
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> crate::preview_cache::CacheStats {
        self.cache.stats().await
    }
}

impl Default for PreviewService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Resolution, Track, TrackType};

    #[test]
    fn test_find_active_clips() {
        let service = PreviewService::new();

        let timeline = Timeline {
            id: "test".to_string(),
            name: "Test".to_string(),
            framerate: 30.0,
            resolution: Resolution {
                width: 1920,
                height: 1080,
            },
            duration: 30.0,
            tracks: vec![Track {
                id: "track1".to_string(),
                track_type: TrackType::Video,
                muted: false,
                locked: false,
                clips: vec![
                    Clip {
                        id: "clip1".to_string(),
                        media_file_id: "media1".to_string(),
                        name: Some("test-clip1.mp4".to_string()),
                        track_position: 0.0,
                        duration: 10.0,
                        trim_start: 0.0,
                        trim_end: 10.0,
                        effects: vec![],
                        volume: 1.0,
                        speed: 1.0,
                    },
                    Clip {
                        id: "clip2".to_string(),
                        media_file_id: "media2".to_string(),
                        name: Some("test-clip2.mp4".to_string()),
                        track_position: 10.0,
                        duration: 10.0,
                        trim_start: 0.0,
                        trim_end: 10.0,
                        effects: vec![],
                        volume: 1.0,
                        speed: 1.0,
                    },
                ],
            }],
        };

        // Test at 5.0 seconds (should find clip1)
        let active = service.find_active_clips(&timeline, 5.0);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, "clip1");

        // Test at 15.0 seconds (should find clip2)
        let active = service.find_active_clips(&timeline, 15.0);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, "clip2");

        // Test at 25.0 seconds (should find nothing)
        let active = service.find_active_clips(&timeline, 25.0);
        assert_eq!(active.len(), 0);
    }
}
