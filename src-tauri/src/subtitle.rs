use crate::models::{MediaFile, SubtitleSegment, SubtitleSource, SubtitleTrack, SubtitleError, Timeline, Clip};
use crate::ffmpeg_utils;
use log::{info, warn, error};
use reqwest::multipart;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{BufReader, Read};
use tauri::{Window, Emitter};

/// OpenAI Whisper API response format
#[derive(Debug, Deserialize)]
struct WhisperResponse {
    text: String,
}

/// OpenAI Whisper verbose JSON response with segments
#[derive(Debug, Deserialize)]
struct WhisperVerboseResponse {
    text: String,
    segments: Vec<WhisperSegment>,
}

#[derive(Debug, Deserialize)]
struct WhisperSegment {
    id: usize,
    start: f64,
    end: f64,
    text: String,
}

/// Progress event payload
#[derive(Debug, Clone, Serialize)]
struct SubtitleProgress {
    stage: String,
    progress: f64,  // 0.0 to 1.0
}

/// Subtitle service for AI transcription and SRT handling
pub struct SubtitleService {
    api_key: String,
    cache_dir: PathBuf,
    client: reqwest::Client,
}

impl SubtitleService {
    /// Create a new subtitle service
    pub fn new(api_key: String) -> Result<Self, SubtitleError> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| SubtitleError::CacheError("Could not find cache directory".into()))?
            .join("clipforge")
            .join("subtitle_cache");

        fs::create_dir_all(&cache_dir)?;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))  // 5 minute timeout
            .build()?;

        Ok(Self {
            api_key,
            cache_dir,
            client,
        })
    }

    /// Transcribe a media file using OpenAI Whisper API
    pub async fn transcribe_media_file(
        &self,
        media_file: &MediaFile,
        language: Option<String>,
        window: Option<Window>,
    ) -> Result<SubtitleTrack, SubtitleError> {
        info!("Starting transcription for media file: {:?}", media_file.path);

        // Emit progress: Starting
        if let Some(ref win) = window {
            let _ = win.emit("subtitle:progress", SubtitleProgress {
                stage: "Preparing audio".to_string(),
                progress: 0.1,
            });
        }

        // Check cache first
        let file_hash = self.compute_file_hash(&media_file.path)?;
        let lang = language.as_deref().unwrap_or("en");

        if let Some(cached_track) = self.get_cached_transcription(&file_hash, lang)? {
            info!("Using cached transcription for file hash: {}", file_hash);
            if let Some(ref win) = window {
                let _ = win.emit("subtitle:progress", SubtitleProgress {
                    stage: "Loaded from cache".to_string(),
                    progress: 1.0,
                });
            }
            return Ok(cached_track);
        }

        // Extract audio if needed (for video files)
        let audio_path = if media_file.codec.video.is_some() {
            if let Some(ref win) = window {
                let _ = win.emit("subtitle:progress", SubtitleProgress {
                    stage: "Extracting audio".to_string(),
                    progress: 0.2,
                });
            }
            self.extract_audio(&media_file.path).await?
        } else {
            media_file.path.clone()
        };

        // Call OpenAI Whisper API
        if let Some(ref win) = window {
            let _ = win.emit("subtitle:progress", SubtitleProgress {
                stage: "Transcribing with OpenAI Whisper".to_string(),
                progress: 0.4,
            });
        }

        let segments = self.call_whisper_api(&audio_path, language.clone()).await?;

        // Clean up extracted audio if temporary
        if audio_path != media_file.path {
            let _ = fs::remove_file(&audio_path);
        }

        // Create subtitle track
        let track = SubtitleTrack {
            segments,
            language: lang.to_string(),
            source: SubtitleSource::Transcribed {
                media_file_id: media_file.id.clone(),
                provider: "openai-whisper".to_string(),
            },
            style: Default::default(),
        };

        // Cache the result
        self.cache_transcription(&file_hash, lang, &track)?;

        if let Some(ref win) = window {
            let _ = win.emit("subtitle:progress", SubtitleProgress {
                stage: "Complete".to_string(),
                progress: 1.0,
            });
        }

        info!("Transcription complete: {} segments", track.segments.len());
        Ok(track)
    }

    /// Transcribe timeline audio by extracting and merging all audio clips
    pub async fn transcribe_timeline(
        &self,
        timeline: &Timeline,
        audio_clips: Vec<(Clip, MediaFile)>,
        language: Option<String>,
        window: Option<Window>,
    ) -> Result<SubtitleTrack, SubtitleError> {
        info!("Starting timeline transcription with {} audio clips", audio_clips.len());

        // Emit progress: Starting
        if let Some(ref win) = window {
            let _ = win.emit("subtitle:progress", SubtitleProgress {
                stage: "Extracting timeline audio".to_string(),
                progress: 0.1,
            });
        }

        // Extract timeline audio to temporary file
        let timeline_audio_path = self.extract_timeline_audio(&audio_clips, window.as_ref()).await?;

        // Call OpenAI Whisper API
        if let Some(ref win) = window {
            let _ = win.emit("subtitle:progress", SubtitleProgress {
                stage: "Transcribing with OpenAI Whisper".to_string(),
                progress: 0.5,
            });
        }

        let segments = self.call_whisper_api(&timeline_audio_path, language.clone()).await?;

        // Clean up temporary timeline audio file
        let _ = fs::remove_file(&timeline_audio_path);

        // Create subtitle track
        let lang = language.as_deref().unwrap_or("en");
        let track = SubtitleTrack {
            segments,
            language: lang.to_string(),
            source: SubtitleSource::Transcribed {
                media_file_id: timeline.id.clone(),
                provider: "openai-whisper".to_string(),
            },
            style: Default::default(),
        };

        if let Some(ref win) = window {
            let _ = win.emit("subtitle:progress", SubtitleProgress {
                stage: "Complete".to_string(),
                progress: 1.0,
            });
        }

        info!("Timeline transcription complete: {} segments", track.segments.len());
        Ok(track)
    }

    /// Extract and merge audio from timeline clips
    async fn extract_timeline_audio(
        &self,
        audio_clips: &[(Clip, MediaFile)],
        window: Option<&Window>,
    ) -> Result<PathBuf, SubtitleError> {
        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join(format!("clipforge_timeline_audio_{}.mp3", uuid::Uuid::new_v4()));

        info!("Extracting timeline audio to: {:?}", output_path);

        // Sort clips by timeline position
        let mut sorted_clips: Vec<_> = audio_clips.iter().collect();
        sorted_clips.sort_by(|a, b| a.0.track_position.partial_cmp(&b.0.track_position).unwrap());

        // Build FFmpeg concat filter
        // For simple timeline with non-overlapping clips, we can use concat demuxer
        // For now, let's extract each clip's audio and concat them

        let mut temp_files = Vec::new();
        let mut filter_inputs = Vec::new();

        for (idx, (clip, media_file)) in sorted_clips.iter().enumerate() {
            if let Some(ref win) = window {
                let progress = 0.1 + (0.3 * (idx as f64 / sorted_clips.len() as f64));
                let _ = win.emit("subtitle:progress", SubtitleProgress {
                    stage: format!("Extracting clip {}/{}", idx + 1, sorted_clips.len()),
                    progress,
                });
            }

            // Extract this clip's audio segment
            let clip_audio_path = temp_dir.join(format!("clipforge_clip_audio_{}_{}.mp3", uuid::Uuid::new_v4(), idx));

            // Calculate duration from trim points
            let duration = clip.trim_end - clip.trim_start;

            info!(
                "Extracting audio from clip {}: start={}, duration={}, file={:?}",
                idx, clip.trim_start, duration, media_file.path
            );

            let ffmpeg_path = ffmpeg_utils::find_ffmpeg_path()
                .map_err(|e| SubtitleError::CacheError(e))?;
            let output = tokio::process::Command::new(&ffmpeg_path)
                .arg("-y")
                .arg("-ss")
                .arg(clip.trim_start.to_string())
                .arg("-t")
                .arg(duration.to_string())
                .arg("-i")
                .arg(&media_file.path)
                .arg("-vn")  // No video
                .arg("-ar")
                .arg("16000")  // 16kHz sample rate (Whisper recommended)
                .arg("-ac")
                .arg("1")  // Mono
                .arg("-b:a")
                .arg("64k")
                .arg(&clip_audio_path)
                .output()
                .await
                .map_err(|e| SubtitleError::IoError(e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!("FFmpeg failed to extract clip audio: {}", stderr);
                // Clean up temp files
                for temp_file in &temp_files {
                    let _ = fs::remove_file(temp_file);
                }
                return Err(SubtitleError::CacheError(format!("Failed to extract clip audio: {}", stderr)));
            }

            temp_files.push(clip_audio_path.clone());
            filter_inputs.push(format!("[{}:a]", idx));
        }

        // Now concat all audio clips
        if let Some(ref win) = window {
            let _ = win.emit("subtitle:progress", SubtitleProgress {
                stage: "Merging audio clips".to_string(),
                progress: 0.4,
            });
        }

        info!("Concatenating {} audio clips", temp_files.len());

        // Build FFmpeg command to concat all clips
        let ffmpeg_path = ffmpeg_utils::find_ffmpeg_path()
            .map_err(|e| SubtitleError::CacheError(e))?;
        let mut cmd = tokio::process::Command::new(&ffmpeg_path);
        cmd.arg("-y");

        // Add all input files
        for temp_file in &temp_files {
            cmd.arg("-i").arg(temp_file);
        }

        // Build concat filter
        let concat_filter = format!("{}concat=n={}:v=0:a=1[outa]",
            filter_inputs.join(""),
            temp_files.len()
        );

        cmd.arg("-filter_complex")
            .arg(&concat_filter)
            .arg("-map")
            .arg("[outa]")
            .arg(&output_path);

        let output = cmd.output()
            .await
            .map_err(|e| SubtitleError::IoError(e))?;

        // Clean up temp clip files
        for temp_file in &temp_files {
            let _ = fs::remove_file(temp_file);
        }

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("FFmpeg failed to concat audio: {}", stderr);
            return Err(SubtitleError::CacheError(format!("Failed to concat timeline audio: {}", stderr)));
        }

        Ok(output_path)
    }

    /// Extract audio from video file using FFmpeg
    async fn extract_audio(&self, video_path: &Path) -> Result<PathBuf, SubtitleError> {
        let temp_dir = std::env::temp_dir();
        let audio_path = temp_dir.join(format!("clipforge_audio_{}.mp3", uuid::Uuid::new_v4()));

        info!("Extracting audio to: {:?}", audio_path);

        let output = tokio::process::Command::new("ffmpeg")
            .arg("-y")
            .arg("-i")
            .arg(video_path)
            .arg("-vn")  // No video
            .arg("-ar")
            .arg("16000")  // 16kHz sample rate (Whisper recommended)
            .arg("-ac")
            .arg("1")  // Mono
            .arg("-b:a")
            .arg("64k")  // Low bitrate for smaller upload
            .arg(&audio_path)
            .output()
            .await
            .map_err(|e| SubtitleError::IoError(e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let error_msg = format!(
                "FFmpeg failed to extract audio from {:?}. Exit code: {:?}. stderr: {}",
                video_path,
                output.status.code(),
                stderr
            );
            error!("{}", error_msg);
            return Err(SubtitleError::CacheError(error_msg));
        }

        Ok(audio_path)
    }

    /// Call OpenAI Whisper API
    async fn call_whisper_api(
        &self,
        audio_path: &Path,
        language: Option<String>,
    ) -> Result<Vec<SubtitleSegment>, SubtitleError> {
        info!("Calling OpenAI Whisper API");

        // Read audio file
        let audio_bytes = fs::read(audio_path)?;
        let file_name = audio_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("audio.mp3");

        // Build multipart form
        let file_part = multipart::Part::bytes(audio_bytes)
            .file_name(file_name.to_string())
            .mime_str("audio/mpeg")?;

        let mut form = multipart::Form::new()
            .part("file", file_part)
            .text("model", "whisper-1")
            .text("response_format", "verbose_json")  // Get segments with timing
            .text("timestamp_granularities[]", "segment");

        if let Some(lang) = language {
            form = form.text("language", lang);
        }

        // Make API request
        let response = self.client
            .post("https://api.openai.com/v1/audio/transcriptions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("OpenAI API error: {}", error_text);
            return Err(SubtitleError::ApiError(error_text));
        }

        let verbose_response: WhisperVerboseResponse = response.json().await?;

        // Convert to SubtitleSegment format with sequential 1-based IDs
        let segments = verbose_response.segments
            .into_iter()
            .enumerate()
            .map(|(idx, seg)| SubtitleSegment {
                id: idx + 1,  // Force sequential 1-based numbering
                start_time: seg.start,
                end_time: seg.end,
                text: seg.text.trim().to_string(),
            })
            .collect();

        Ok(segments)
    }

    /// Compute SHA256 hash of file for caching (streaming to avoid memory exhaustion)
    fn compute_file_hash(&self, path: &Path) -> Result<String, SubtitleError> {
        let file = fs::File::open(path)?;
        let mut reader = BufReader::with_capacity(8192, file); // 8KB buffer
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Get cached transcription if available
    fn get_cached_transcription(
        &self,
        file_hash: &str,
        language: &str,
    ) -> Result<Option<SubtitleTrack>, SubtitleError> {
        let cache_key = format!("{}_{}.json", file_hash, language);
        let cache_path = self.cache_dir.join(cache_key);

        if cache_path.exists() {
            let json = fs::read_to_string(cache_path)?;
            let track: SubtitleTrack = serde_json::from_str(&json)?;
            Ok(Some(track))
        } else {
            Ok(None)
        }
    }

    /// Cache transcription result
    fn cache_transcription(
        &self,
        file_hash: &str,
        language: &str,
        track: &SubtitleTrack,
    ) -> Result<(), SubtitleError> {
        let cache_key = format!("{}_{}.json", file_hash, language);
        let cache_path = self.cache_dir.join(cache_key);

        let json = serde_json::to_string_pretty(track)?;
        fs::write(cache_path, json)?;

        Ok(())
    }

    /// Parse SRT format string into subtitle segments with validation
    pub fn parse_srt(srt_content: &str) -> Result<Vec<SubtitleSegment>, SubtitleError> {
        let mut segments = Vec::new();
        let blocks: Vec<&str> = srt_content.split("\n\n").collect();
        let mut last_end_time = 0.0;

        for block in blocks {
            let lines: Vec<&str> = block.lines().collect();
            if lines.len() < 3 {
                continue;
            }

            // Parse ID
            let id = lines[0].trim().parse::<usize>()
                .map_err(|_| SubtitleError::InvalidSRT("Invalid segment ID".into()))?;

            // Parse timestamps
            let times: Vec<&str> = lines[1].split(" --> ").collect();
            if times.len() != 2 {
                return Err(SubtitleError::InvalidSRT("Invalid timestamp format".into()));
            }

            let start_time = Self::parse_srt_timestamp(times[0].trim())?;
            let end_time = Self::parse_srt_timestamp(times[1].trim())?;

            // Validate timestamp ordering within segment
            if start_time >= end_time {
                return Err(SubtitleError::InvalidSRT(
                    format!("Segment {}: start time ({}) must be before end time ({})", id, start_time, end_time)
                ));
            }

            // Validate timestamp ordering across segments
            if start_time < last_end_time {
                warn!("Segment {}: start time ({}) is before previous segment's end time ({})",
                      id, start_time, last_end_time);
            }
            last_end_time = end_time;

            // Parse text (may span multiple lines)
            let text = lines[2..].join("\n");

            // Validate non-empty text
            if text.trim().is_empty() {
                warn!("Segment {}: text is empty", id);
            }

            segments.push(SubtitleSegment {
                id,
                start_time,
                end_time,
                text,
            });
        }

        Ok(segments)
    }

    /// Parse SRT timestamp (00:00:05,000)
    fn parse_srt_timestamp(timestamp: &str) -> Result<f64, SubtitleError> {
        let parts: Vec<&str> = timestamp.split(&[':', ',']).collect();
        if parts.len() != 4 {
            return Err(SubtitleError::InvalidSRT("Invalid timestamp format".into()));
        }

        let hours: f64 = parts[0].parse()
            .map_err(|_| SubtitleError::InvalidSRT("Invalid hours".into()))?;
        let minutes: f64 = parts[1].parse()
            .map_err(|_| SubtitleError::InvalidSRT("Invalid minutes".into()))?;
        let seconds: f64 = parts[2].parse()
            .map_err(|_| SubtitleError::InvalidSRT("Invalid seconds".into()))?;
        let millis: f64 = parts[3].parse()
            .map_err(|_| SubtitleError::InvalidSRT("Invalid milliseconds".into()))?;

        Ok(hours * 3600.0 + minutes * 60.0 + seconds + millis / 1000.0)
    }

    /// Generate SRT format string from subtitle segments
    pub fn generate_srt(segments: &[SubtitleSegment]) -> String {
        segments.iter()
            .map(|seg| {
                let start = Self::format_srt_timestamp(seg.start_time);
                let end = Self::format_srt_timestamp(seg.end_time);
                format!("{}\n{} --> {}\n{}\n", seg.id, start, end, seg.text)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Format timestamp for SRT (00:00:05,000)
    fn format_srt_timestamp(seconds: f64) -> String {
        let hours = (seconds / 3600.0).floor() as u32;
        let minutes = ((seconds % 3600.0) / 60.0).floor() as u32;
        let secs = (seconds % 60.0).floor() as u32;
        let millis = ((seconds % 1.0) * 1000.0).round() as u32;

        format!("{:02}:{:02}:{:02},{:03}", hours, minutes, secs, millis)
    }

    /// Export subtitle track to SRT file
    pub fn export_srt(
        track: &SubtitleTrack,
        output_path: &Path,
    ) -> Result<(), SubtitleError> {
        let srt_content = Self::generate_srt(&track.segments);
        fs::write(output_path, srt_content)?;
        info!("Exported SRT file to: {:?}", output_path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_srt_timestamp() {
        assert_eq!(
            SubtitleService::parse_srt_timestamp("00:00:05,000").unwrap(),
            5.0
        );
        assert_eq!(
            SubtitleService::parse_srt_timestamp("00:01:30,500").unwrap(),
            90.5
        );
        assert_eq!(
            SubtitleService::parse_srt_timestamp("01:00:00,000").unwrap(),
            3600.0
        );
    }

    #[test]
    fn test_format_srt_timestamp() {
        assert_eq!(
            SubtitleService::format_srt_timestamp(5.0),
            "00:00:05,000"
        );
        assert_eq!(
            SubtitleService::format_srt_timestamp(90.5),
            "00:01:30,500"
        );
        assert_eq!(
            SubtitleService::format_srt_timestamp(3600.0),
            "01:00:00,000"
        );
    }

    #[test]
    fn test_parse_srt() {
        let srt = "1\n00:00:05,000 --> 00:00:08,000\nHello world\n\n2\n00:00:08,000 --> 00:00:12,000\nThis is a test\n";
        let segments = SubtitleService::parse_srt(srt).unwrap();

        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].id, 1);
        assert_eq!(segments[0].text, "Hello world");
        assert_eq!(segments[1].id, 2);
        assert_eq!(segments[1].text, "This is a test");
    }

    #[test]
    fn test_generate_srt() {
        let segments = vec![
            SubtitleSegment {
                id: 1,
                start_time: 5.0,
                end_time: 8.0,
                text: "Hello world".to_string(),
            },
            SubtitleSegment {
                id: 2,
                start_time: 8.0,
                end_time: 12.0,
                text: "This is a test".to_string(),
            },
        ];

        let srt = SubtitleService::generate_srt(&segments);
        assert!(srt.contains("00:00:05,000 --> 00:00:08,000"));
        assert!(srt.contains("Hello world"));
        assert!(srt.contains("This is a test"));
    }
}
