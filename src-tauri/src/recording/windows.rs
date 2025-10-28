// Windows screen recording implementation (stub for future development)

use super::{RecordingConfig, RecordingError, RecordingSource, RecordingState, ScreenRecorder};
use std::path::PathBuf;
use log::warn;

/// Windows screen recorder (not yet implemented)
///
/// Future implementation will use:
/// - Graphics.Capture API (Windows 10+)
/// - windows-capture crate
pub struct WindowsRecorder {
    state: RecordingState,
}

impl WindowsRecorder {
    pub fn new() -> Self {
        Self {
            state: RecordingState::Idle,
        }
    }
}

#[async_trait::async_trait]
impl ScreenRecorder for WindowsRecorder {
    async fn list_sources(&self) -> Result<Vec<RecordingSource>, RecordingError> {
        warn!("Windows screen recording not yet implemented");
        Err(RecordingError::PlatformNotSupported)
    }

    async fn check_permissions(&self) -> Result<bool, RecordingError> {
        Ok(true) // Windows doesn't require explicit permission for screen capture
    }

    async fn request_permissions(&self) -> Result<bool, RecordingError> {
        Ok(true)
    }

    async fn start_recording(
        &mut self,
        _source: &RecordingSource,
        _config: RecordingConfig,
    ) -> Result<(), RecordingError> {
        Err(RecordingError::PlatformNotSupported)
    }

    async fn stop_recording(&mut self) -> Result<PathBuf, RecordingError> {
        Err(RecordingError::PlatformNotSupported)
    }

    fn get_state(&self) -> RecordingState {
        self.state
    }

    fn get_duration(&self) -> f64 {
        0.0
    }
}

impl Default for WindowsRecorder {
    fn default() -> Self {
        Self::new()
    }
}
