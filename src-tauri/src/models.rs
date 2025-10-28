use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

/// Represents a media file in the library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaFile {
    pub id: String,              // UUID
    pub path: PathBuf,
    pub filename: String,
    pub duration: f64,           // seconds
    pub resolution: Resolution,
    pub codec: VideoCodec,
    pub file_size: u64,          // bytes
    pub thumbnail_path: Option<PathBuf>,
    pub hash: String,            // SHA-256 for deduplication
    pub imported_at: DateTime<Utc>,
}

/// Video resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

/// Video and audio codec information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoCodec {
    pub video: String,   // e.g., "h264", "hevc"
    pub audio: String,   // e.g., "aac", "mp3"
}

/// Complete metadata extracted from a video file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub duration: f64,
    pub resolution: Resolution,
    pub codec: VideoCodec,
    pub bitrate: u64,
    pub framerate: f64,
    pub has_audio: bool,
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
