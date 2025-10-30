use crate::models::{MediaFile, SubtitleSegment, SubtitleSource, SubtitleTrack, SubtitleError};
use log::{info, warn, error};
use reqwest::multipart;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::path::{Path, PathBuf};
use std::fs;
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
            error!("FFmpeg audio extraction failed: {}", String::from_utf8_lossy(&output.stderr));
            return Err(SubtitleError::NoAudioTrack);
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

        // Convert to SubtitleSegment format
        let segments = verbose_response.segments
            .into_iter()
            .map(|seg| SubtitleSegment {
                id: seg.id,
                start_time: seg.start,
                end_time: seg.end,
                text: seg.text.trim().to_string(),
            })
            .collect();

        Ok(segments)
    }

    /// Compute SHA256 hash of file for caching
    fn compute_file_hash(&self, path: &Path) -> Result<String, SubtitleError> {
        let bytes = fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
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

    /// Parse SRT format string into subtitle segments
    pub fn parse_srt(srt_content: &str) -> Result<Vec<SubtitleSegment>, SubtitleError> {
        let mut segments = Vec::new();
        let blocks: Vec<&str> = srt_content.split("\n\n").collect();

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

            // Parse text (may span multiple lines)
            let text = lines[2..].join("\n");

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
