// Recording error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RecordingError {
    #[error("Recording permission denied")]
    PermissionDenied,

    #[error("Recording source not found: {0}")]
    SourceNotFound(String),

    #[error("Recording already in progress")]
    AlreadyRecording,

    #[error("No recording in progress")]
    NotRecording,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Recording failed: {0}")]
    RecordingFailed(String),

    #[error("Platform not supported")]
    PlatformNotSupported,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("System error: {0}")]
    SystemError(String),
}

pub type RecordingResult<T> = Result<T, RecordingError>;
