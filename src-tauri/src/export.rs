use crate::models::{
    Timeline, Effect, EffectType, TrackType,
    ExportSettings, ExportProgress, ExportError, MediaFile,
    SubtitleTrack,
};
use crate::ffmpeg_utils;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as TokioCommand;
use tauri::{Window, Emitter};
use log::info;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Export service for rendering timelines to video files
pub struct ExportService {
    ffmpeg_path: String,
    cancel_flag: Arc<AtomicBool>,
}

impl ExportService {
    /// Create a new export service
    pub fn new() -> Result<Self, ExportError> {
        // Use shared utility to find FFmpeg
        let ffmpeg_path = ffmpeg_utils::find_ffmpeg_path()
            .map_err(|e| ExportError::FFmpegError(e))?;

        let ffmpeg_path_str = ffmpeg_path.to_str()
            .ok_or_else(|| ExportError::FFmpegError("Invalid FFmpeg path".to_string()))?
            .to_string();

        Ok(Self {
            ffmpeg_path: ffmpeg_path_str,
            cancel_flag: Arc::new(AtomicBool::new(false)),
        })
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
            .any(|t| matches!(t.track_type, TrackType::Video) && !t.muted && !t.clips.is_empty());

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
            if track.muted {
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

        info!("Generated filter_complex ({} bytes): {}", filter_complex.len(),
            if filter_complex.len() > 500 {
                format!("{}...", &filter_complex[..500])
            } else {
                filter_complex.clone()
            });

        if !filter_complex.is_empty() {
            args.push("-filter_complex".to_string());
            args.push(filter_complex);

            // Map the filtered outputs
            args.push("-map".to_string());
            args.push("[outv]".to_string());
            args.push("-map".to_string());
            args.push("[outa]".to_string());
        }

        // Performance and stability flags
        args.push("-threads".to_string());
        args.push("0".to_string()); // Auto-detect optimal thread count

        args.push("-max_muxing_queue_size".to_string());
        args.push("1024".to_string()); // Prevent queue exhaustion

        args.push("-max_interleave_delta".to_string());
        args.push("500M".to_string()); // Reduce buffering

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
        _media_files: &HashMap<String, MediaFile>,
    ) -> Result<String, ExportError> {
        let mut filters = Vec::new();
        let mut video_inputs = Vec::new();
        let mut audio_inputs = Vec::new();

        // Count how many times each input is used for video and audio
        let mut video_usage_count: HashMap<usize, usize> = HashMap::new();
        let mut audio_usage_count: HashMap<usize, usize> = HashMap::new();

        for track in &timeline.tracks {
            if track.muted {
                continue;
            }

            match track.track_type {
                TrackType::Video | TrackType::Overlay => {
                    for clip in &track.clips {
                        if let Some(&input_idx) = input_map.get(&clip.media_file_id) {
                            *video_usage_count.entry(input_idx).or_insert(0) += 1;
                        }
                    }
                }
                TrackType::Audio => {
                    for clip in &track.clips {
                        if let Some(&input_idx) = input_map.get(&clip.media_file_id) {
                            *audio_usage_count.entry(input_idx).or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        // Create split filters for inputs used multiple times
        let mut video_split_map: HashMap<usize, Vec<String>> = HashMap::new();
        let mut audio_split_map: HashMap<usize, Vec<String>> = HashMap::new();

        for (&input_idx, &count) in &video_usage_count {
            if count > 1 {
                let split_outputs: Vec<String> = (0..count)
                    .map(|i| format!("vsplit{}_tmp{}", input_idx, i))
                    .collect();

                let split_filter = format!(
                    "[{}:v]split={}{}",
                    input_idx,
                    count,
                    split_outputs.iter().map(|s| format!("[{}]", s)).collect::<Vec<_>>().join("")
                );
                filters.push(split_filter);
                video_split_map.insert(input_idx, split_outputs);

                info!("Created video split filter for input {} with {} outputs", input_idx, count);
            }
        }

        for (&input_idx, &count) in &audio_usage_count {
            if count > 1 {
                let split_outputs: Vec<String> = (0..count)
                    .map(|i| format!("asplit{}_tmp{}", input_idx, i))
                    .collect();

                let split_filter = format!(
                    "[{}:a]asplit={}{}",
                    input_idx,
                    count,
                    split_outputs.iter().map(|s| format!("[{}]", s)).collect::<Vec<_>>().join("")
                );
                filters.push(split_filter);
                audio_split_map.insert(input_idx, split_outputs);

                info!("Created audio split filter for input {} with {} outputs", input_idx, count);
            }
        }

        // Track which split output to use next for each input
        let mut video_split_counters: HashMap<usize, usize> = HashMap::new();
        let mut audio_split_counters: HashMap<usize, usize> = HashMap::new();

        // Process each track
        for (track_idx, track) in timeline.tracks.iter().enumerate() {
            if track.muted {
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

                        // Determine source stream (either split output or direct input)
                        let source_stream = if let Some(split_outputs) = video_split_map.get(input_idx) {
                            let counter = video_split_counters.entry(*input_idx).or_insert(0);
                            let stream = split_outputs[*counter].clone();
                            *counter += 1;
                            format!("[{}]", stream)
                        } else {
                            format!("[{}:v]", input_idx)
                        };

                        // Trim and scale clip
                        let mut clip_filter = format!(
                            "{}trim=start={}:duration={},setpts=PTS-STARTPTS",
                            source_stream, clip.trim_start, clip.duration
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

                        // Determine source stream (either split output or direct input)
                        let source_stream = if let Some(split_outputs) = audio_split_map.get(input_idx) {
                            let counter = audio_split_counters.entry(*input_idx).or_insert(0);
                            let stream = split_outputs[*counter].clone();
                            *counter += 1;
                            format!("[{}]", stream)
                        } else {
                            format!("[{}:a]", input_idx)
                        };

                        let mut clip_filter = format!(
                            "{}atrim=start={}:duration={},asetpts=PTS-STARTPTS",
                            source_stream, clip.trim_start, clip.duration
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
            let video_label = if timeline.subtitle_enabled && timeline.subtitle_track.is_some() {
                "[vconcat]" // Intermediate label for subtitle burning
            } else {
                "[outv]" // Final output if no subtitles
            };

            let concat_filter = format!(
                "{}concat=n={}:v=1:a=0{}",
                video_inputs.iter().map(|l| format!("[{}]", l)).collect::<Vec<_>>().join(""),
                video_inputs.len(),
                video_label
            );
            filters.push(concat_filter);

            // Add subtitle burning if enabled
            if timeline.subtitle_enabled {
                if let Some(ref subtitle_track) = timeline.subtitle_track {
                    // Generate SRT content
                    let srt_content = self.generate_srt_content(subtitle_track)?;

                    // Create temporary SRT file path
                    let temp_srt = std::env::temp_dir().join(format!("clipforge_subtitles_{}.srt",
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs()));

                    // Write SRT file
                    std::fs::write(&temp_srt, srt_content)
                        .map_err(|e| ExportError::OutputError(format!("Failed to write SRT: {}", e)))?;

                    // Add subtitles filter
                    let subtitle_filter = format!(
                        "[vconcat]subtitles={}:force_style='FontName=Arial,FontSize=24,PrimaryColour=&H00FFFFFF,OutlineColour=&H00000000,BorderStyle=3,Outline=2,Shadow=1,MarginV=20'[outv]",
                        temp_srt.to_string_lossy().replace("\\", "\\\\").replace(":", "\\:")
                    );
                    filters.push(subtitle_filter);

                    info!("Added subtitle burning filter with {} segments", subtitle_track.segments.len());
                }
            }
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
            if !effect.enabled {
                continue;
            }

            let filter = match &effect.effect_type {
                EffectType::Brightness { value } => {
                    format!("eq=brightness={}", value)
                }
                EffectType::Contrast { value } => {
                    format!("eq=contrast={}", value)
                }
                EffectType::Saturation { value } => {
                    format!("eq=saturation={}", value)
                }
                EffectType::Blur { radius } => {
                    format!("boxblur={}:1", (*radius * 10.0) as i32)
                }
                EffectType::Sharpen { amount } => {
                    format!("unsharp=5:5:{}", amount * 2.0)
                }
                EffectType::Normalize => {
                    "loudnorm".to_string()
                }
                EffectType::FadeIn { duration } => {
                    format!("fade=t=in:st=0:d={}", duration)
                }
                EffectType::FadeOut { duration } => {
                    format!("fade=t=out:st=0:d={}", duration)
                }
            };
            filters.push(filter);
        }

        Ok(filters.join(","))
    }

    /// Generate SRT content from subtitle track
    fn generate_srt_content(&self, track: &SubtitleTrack) -> Result<String, ExportError> {
        let mut srt = String::new();

        for (idx, segment) in track.segments.iter().enumerate() {
            // Index (1-based)
            srt.push_str(&format!("{}\n", idx + 1));

            // Timecode format: HH:MM:SS,mmm --> HH:MM:SS,mmm
            let start = self.format_srt_timestamp(segment.start_time);
            let end = self.format_srt_timestamp(segment.end_time);
            srt.push_str(&format!("{} --> {}\n", start, end));

            // Text
            srt.push_str(&format!("{}\n\n", segment.text));
        }

        Ok(srt)
    }

    /// Format timestamp for SRT format
    fn format_srt_timestamp(&self, seconds: f64) -> String {
        let hours = (seconds / 3600.0) as u32;
        let minutes = ((seconds % 3600.0) / 60.0) as u32;
        let secs = (seconds % 60.0) as u32;
        let millis = ((seconds % 1.0) * 1000.0) as u32;

        format!("{:02}:{:02}:{:02},{:03}", hours, minutes, secs, millis)
    }

    /// Execute FFmpeg command with progress tracking
    async fn execute_ffmpeg(
        &self,
        args: Vec<String>,
        total_duration: f64,
        window: Window,
    ) -> Result<(), ExportError> {
        info!("=== Starting FFmpeg Export ===");
        info!("Total expected duration: {:.2} seconds", total_duration);
        info!("Executing FFmpeg with {} arguments", args.len());
        info!("FFmpeg command: {} {}", self.ffmpeg_path, args.join(" "));

        info!("Spawning FFmpeg process...");
        let spawn_start = std::time::Instant::now();
        let mut child = TokioCommand::new(&self.ffmpeg_path)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ExportError::FFmpegError(format!("Failed to spawn FFmpeg: {}", e)))?;

        info!("FFmpeg process spawned successfully in {:?}", spawn_start.elapsed());

        let stdout = child.stdout.take()
            .ok_or_else(|| ExportError::FFmpegError("Failed to capture stdout".to_string()))?;

        let stderr = child.stderr.take()
            .ok_or_else(|| ExportError::FFmpegError("Failed to capture stderr".to_string()))?;

        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        let stderr_reader = BufReader::new(stderr);
        let mut stderr_lines = stderr_reader.lines();

        let cancel_flag = self.cancel_flag.clone();

        // Spawn task to read stderr
        let stderr_task = tokio::spawn(async move {
            let mut error_output = Vec::new();
            while let Ok(Some(line)) = stderr_lines.next_line().await {
                log::debug!("FFmpeg stderr: {}", line);
                error_output.push(line);
            }
            error_output
        });

        // Read progress from stdout
        let mut last_progress_log = std::time::Instant::now();
        let mut last_progress_update = std::time::Instant::now();
        let mut progress_count = 0;

        info!("Starting to read progress from FFmpeg stdout...");

        while let Ok(Some(line)) = lines.next_line().await {
            // Check for cancellation
            if cancel_flag.load(Ordering::Relaxed) {
                info!("Export cancelled by user");
                let _ = child.kill().await;
                return Err(ExportError::Cancelled);
            }

            // Warn if no progress for 15 seconds
            if last_progress_update.elapsed().as_secs() > 15 {
                log::warn!("No progress update for {} seconds - FFmpeg may be stalled",
                    last_progress_update.elapsed().as_secs());
                last_progress_update = std::time::Instant::now(); // Reset to avoid spam
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

                        progress_count += 1;
                        last_progress_update = std::time::Instant::now();

                        // Log progress every 2 seconds
                        if last_progress_log.elapsed().as_secs() >= 2 {
                            info!("Export progress: {:.1}% ({:.2}s / {:.2}s) [update #{}]",
                                percentage, current_time, total_duration, progress_count);
                            last_progress_log = std::time::Instant::now();
                        }

                        // Emit progress event
                        let _ = window.emit("export-progress", progress);
                    }
                }
            }
        }

        info!("Finished reading FFmpeg stdout after {} progress updates", progress_count);

        info!("Finished reading FFmpeg stdout, waiting for process to complete");

        // Wait for process to complete
        let status = child.wait().await
            .map_err(|e| ExportError::FFmpegError(format!("FFmpeg process failed: {}", e)))?;

        // Get stderr output
        let stderr_output = stderr_task.await.unwrap_or_default();

        if !status.success() {
            let error_message = if !stderr_output.is_empty() {
                format!("FFmpeg exited with status: {}. Error output:\n{}",
                    status,
                    stderr_output.join("\n"))
            } else {
                format!("FFmpeg exited with status: {}", status)
            };

            log::error!("FFmpeg error: {}", error_message);
            return Err(ExportError::FFmpegError(error_message));
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
