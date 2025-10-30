use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

/// Type of media file
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Video,  // Video file (with or without audio)
    Audio,  // Audio-only file (.mp3, .wav, .aac, etc.)
    Image,  // Image file (.jpg, .png, etc.) - for future use
}

/// Represents a media file in the library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaFile {
    pub id: String,              // UUID
    pub path: PathBuf,
    pub filename: String,
    pub media_type: MediaType,   // Type of media
    pub duration: f64,           // seconds
    pub resolution: Option<Resolution>,  // Optional for audio files
    pub codec: MediaCodec,       // Video and/or audio codec info
    pub file_size: u64,          // bytes
    pub thumbnail_path: Option<PathBuf>,
    pub hash: String,            // SHA-256 for deduplication
    pub imported_at: DateTime<Utc>,
    pub proxy_path: Option<PathBuf>,    // H.264 proxy for smooth editing
    pub has_proxy: bool,                // Whether proxy exists
    pub proxy_status: ProxyStatus,      // Proxy generation status
}

/// Status of proxy file generation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProxyStatus {
    None,       // No proxy requested
    Generating, // Proxy being generated
    Ready,      // Proxy ready to use
    Failed,     // Proxy generation failed
}

/// Video resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

/// Media codec information (video and/or audio)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaCodec {
    pub video: Option<String>,   // e.g., "h264", "hevc" (None for audio-only)
    pub audio: Option<String>,   // e.g., "aac", "mp3" (None for video-only without audio)
}

/// Legacy type alias for backwards compatibility
pub type VideoCodec = MediaCodec;

/// Complete metadata extracted from a media file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub media_type: MediaType,          // Type of media
    pub duration: f64,
    pub resolution: Option<Resolution>, // None for audio files
    pub codec: MediaCodec,
    pub bitrate: u64,
    pub framerate: Option<f64>,         // None for audio files
    pub has_audio: bool,
    pub has_video: bool,
}

/// Custom error types for file operations
#[derive(Debug, thiserror::Error)]
pub enum FileError {
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("Metadata extraction failed: {0}")]
    MetadataError(String),

    #[error("Thumbnail generation failed")]
    ThumbnailError,

    #[error("Invalid file format")]
    InvalidFormat,
}

/// Custom error types for metadata extraction
#[derive(Debug, thiserror::Error)]
pub enum MetadataError {
    #[error("FFprobe execution failed")]
    FFprobeError,

    #[error("No video stream found")]
    NoVideoStream,

    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Custom error types for thumbnail generation
#[derive(Debug, thiserror::Error)]
pub enum ThumbnailError {
    #[error("Thumbnail generation failed")]
    GenerationFailed,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

// ============================================================================
// Module 5: Timeline Engine Data Structures
// ============================================================================

/// Represents a complete timeline project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub id: String,
    pub name: String,
    pub framerate: f64,
    pub resolution: Resolution,
    pub tracks: Vec<Track>,
    pub duration: f64,  // Total duration in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle_track: Option<SubtitleTrack>,  // AI-generated or imported subtitles
    #[serde(default)]
    pub subtitle_enabled: bool,  // Global toggle for preview and export
}

/// A track in the timeline (Video, Audio, or Overlay)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub track_type: TrackType,
    pub clips: Vec<Clip>,
    pub muted: bool,
    pub locked: bool,
}

/// Type of track
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackType {
    Video,
    Audio,
    Overlay,
}

/// A clip on the timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    pub id: String,
    pub media_file_id: String,
    pub name: Option<String>,  // Display name (typically filename)
    pub track_position: f64,  // Position on timeline in seconds
    pub duration: f64,  // Duration in seconds (can differ from source if trimmed)
    pub trim_start: f64,  // Trim from start of source in seconds
    pub trim_end: f64,  // Trim from end of source in seconds
    pub effects: Vec<Effect>,
    pub volume: f32,  // 0.0 to 1.0 (or higher for amplification)
    pub speed: f32,  // Playback speed multiplier (0.5 = half speed, 2.0 = double speed)
}

/// Video/audio effect applied to a clip
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Effect {
    pub id: String,
    pub effect_type: EffectType,
    pub enabled: bool,
}

/// Types of effects that can be applied
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EffectType {
    // Video effects
    Brightness { value: f32 },  // -1.0 to 1.0
    Contrast { value: f32 },  // -1.0 to 1.0
    Saturation { value: f32 },  // -1.0 to 1.0
    Blur { radius: f32 },  // 0.0 to 100.0
    Sharpen { amount: f32 },  // 0.0 to 1.0

    // Audio effects
    Normalize,
    FadeIn { duration: f64 },  // seconds
    FadeOut { duration: f64 },  // seconds
}

/// Custom error types for timeline operations
#[derive(Debug, thiserror::Error)]
pub enum TimelineError {
    #[error("Timeline not found")]
    TimelineNotFound,

    #[error("Track not found: {0}")]
    TrackNotFound(String),

    #[error("Clip not found: {0}")]
    ClipNotFound(String),

    #[error("Invalid clip position: {0}")]
    InvalidPosition(String),

    #[error("Clip overlap detected")]
    ClipOverlap,

    #[error("Invalid trim values")]
    InvalidTrim,

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Overlap error: {0}")]
    OverlapError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

// ============================================================================
// Module 6: Export & Rendering Data Structures
// ============================================================================

/// Export settings for timeline rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSettings {
    pub video_codec: String,      // e.g., "libx264", "libx265"
    pub audio_codec: String,      // e.g., "aac", "mp3"
    pub video_bitrate: u32,       // in kbps
    pub audio_bitrate: u32,       // in kbps
    pub framerate: f64,           // frames per second
    pub resolution: Resolution,   // output resolution
    pub format: String,           // e.g., "mp4", "mov", "webm"
}

impl ExportSettings {
    /// YouTube 1080p preset
    pub fn youtube_1080p() -> Self {
        Self {
            video_codec: "libx264".to_string(),
            audio_codec: "aac".to_string(),
            video_bitrate: 8000,
            audio_bitrate: 192,
            framerate: 30.0,
            resolution: Resolution { width: 1920, height: 1080 },
            format: "mp4".to_string(),
        }
    }

    /// Instagram post preset (1:1 square)
    pub fn instagram_post() -> Self {
        Self {
            video_codec: "libx264".to_string(),
            audio_codec: "aac".to_string(),
            video_bitrate: 5000,
            audio_bitrate: 128,
            framerate: 30.0,
            resolution: Resolution { width: 1080, height: 1080 },
            format: "mp4".to_string(),
        }
    }

    /// Twitter video preset
    pub fn twitter_video() -> Self {
        Self {
            video_codec: "libx264".to_string(),
            audio_codec: "aac".to_string(),
            video_bitrate: 6000,
            audio_bitrate: 128,
            framerate: 30.0,
            resolution: Resolution { width: 1280, height: 720 },
            format: "mp4".to_string(),
        }
    }

    /// Custom export settings
    pub fn custom(
        video_codec: String,
        audio_codec: String,
        video_bitrate: u32,
        audio_bitrate: u32,
        framerate: f64,
        resolution: Resolution,
    ) -> Self {
        Self {
            video_codec,
            audio_codec,
            video_bitrate,
            audio_bitrate,
            framerate,
            resolution,
            format: "mp4".to_string(),
        }
    }
}

/// Export progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportProgress {
    pub percentage: f64,           // 0.0 to 100.0
    pub current_frame: u64,        // Current frame being processed
    pub fps: f64,                  // Current encoding FPS
    pub time_remaining_secs: u64,  // Estimated seconds remaining
}

/// Custom error types for export operations
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("FFmpeg error: {0}")]
    FFmpegError(String),

    #[error("Timeline validation error: {0}")]
    ValidationError(String),

    #[error("Output file error: {0}")]
    OutputError(String),

    #[error("Export cancelled by user")]
    Cancelled,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

// ============================================================================
// AI Subtitle Generation Data Structures
// ============================================================================

/// A single subtitle segment with timing and text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleSegment {
    pub id: usize,
    pub start_time: f64,      // seconds from timeline start
    pub end_time: f64,        // seconds from timeline start
    pub text: String,
}

/// Source of subtitle generation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SubtitleSource {
    Transcribed {
        media_file_id: String,
        provider: String,  // "openai-whisper"
    },
    Imported {
        file_path: PathBuf
    },
    Manual,
}

/// Subtitle track for a timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleTrack {
    pub segments: Vec<SubtitleSegment>,
    pub language: String,      // ISO 639-1 code (e.g., "en", "es")
    pub source: SubtitleSource,
}

/// Custom error types for subtitle operations
#[derive(Debug, thiserror::Error)]
pub enum SubtitleError {
    #[error("API error: {0}")]
    ApiError(String),

    #[error("No audio track found in media file")]
    NoAudioTrack,

    #[error("Invalid SRT format: {0}")]
    InvalidSRT(String),

    #[error("Transcription cache error: {0}")]
    CacheError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("HTTP request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}
