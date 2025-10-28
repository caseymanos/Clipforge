use crate::models::{
    Timeline, Track, Clip, Effect, EffectType, TrackType,
    ExportSettings, ExportProgress, ExportError, MediaFile,
};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as TokioCommand;
use tauri::Window;
use log::{info, error, warn};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

/// Export service for rendering timelines to video files
pub struct ExportService {
    ffmpeg_path: String,
    cancel_flag: Arc<AtomicBool>,
}

impl ExportService {
    /// Create a new export service
    pub fn new() -> Result<Self, ExportError> {
        // Verify FFmpeg is available
        let ffmpeg_path = Self::find_ffmpeg()?;

        Ok(Self {
            ffmpeg_path,
            cancel_flag: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Find FFmpeg executable
    fn find_ffmpeg() -> Result<String, ExportError> {
        // Try system FFmpeg first
        if let Ok(output) = Command::new("ffmpeg").arg("-version").output() {
            if output.status.success() {
                return Ok("ffmpeg".to_string());
            }
        }

        // Check common installation paths
        let paths = vec![
            "/usr/local/bin/ffmpeg",
            "/opt/homebrew/bin/ffmpeg",
            "C:\\Program Files\\ffmpeg\\bin\\ffmpeg.exe",
        ];

        for path in paths {
            if Path::new(path).exists() {
                return Ok(path.to_string());
            }
        }

        Err(ExportError::FFmpegError("FFmpeg not found in system PATH".to_string()))
    }

    /// Export a timeline to a video file
    pub async fn export_timeline(
        &self,
        timeline: &Timeline,
        settings: &ExportSettings,
        output_path: PathBuf,
        media_files: &HashMap<String, MediaFile>,
        window: Window,
    ) -> Result<PathBuf, ExportError> {
        info!("Starting export to: {:?}", output_path);

        // Reset cancel flag
        self.cancel_flag.store(false, Ordering::Relaxed);

        // Step 1: Validate timeline
        self.validate_timeline(timeline, media_files)?;

        // Step 2: Build FFmpeg command
        let ffmpeg_args = self.build_ffmpeg_command(
            timeline,
            settings,
            &output_path,
            media_files,
        )?;

        // Step 3: Execute FFmpeg with progress tracking
        self.execute_ffmpeg(ffmpeg_args, timeline.duration, window).await?;

        // Step 4: Verify output file
        if !output_path.exists() {
            return Err(ExportError::OutputError("Output file was not created".to_string()));
        }

        info!("Export completed successfully: {:?}", output_path);
        Ok(output_path)
    }

    /// Validate timeline before export
    fn validate_timeline(
        &self,
        timeline: &Timeline,
        media_files: &HashMap<String, MediaFile>,
    ) -> Result<(), ExportError> {
        // Check if timeline has any video tracks
        let has_video = timeline.tracks.iter()
            .any(|t| matches!(t.track_type, TrackType::Video) && t.enabled && !t.clips.is_empty());

        if !has_video {
            return Err(ExportError::ValidationError(
                "Timeline must have at least one enabled video track with clips".to_string()
            ));
        }

        // Validate all clips reference existing media files
        for track in &timeline.tracks {
            for clip in &track.clips {
                if !media_files.contains_key(&clip.media_file_id) {
                    return Err(ExportError::ValidationError(
                        format!("Clip references missing media file: {}", clip.media_file_id)
                    ));
                }

                // Validate clip duration
                if clip.duration <= 0.0 {
                    return Err(ExportError::ValidationError(
                        format!("Clip has invalid duration: {}", clip.duration)
                    ));
                }
            }
        }

        Ok(())
    }

    /// Build FFmpeg command with filter_complex
    fn build_ffmpeg_command(
        &self,
        timeline: &Timeline,
        settings: &ExportSettings,
        output_path: &Path,
        media_files: &HashMap<String, MediaFile>,
    ) -> Result<Vec<String>, ExportError> {
        let mut args = Vec::new();

        // Overwrite output file
        args.push("-y".to_string());

        // Add all input files
        let mut input_map: HashMap<String, usize> = HashMap::new();
        let mut input_index = 0;

        for track in &timeline.tracks {
            if !track.enabled {
                continue;
            }

            for clip in &track.clips {
                let media_file = media_files.get(&clip.media_file_id)
                    .ok_or_else(|| ExportError::ValidationError(
                        format!("Media file not found: {}", clip.media_file_id)
                    ))?;

                if !input_map.contains_key(&clip.media_file_id) {
                    args.push("-i".to_string());
                    args.push(media_file.path.to_string_lossy().to_string());
                    input_map.insert(clip.media_file_id.clone(), input_index);
                    input_index += 1;
                }
            }
        }

        // Build filter_complex for timeline
        let filter_complex = self.build_filter_complex(timeline, &input_map, media_files)?;

        if !filter_complex.is_empty() {
            args.push("-filter_complex".to_string());
            args.push(filter_complex);
        }

        // Output settings
        args.push("-c:v".to_string());
        args.push(settings.video_codec.clone());

        args.push("-b:v".to_string());
        args.push(format!("{}k", settings.video_bitrate));

        args.push("-c:a".to_string());
        args.push(settings.audio_codec.clone());

        args.push("-b:a".to_string());
        args.push(format!("{}k", settings.audio_bitrate));

        args.push("-r".to_string());
        args.push(format!("{}", settings.framerate));

        args.push("-s".to_string());
        args.push(format!("{}x{}", settings.resolution.width, settings.resolution.height));

        // Progress reporting
        args.push("-progress".to_string());
        args.push("pipe:1".to_string());

        // Output file
        args.push(output_path.to_string_lossy().to_string());

        Ok(args)
    }

    /// Build filter_complex string for timeline
    fn build_filter_complex(
        &self,
        timeline: &Timeline,
        input_map: &HashMap<String, usize>,
        media_files: &HashMap<String, MediaFile>,
    ) -> Result<String, ExportError> {
        let mut filters = Vec::new();
        let mut video_inputs = Vec::new();
        let mut audio_inputs = Vec::new();

        // Process each track
        for (track_idx, track) in timeline.tracks.iter().enumerate() {
            if !track.enabled {
                continue;
            }

            match track.track_type {
                TrackType::Video | TrackType::Overlay => {
                    // Process video clips
                    for (clip_idx, clip) in track.clips.iter().enumerate() {
                        let input_idx = input_map.get(&clip.media_file_id)
                            .ok_or_else(|| ExportError::ValidationError(
                                format!("Input mapping not found for: {}", clip.media_file_id)
                            ))?;

                        let label = format!("v{}_{}", track_idx, clip_idx);

                        // Trim and scale clip
                        let mut clip_filter = format!(
                            "[{}:v]trim=start={}:duration={},setpts=PTS-STARTPTS",
                            input_idx, clip.trim_start, clip.duration
                        );

                        // Apply effects
                        if !clip.effects.is_empty() {
                            let effects_str = self.build_effects_filter(&clip.effects)?;
                            clip_filter.push_str(&format!(",{}", effects_str));
                        }

                        // Apply speed
                        if (clip.speed - 1.0).abs() > 0.01 {
                            clip_filter.push_str(&format!(",setpts={}*PTS", 1.0 / clip.speed));
                        }

                        clip_filter.push_str(&format!("[{}]", label));
                        filters.push(clip_filter);
                        video_inputs.push(label);
                    }
                }
                TrackType::Audio => {
                    // Process audio clips
                    for (clip_idx, clip) in track.clips.iter().enumerate() {
                        let input_idx = input_map.get(&clip.media_file_id)
                            .ok_or_else(|| ExportError::ValidationError(
                                format!("Input mapping not found for: {}", clip.media_file_id)
                            ))?;

                        let label = format!("a{}_{}", track_idx, clip_idx);

                        let mut clip_filter = format!(
                            "[{}:a]atrim=start={}:duration={},asetpts=PTS-STARTPTS",
                            input_idx, clip.trim_start, clip.duration
                        );

                        // Apply volume
                        if (clip.volume - 1.0).abs() > 0.01 {
                            clip_filter.push_str(&format!(",volume={}", clip.volume));
                        }

                        clip_filter.push_str(&format!("[{}]", label));
                        filters.push(clip_filter);
                        audio_inputs.push(label);
                    }
                }
            }
        }

        // Concatenate all video inputs
        if !video_inputs.is_empty() {
            let concat_filter = format!(
                "{}concat=n={}:v=1:a=0[outv]",
                video_inputs.iter().map(|l| format!("[{}]", l)).collect::<Vec<_>>().join(""),
                video_inputs.len()
            );
            filters.push(concat_filter);
        }

        // Concatenate all audio inputs
        if !audio_inputs.is_empty() {
            let concat_filter = format!(
                "{}concat=n={}:v=0:a=1[outa]",
                audio_inputs.iter().map(|l| format!("[{}]", l)).collect::<Vec<_>>().join(""),
                audio_inputs.len()
            );
            filters.push(concat_filter);
        }

        Ok(filters.join(";"))
    }

    /// Build effects filter string
    fn build_effects_filter(&self, effects: &[Effect]) -> Result<String, ExportError> {
        let mut filters = Vec::new();

        for effect in effects {
            let filter = match effect.effect_type {
                EffectType::Brightness => {
                    format!("eq=brightness={}", effect.intensity)
                }
                EffectType::Contrast => {
                    format!("eq=contrast={}", effect.intensity)
                }
                EffectType::Saturation => {
                    format!("eq=saturation={}", effect.intensity)
                }
                EffectType::Blur => {
                    format!("boxblur={}:1", (effect.intensity * 10.0) as i32)
                }
                EffectType::FadeIn => {
                    format!("fade=t=in:st=0:d={}", effect.intensity)
                }
                EffectType::FadeOut => {
                    format!("fade=t=out:st=0:d={}", effect.intensity)
                }
            };
            filters.push(filter);
        }

        Ok(filters.join(","))
    }

    /// Execute FFmpeg command with progress tracking
    async fn execute_ffmpeg(
        &self,
        args: Vec<String>,
        total_duration: f64,
        window: Window,
    ) -> Result<(), ExportError> {
        info!("Executing FFmpeg with {} arguments", args.len());

        let mut child = TokioCommand::new(&self.ffmpeg_path)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ExportError::FFmpegError(format!("Failed to spawn FFmpeg: {}", e)))?;

        let stdout = child.stdout.take()
            .ok_or_else(|| ExportError::FFmpegError("Failed to capture stdout".to_string()))?;

        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        let cancel_flag = self.cancel_flag.clone();

        // Read progress from stdout
        while let Ok(Some(line)) = lines.next_line().await {
            // Check for cancellation
            if cancel_flag.load(Ordering::Relaxed) {
                let _ = child.kill().await;
                return Err(ExportError::Cancelled);
            }

            // Parse progress line (format: "out_time_ms=123456")
            if line.starts_with("out_time_ms=") {
                if let Some(time_str) = line.strip_prefix("out_time_ms=") {
                    if let Ok(time_us) = time_str.parse::<u64>() {
                        let current_time = time_us as f64 / 1_000_000.0;
                        let percentage = (current_time / total_duration * 100.0).min(100.0);

                        let progress = ExportProgress {
                            percentage,
                            current_frame: (current_time * 30.0) as u64, // Assume 30fps
                            fps: 30.0,
                            time_remaining_secs: ((total_duration - current_time) / 30.0 * 30.0) as u64,
                        };

                        // Emit progress event
                        let _ = window.emit("export-progress", progress);
                    }
                }
            }
        }

        // Wait for process to complete
        let status = child.wait().await
            .map_err(|e| ExportError::FFmpegError(format!("FFmpeg process failed: {}", e)))?;

        if !status.success() {
            return Err(ExportError::FFmpegError(
                format!("FFmpeg exited with status: {}", status)
            ));
        }

        // Emit completion event
        let _ = window.emit("export-complete", ());

        Ok(())
    }

    /// Cancel ongoing export
    pub fn cancel_export(&self) {
        info!("Cancelling export");
        self.cancel_flag.store(true, Ordering::Relaxed);
    }

    /// Get available export presets
    pub fn get_presets() -> Vec<(String, ExportSettings)> {
        vec![
            ("YouTube 1080p".to_string(), ExportSettings::youtube_1080p()),
            ("Instagram Post".to_string(), ExportSettings::instagram_post()),
            ("Twitter Video".to_string(), ExportSettings::twitter_video()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_service_creation() {
        let service = ExportService::new();
        assert!(service.is_ok());
    }

    #[test]
    fn test_presets() {
        let presets = ExportService::get_presets();
        assert_eq!(presets.len(), 3);
        assert_eq!(presets[0].0, "YouTube 1080p");
    }
}
