use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FFmpegError {
    #[error("FFmpeg executable not found")]
    ExecutableNotFound,

    #[error("Input file not found: {0}")]
    InputNotFound(PathBuf),

    #[error("Invalid time range: start={start}, duration={duration}")]
    InvalidTimeRange { start: f64, duration: f64 },

    #[error("FFmpeg command failed: {0}")]
    CommandFailed(String),

    #[error("Failed to parse FFmpeg output")]
    #[allow(dead_code)]
    ParseError,

    #[error("Operation was cancelled")]
    #[allow(dead_code)]
    Cancelled,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid filter expression: {0}")]
    InvalidFilter(String),

    #[error("No input files provided")]
    NoInputFiles,
}

pub type FFmpegResult<T> = Result<T, FFmpegError>;
