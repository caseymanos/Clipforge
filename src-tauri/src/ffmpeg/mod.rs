mod error;
mod progress;

pub use error::{FFmpegError, FFmpegResult};
pub use progress::{ProgressCallback, ProgressParser};
#[allow(unused_imports)]
pub use progress::ProgressTracker;

use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use uuid::Uuid;

/// FFmpeg service for video processing operations
pub struct FFmpegService {
    ffmpeg_path: PathBuf,
    temp_dir: PathBuf,
}

impl FFmpegService {
    /// Create a new FFmpeg service
    pub fn new() -> FFmpegResult<Self> {
        // Verify ffmpeg is available
        let ffmpeg_path = which::which("ffmpeg")
            .map_err(|_| FFmpegError::ExecutableNotFound)?;

        let temp_dir = std::env::temp_dir().join("clipforge_ffmpeg");
        std::fs::create_dir_all(&temp_dir)?;

        log::info!("FFmpeg service initialized: {:?}", ffmpeg_path);

        Ok(Self {
            ffmpeg_path,
            temp_dir,
        })
    }

    /// Trim a video to a specific time range
    ///
    /// Uses frame-accurate seeking with re-encoding for precision
    pub async fn trim_video(
        &self,
        input: &Path,
        output: &Path,
        start_time: f64,
        duration: f64,
        progress_callback: Option<ProgressCallback>,
    ) -> FFmpegResult<()> {
        // Validate inputs
        if !input.exists() {
            return Err(FFmpegError::InputNotFound(input.to_path_buf()));
        }
        if start_time < 0.0 || duration <= 0.0 {
            return Err(FFmpegError::InvalidTimeRange { start: start_time, duration });
        }

        log::info!(
            "Trimming video: {:?} from {:.2}s for {:.2}s",
            input,
            start_time,
            duration
        );

        // Use temporary output file for atomic operation
        let temp_output = self.temp_dir.join(format!("trim_{}.mp4", Uuid::new_v4()));

        let input_str = input.to_str()
            .ok_or_else(|| FFmpegError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path contains invalid UTF-8"
            )))?;

        let temp_output_str = temp_output.to_str()
            .ok_or_else(|| FFmpegError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path contains invalid UTF-8"
            )))?;

        // Build FFmpeg command
        let mut cmd = Command::new(&self.ffmpeg_path);
        cmd.args([
            "-ss", &start_time.to_string(),  // Seek to start
            "-i", input_str,                  // Input file
            "-t", &duration.to_string(),      // Duration
            "-c:v", "libx264",                // Video codec
            "-crf", "23",                     // Quality (0-51, lower is better)
            "-preset", "medium",              // Encoding speed
            "-c:a", "aac",                    // Audio codec
            "-b:a", "128k",                   // Audio bitrate
            "-y",                             // Overwrite output
            temp_output_str,
        ]);

        // Execute with progress tracking
        self.execute_with_progress(cmd, duration, progress_callback).await?;

        // Move temp file to final location
        std::fs::rename(&temp_output, output)?;

        log::info!("Trim complete: {:?}", output);
        Ok(())
    }

    /// Concatenate multiple videos into one
    ///
    /// Uses concat demuxer for fast operation without re-encoding
    pub async fn concat_videos(
        &self,
        inputs: &[PathBuf],
        output: &Path,
        progress_callback: Option<ProgressCallback>,
    ) -> FFmpegResult<()> {
        if inputs.is_empty() {
            return Err(FFmpegError::NoInputFiles);
        }

        // Validate all inputs exist
        for input in inputs {
            if !input.exists() {
                return Err(FFmpegError::InputNotFound(input.clone()));
            }
        }

        log::info!("Concatenating {} videos", inputs.len());

        // Create concat file list
        let filelist_path = self.temp_dir.join(format!("concat_{}.txt", Uuid::new_v4()));
        let mut filelist_content = String::new();
        for input in inputs {
            let input_str = input.to_str()
                .ok_or_else(|| FFmpegError::IoError(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Path contains invalid UTF-8"
                )))?;

            // Validate filename doesn't contain dangerous characters for concat format
            if input_str.contains('\n') || input_str.contains('\'') || input_str.contains('\r') {
                return Err(FFmpegError::IoError(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Path contains dangerous characters (newline or quote)"
                )));
            }

            filelist_content.push_str(&format!("file '{}'\n", input_str));
        }
        std::fs::write(&filelist_path, filelist_content)?;

        // Use temporary output file
        let temp_output = self.temp_dir.join(format!("concat_{}.mp4", Uuid::new_v4()));

        let filelist_str = filelist_path.to_str()
            .ok_or_else(|| FFmpegError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path contains invalid UTF-8"
            )))?;

        let temp_output_str = temp_output.to_str()
            .ok_or_else(|| FFmpegError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path contains invalid UTF-8"
            )))?;

        // Build FFmpeg command
        let mut cmd = Command::new(&self.ffmpeg_path);
        cmd.args([
            "-f", "concat",          // Use concat demuxer
            "-safe", "0",            // Allow absolute paths
            "-i", filelist_str,      // Input filelist
            "-c", "copy",            // Copy streams (no re-encode)
            "-y",                    // Overwrite output
            temp_output_str,
        ]);

        // Calculate approximate total duration for progress
        // For simplicity, we'll estimate based on file count
        let estimated_duration = inputs.len() as f64 * 10.0; // Rough estimate

        self.execute_with_progress(cmd, estimated_duration, progress_callback).await?;

        // Move temp file to final location
        std::fs::rename(&temp_output, output)?;

        // Cleanup filelist
        std::fs::remove_file(&filelist_path).ok();

        log::info!("Concatenation complete: {:?}", output);
        Ok(())
    }

    /// Extract a single frame from a video at a specific timestamp
    pub async fn extract_frame(
        &self,
        input: &Path,
        timestamp: f64,
        output: &Path,
    ) -> FFmpegResult<()> {
        if !input.exists() {
            return Err(FFmpegError::InputNotFound(input.to_path_buf()));
        }

        log::debug!("Extracting frame at {:.2}s from {:?}", timestamp, input);

        let input_str = input.to_str()
            .ok_or_else(|| FFmpegError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path contains invalid UTF-8"
            )))?;

        let output_str = output.to_str()
            .ok_or_else(|| FFmpegError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path contains invalid UTF-8"
            )))?;

        let mut cmd = Command::new(&self.ffmpeg_path);
        cmd.args([
            "-ss", &timestamp.to_string(),
            "-i", input_str,
            "-vframes", "1",              // Extract 1 frame
            "-q:v", "2",                  // High quality
            "-y",
            output_str,
        ]);

        // Frame extraction is fast, no progress needed
        self.execute_command(cmd).await?;

        log::debug!("Frame extracted: {:?}", output);
        Ok(())
    }

    /// Apply a video filter
    pub async fn apply_filter(
        &self,
        input: &Path,
        output: &Path,
        filter: &str,
        duration: f64,
        progress_callback: Option<ProgressCallback>,
    ) -> FFmpegResult<()> {
        if !input.exists() {
            return Err(FFmpegError::InputNotFound(input.to_path_buf()));
        }

        // Validate filter to prevent command injection
        if filter.is_empty() {
            return Err(FFmpegError::InvalidFilter("Filter cannot be empty".to_string()));
        }

        Self::validate_filter(filter)?;

        log::info!("Applying filter '{}' to {:?}", filter, input);

        let temp_output = self.temp_dir.join(format!("filter_{}.mp4", Uuid::new_v4()));

        let input_str = input.to_str()
            .ok_or_else(|| FFmpegError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path contains invalid UTF-8"
            )))?;

        let temp_output_str = temp_output.to_str()
            .ok_or_else(|| FFmpegError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path contains invalid UTF-8"
            )))?;

        let mut cmd = Command::new(&self.ffmpeg_path);
        cmd.args([
            "-i", input_str,
            "-vf", filter,
            "-c:a", "copy",          // Copy audio stream
            "-y",
            temp_output_str,
        ]);

        self.execute_with_progress(cmd, duration, progress_callback).await?;

        std::fs::rename(&temp_output, output)?;

        log::info!("Filter applied: {:?}", output);
        Ok(())
    }

    /// Validate filter string to prevent command injection
    ///
    /// Only allows safe filter names and common parameters
    fn validate_filter(filter: &str) -> FFmpegResult<()> {
        // Whitelist of allowed filter names
        const ALLOWED_FILTERS: &[&str] = &[
            "scale", "crop", "rotate", "hflip", "vflip",
            "eq", "brightness", "contrast", "saturation",
            "fade", "fadein", "fadeout",
            "blur", "unsharp", "sharpen",
            "fps", "setpts", "trim",
            "pad", "drawtext", "overlay",
            "colorbalance", "curves", "hue",
        ];

        // Check for dangerous characters that could be used for injection
        let dangerous_chars = ['`', ';', '|', '&', '$', '>', '<', '\n', '\r'];
        if filter.chars().any(|c| dangerous_chars.contains(&c)) {
            return Err(FFmpegError::InvalidFilter(
                "Filter contains dangerous characters".to_string()
            ));
        }

        // Extract filter name (text before first '=' or ':')
        let filter_name = filter
            .split(|c| c == '=' || c == ':' || c == '(')
            .next()
            .unwrap_or("");

        // Check if filter name is in whitelist
        if !ALLOWED_FILTERS.contains(&filter_name) {
            return Err(FFmpegError::InvalidFilter(
                format!("Filter '{}' is not allowed", filter_name)
            ));
        }

        // Additional check: prevent file paths in filter (detect '/' or '\')
        if filter.contains('/') || filter.contains('\\') {
            return Err(FFmpegError::InvalidFilter(
                "Filter cannot contain file paths".to_string()
            ));
        }

        Ok(())
    }

    /// Execute FFmpeg command with progress tracking
    async fn execute_with_progress(
        &self,
        mut cmd: Command,
        total_duration: f64,
        progress_callback: Option<ProgressCallback>,
    ) -> FFmpegResult<()> {
        cmd.stdout(Stdio::null())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn()
            .map_err(|e| FFmpegError::CommandFailed(e.to_string()))?;

        let stderr = child.stderr.take()
            .ok_or_else(|| FFmpegError::CommandFailed("Failed to capture stderr".to_string()))?;

        let parser = ProgressParser::new(total_duration);
        let mut reader = BufReader::new(stderr).lines();

        // Read stderr line by line and parse progress
        while let Ok(Some(line)) = reader.next_line().await {
            if let Some(progress) = parser.parse_line(&line) {
                if let Some(callback) = &progress_callback {
                    callback(progress);
                }
            }
        }

        let status = child.wait().await
            .map_err(|e| FFmpegError::CommandFailed(e.to_string()))?;

        if !status.success() {
            return Err(FFmpegError::CommandFailed(
                format!("FFmpeg exited with code {:?}", status.code())
            ));
        }

        Ok(())
    }

    /// Execute FFmpeg command without progress tracking
    async fn execute_command(&self, mut cmd: Command) -> FFmpegResult<()> {
        cmd.stdout(Stdio::null())
            .stderr(Stdio::null());

        let status = cmd.status().await
            .map_err(|e| FFmpegError::CommandFailed(e.to_string()))?;

        if !status.success() {
            return Err(FFmpegError::CommandFailed(
                format!("FFmpeg exited with code {:?}", status.code())
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffmpeg_service_creation() {
        // This will only pass if ffmpeg is installed
        match FFmpegService::new() {
            Ok(service) => {
                assert!(service.ffmpeg_path.exists());
            }
            Err(FFmpegError::ExecutableNotFound) => {
                // Expected if ffmpeg not installed
                println!("FFmpeg not found - test skipped");
            }
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }
}
