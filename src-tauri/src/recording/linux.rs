// Linux screen recording implementation (stub for future development)

use super::{RecordingConfig, RecordingError, RecordingSource, RecordingState, ScreenRecorder, SourceTypeFilter};
use std::path::PathBuf;
use log::warn;

/// Linux screen recorder (not yet implemented)
///
/// Future implementation will use:
/// - GStreamer + PipeWire (Wayland)
/// - GStreamer + X11 (X.org)
pub struct LinuxRecorder {
    state: RecordingState,
}

impl LinuxRecorder {
    pub fn new() -> Self {
        Self {
            state: RecordingState::Idle,
        }
    }
}

#[async_trait::async_trait]
impl ScreenRecorder for LinuxRecorder {
    async fn list_sources(&self, _filter: SourceTypeFilter) -> Result<Vec<RecordingSource>, RecordingError> {
        warn!("Linux screen recording not yet implemented");
        Err(RecordingError::PlatformNotSupported)
    }

    async fn check_permissions(&self) -> Result<bool, RecordingError> {
        Ok(true) // Permission handling varies by display server
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

impl Default for LinuxRecorder {
    fn default() -> Self {
        Self::new()
    }
}
