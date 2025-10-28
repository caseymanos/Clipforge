// Recording state management

use serde::{Deserialize, Serialize};

/// Current state of the recording system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecordingState {
    /// No recording in progress
    Idle,

    /// Recording is active
    Recording,

    /// Recording is paused (future feature)
    #[allow(dead_code)]
    Paused,

    /// Recording is being finalized
    Finalizing,

    /// An error occurred
    Error,
}

impl Default for RecordingState {
    fn default() -> Self {
        RecordingState::Idle
    }
}

impl RecordingState {
    pub fn is_recording(&self) -> bool {
        matches!(self, RecordingState::Recording)
    }

    pub fn is_idle(&self) -> bool {
        matches!(self, RecordingState::Idle)
    }

    pub fn can_start(&self) -> bool {
        matches!(self, RecordingState::Idle)
    }

    pub fn can_stop(&self) -> bool {
        matches!(self, RecordingState::Recording)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recording_state_default() {
        let state = RecordingState::default();
        assert_eq!(state, RecordingState::Idle);
    }

    #[test]
    fn test_recording_state_checks() {
        let idle = RecordingState::Idle;
        assert!(idle.is_idle());
        assert!(!idle.is_recording());
        assert!(idle.can_start());
        assert!(!idle.can_stop());

        let recording = RecordingState::Recording;
        assert!(!recording.is_idle());
        assert!(recording.is_recording());
        assert!(!recording.can_start());
        assert!(recording.can_stop());
    }
}
