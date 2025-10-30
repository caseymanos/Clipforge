// Module 4: Screen Recording
// Platform-independent recording API with platform-specific implementations

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

pub mod error;
pub mod state;
pub mod integration;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

// Re-export platform-specific recorder as default
#[cfg(target_os = "macos")]
pub use macos::MacOSRecorder as PlatformRecorder;

#[cfg(target_os = "windows")]
pub use windows::WindowsRecorder as PlatformRecorder;

#[cfg(target_os = "linux")]
pub use linux::LinuxRecorder as PlatformRecorder;

pub use error::RecordingError;
pub use state::RecordingState;

/// Represents a recording source (screen, window, or application)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RecordingSource {
    /// Entire screen
    Screen {
        id: String,
        name: String,
        width: u32,
        height: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        preview_path: Option<String>,
    },
    /// Individual window
    Window {
        id: String,
        name: String,
        app_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        preview_path: Option<String>,
    },
}

impl RecordingSource {
    pub fn id(&self) -> &str {
        match self {
            RecordingSource::Screen { id, .. } => id,
            RecordingSource::Window { id, .. } => id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            RecordingSource::Screen { name, .. } => name,
            RecordingSource::Window { name, .. } => name,
        }
    }
}

/// Audio input type for recording
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AudioInputType {
    None,
    Microphone,
    SystemAudio,
    Both,
}

impl Default for AudioInputType {
    fn default() -> Self {
        Self::None
    }
}

/// Recording configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingConfig {
    /// Output file path
    pub output_path: PathBuf,

    /// Frames per second (default: 30)
    #[serde(default = "default_fps")]
    pub fps: u32,

    /// Video quality (1-10, default: 7)
    #[serde(default = "default_quality")]
    pub quality: u8,

    /// Audio input type (default: None)
    #[serde(default)]
    pub audio_input: AudioInputType,

    /// Optional audio device ID for specific device selection
    #[serde(default)]
    pub audio_device_id: Option<String>,

    /// Whether to show cursor
    #[serde(default = "default_cursor")]
    pub show_cursor: bool,

    /// Optional screen region to record (x, y, width, height)
    /// If None, records the entire screen
    #[serde(default)]
    pub crop_region: Option<CropRegion>,
}

/// Screen region for cropped recording
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CropRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

fn default_fps() -> u32 { 30 }
fn default_quality() -> u8 { 7 }
fn default_cursor() -> bool { true }

impl Default for RecordingConfig {
    fn default() -> Self {
        Self {
            output_path: PathBuf::new(),
            fps: default_fps(),
            quality: default_quality(),
            audio_input: AudioInputType::default(),
            audio_device_id: None,
            show_cursor: default_cursor(),
            crop_region: None,
        }
    }
}

/// Platform-independent screen recording trait
///
/// Each platform (macOS, Windows, Linux) implements this trait
/// with platform-specific APIs.
#[async_trait::async_trait]
pub trait ScreenRecorder: Send + Sync {
    /// List available recording sources (screens and windows)
    async fn list_sources(&self) -> Result<Vec<RecordingSource>, RecordingError>;

    /// Check if we have necessary recording permissions
    async fn check_permissions(&self) -> Result<bool, RecordingError>;

    /// Request recording permissions (may show system dialog)
    async fn request_permissions(&self) -> Result<bool, RecordingError>;

    /// Start recording from the specified source
    async fn start_recording(
        &mut self,
        source: &RecordingSource,
        config: RecordingConfig,
    ) -> Result<(), RecordingError>;

    /// Stop the current recording and finalize the file
    async fn stop_recording(&mut self) -> Result<PathBuf, RecordingError>;

    /// Get the current recording state
    fn get_state(&self) -> RecordingState;

    /// Get the current recording duration in seconds
    fn get_duration(&self) -> f64;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recording_source_id() {
        let screen = RecordingSource::Screen {
            id: "screen-1".to_string(),
            name: "Main Display".to_string(),
            width: 1920,
            height: 1080,
            preview_path: None,
        };

        assert_eq!(screen.id(), "screen-1");
        assert_eq!(screen.name(), "Main Display");
    }

    #[test]
    fn test_recording_config_defaults() {
        let config = RecordingConfig::default();

        assert_eq!(config.fps, 30);
        assert_eq!(config.quality, 7);
        assert_eq!(config.audio_input, AudioInputType::None);
        assert!(config.show_cursor);
        assert!(config.crop_region.is_none());
    }
}
