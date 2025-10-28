use std::sync::Arc;
use tokio::sync::Mutex;
use regex::Regex;

/// Progress callback type
pub type ProgressCallback = Arc<dyn Fn(f64) + Send + Sync>;

/// Parse FFmpeg progress from stderr output
pub struct ProgressParser {
    total_duration: f64,
    time_regex: Regex,
}

impl ProgressParser {
    pub fn new(total_duration: f64) -> Self {
        Self {
            total_duration,
            // Matches "time=HH:MM:SS.MS" or "time=SS.MS"
            time_regex: Regex::new(r"time=(\d+):(\d+):(\d+\.\d+)|time=(\d+\.\d+)")
                .expect("Invalid regex"),
        }
    }

    /// Parse a line of FFmpeg stderr output and return progress (0.0 to 1.0)
    pub fn parse_line(&self, line: &str) -> Option<f64> {
        if let Some(captures) = self.time_regex.captures(line) {
            let seconds = if let Some(simple) = captures.get(4) {
                // Simple format: time=123.45
                simple.as_str().parse::<f64>().ok()?
            } else {
                // Full format: time=01:23:45.67
                let hours = captures.get(1)?.as_str().parse::<f64>().ok()?;
                let minutes = captures.get(2)?.as_str().parse::<f64>().ok()?;
                let secs = captures.get(3)?.as_str().parse::<f64>().ok()?;
                hours * 3600.0 + minutes * 60.0 + secs
            };

            if self.total_duration > 0.0 {
                Some((seconds / self.total_duration).min(1.0))
            } else {
                Some(0.0)
            }
        } else {
            None
        }
    }
}

/// Wrapper for progress tracking with cancellation support
#[allow(dead_code)]
pub struct ProgressTracker {
    cancelled: Arc<Mutex<bool>>,
    callback: Option<ProgressCallback>,
}

#[allow(dead_code)]
impl ProgressTracker {
    pub fn new(callback: Option<ProgressCallback>) -> Self {
        Self {
            cancelled: Arc::new(Mutex::new(false)),
            callback,
        }
    }

    /// Check if operation was cancelled
    pub async fn is_cancelled(&self) -> bool {
        *self.cancelled.lock().await
    }

    /// Cancel the operation
    pub async fn cancel(&self) {
        *self.cancelled.lock().await = true;
    }

    /// Report progress (0.0 to 1.0)
    pub fn report_progress(&self, progress: f64) {
        if let Some(callback) = &self.callback {
            callback(progress);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_time() {
        let parser = ProgressParser::new(100.0);

        // Simple format
        let progress = parser.parse_line("frame=  123 fps= 45 time=50.25 bitrate=1234.5kbits/s");
        assert_eq!(progress, Some(0.5025));
    }

    #[test]
    fn test_parse_full_time() {
        let parser = ProgressParser::new(3600.0); // 1 hour total

        // Full format: 30 minutes = 50% progress
        let progress = parser.parse_line("frame=  123 fps= 45 time=00:30:00.00 bitrate=1234.5kbits/s");
        assert_eq!(progress, Some(0.5));
    }

    #[test]
    fn test_parse_invalid_line() {
        let parser = ProgressParser::new(100.0);
        let progress = parser.parse_line("Some other output");
        assert_eq!(progress, None);
    }

    #[test]
    fn test_progress_clamp() {
        let parser = ProgressParser::new(100.0);
        // Time exceeds duration - should clamp to 1.0
        let progress = parser.parse_line("time=150.0");
        assert_eq!(progress, Some(1.0));
    }
}
