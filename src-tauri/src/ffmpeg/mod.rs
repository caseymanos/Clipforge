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
        // Try bundled FFmpeg first, then fall back to system FFmpeg
        let ffmpeg_path = Self::find_ffmpeg_path()?;

        let temp_dir = std::env::temp_dir().join("clipforge_ffmpeg");
        std::fs::create_dir_all(&temp_dir)?;

        log::info!("FFmpeg service initialized: {:?}", ffmpeg_path);

        Ok(Self {
            ffmpeg_path,
            temp_dir,
        })
    }

    /// Find FFmpeg binary path (bundled or system)
    fn find_ffmpeg_path() -> FFmpegResult<PathBuf> {
        // First try to use bundled FFmpeg (sidecar binary)
        if let Ok(exe_path) = std::env::current_exe() {
            // Tauri bundles external binaries in MacOS/ directory on macOS
            let bundled_path = exe_path
                .parent()
                .map(|p| p.join("ffmpeg"));

            if let Some(path) = bundled_path {
                if path.exists() {
                    log::info!("Using bundled FFmpeg: {:?}", path);
                    return Ok(path);
                }
            }
        }

        // Fall back to system FFmpeg in PATH
        log::info!("Bundled FFmpeg not found, trying system FFmpeg");
        which::which("ffmpeg")
            .map_err(|_| FFmpegError::ExecutableNotFound)
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
            "-af", "afade=t=in:st=0:d=0.01,afade=t=out:d=0.01,aresample=async=1:first_pts=0", // Audio filter: micro-fades to prevent boundary clicks, async resampling
            "-ar", "48000",                   // Standardize to 48kHz
            "-c:a", "aac",                    // Audio codec
            "-b:a", "256k",                   // Audio bitrate (match recording quality)
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

    /// Composite webcam overlay onto screen recording
    ///
    /// Takes two separate video files (screen and webcam) and composites them
    /// using FFmpeg overlay filters based on the provided configuration
    pub async fn composite_webcam(
        &self,
        screen_path: &Path,
        webcam_path: &Path,
        output: &Path,
        config: &crate::recording::WebcamOverlayConfig,
        progress_callback: Option<ProgressCallback>,
    ) -> FFmpegResult<()> {
        use crate::recording::WebcamPosition;

        // Validate inputs
        if !screen_path.exists() {
            return Err(FFmpegError::InputNotFound(screen_path.to_path_buf()));
        }
        if !webcam_path.exists() {
            return Err(FFmpegError::InputNotFound(webcam_path.to_path_buf()));
        }

        log::info!("Compositing webcam overlay: {:?} + {:?}", screen_path, webcam_path);

        // Build filter_complex string based on shape and position
        let filter_complex = Self::build_overlay_filter(config);

        // Calculate overlay position based on config
        let overlay_x = match config.position {
            WebcamPosition::TopLeft | WebcamPosition::BottomLeft => config.margin.to_string(),
            WebcamPosition::TopRight | WebcamPosition::BottomRight => {
                format!("W-w-{}", config.margin)
            }
        };

        let overlay_y = match config.position {
            WebcamPosition::TopLeft | WebcamPosition::TopRight => config.margin.to_string(),
            WebcamPosition::BottomLeft | WebcamPosition::BottomRight => {
                format!("H-h-{}", config.margin)
            }
        };

        // Use temporary output file
        let temp_output = self.temp_dir.join(format!("composite_{}.mp4", Uuid::new_v4()));
        log::info!("Temp output path: {:?}", temp_output);

        // Ensure temp directory exists
        if let Err(e) = std::fs::create_dir_all(&self.temp_dir) {
            log::error!("Failed to create temp directory {:?}: {}", self.temp_dir, e);
            return Err(FFmpegError::IoError(e));
        }

        let screen_str = screen_path.to_str()
            .ok_or_else(|| FFmpegError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path contains invalid UTF-8"
            )))?;

        let webcam_str = webcam_path.to_str()
            .ok_or_else(|| FFmpegError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path contains invalid UTF-8"
            )))?;

        let temp_output_str = temp_output.to_str()
            .ok_or_else(|| FFmpegError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path contains invalid UTF-8"
            )))?;

        // Build complete filter with overlay position
        let complete_filter = format!(
            "{}[0:v][webcam]overlay={}:{}[out]",
            filter_complex, overlay_x, overlay_y
        );

        log::debug!("Using filter_complex: {}", complete_filter);

        let mut cmd = Command::new(&self.ffmpeg_path);
        cmd.args([
            "-i", screen_str,                   // [0:v] Screen recording
            "-i", webcam_str,                   // [1:v] Webcam recording
            "-filter_complex", &complete_filter,
            "-map", "[out]",                    // Map composited video
            "-map", "0:a:0",                    // Map first audio stream from screen recording
            "-c:v", "libx264",                  // Video codec
            "-preset", "fast",                  // Encoding speed
            "-crf", "23",                       // Quality
            "-af", "aresample=async=1:first_pts=0", // Audio filter: async resampling to fix timestamp drift
            "-ar", "48000",                     // Standardize to 48kHz
            "-c:a", "aac",                      // Re-encode audio to AAC for browser compatibility
            "-aac_coder", "twoloop",            // Best AAC encoder method
            "-b:a", "256k",                     // Higher bitrate for native AAC encoder quality
            "-ac", "2",                         // Stereo audio
            "-y",                               // Overwrite output
            temp_output_str,
        ]);

        log::info!("Executing FFmpeg composite command: {:?}", cmd);

        // Estimate duration from screen recording for progress
        // For simplicity, assume 10 seconds; can be improved with ffprobe
        let estimated_duration = 10.0;

        self.execute_with_progress(cmd, estimated_duration, progress_callback).await?;

        // Verify temp file was created and has content
        if !temp_output.exists() {
            return Err(FFmpegError::CommandFailed(
                "FFmpeg composite succeeded but temp file does not exist".into()
            ));
        }

        let temp_size = std::fs::metadata(&temp_output)?.len();
        if temp_size == 0 {
            return Err(FFmpegError::CommandFailed(
                "FFmpeg composite created empty file".into()
            ));
        }

        log::info!("Temp composite file created: {:?} ({} bytes)", temp_output, temp_size);

        // Move temp file to final location
        log::info!("Moving temp file to final location: {:?}", output);
        if let Err(e) = std::fs::rename(&temp_output, output) {
            log::error!("Failed to move composite file: {}", e);
            return Err(FFmpegError::IoError(e));
        }

        // Verify final output exists
        if !output.exists() {
            return Err(FFmpegError::CommandFailed(
                "Composite file was not created at output location".into()
            ));
        }

        log::info!("Composite file created successfully: {:?}", output);

        // Clean up source files after successful composite
        log::info!("Cleaning up source recordings after composite");
        if let Err(e) = std::fs::remove_file(screen_path) {
            log::warn!("Failed to delete source screen recording: {}", e);
        } else {
            log::info!("Deleted source screen recording: {:?}", screen_path);
        }

        if let Err(e) = std::fs::remove_file(webcam_path) {
            log::warn!("Failed to delete source webcam recording: {}", e);
        } else {
            log::info!("Deleted source webcam recording: {:?}", webcam_path);
        }

        log::info!("Composite complete: {:?}", output);
        Ok(())
    }

    /// Build FFmpeg filter string for webcam overlay
    ///
    /// Creates appropriate filter based on shape (square or circle)
    fn build_overlay_filter(config: &crate::recording::WebcamOverlayConfig) -> String {
        use crate::recording::WebcamShape;
        let size = config.size;
        let radius = size / 2;

        match config.shape {
            WebcamShape::Square => {
                // Scale to cover target size while preserving aspect ratio, then crop from center
                format!(
                    "[1:v]scale={}:{}:force_original_aspect_ratio=increase,crop={}:{}[webcam];",
                    size, size, size, size
                )
            }
            WebcamShape::Circle => {
                // Scale to cover, crop to square, then apply circular alpha mask
                // Using geq with proper escaping: no quotes around the expression
                // The alpha channel formula: if distance from center <= radius, then opaque (255), else transparent (0)
                // Preserve all color channels (lum, cb, cr) to prevent blue/purple/green tint
                format!(
                    "[1:v]scale={}:{}:force_original_aspect_ratio=increase,crop={}:{},format=yuva420p,geq=lum=lum(X\\,Y):cb=cb(X\\,Y):cr=cr(X\\,Y):a=if(lte(sqrt(pow(X-{}\\,2)+pow(Y-{}\\,2))\\,{})\\,255\\,0)[webcam];",
                    size, size, size, size, radius, radius, radius
                )
            }
        }
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
            "geq", "format",  // Added for circular webcam mask
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
        // Log the full command for debugging
        log::debug!("FFmpeg command: {:?}", cmd);

        cmd.stdout(Stdio::null())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn()
            .map_err(|e| {
                log::error!("Failed to spawn FFmpeg process: {}", e);
                FFmpegError::CommandFailed(e.to_string())
            })?;

        let stderr = child.stderr.take()
            .ok_or_else(|| FFmpegError::CommandFailed("Failed to capture stderr".to_string()))?;

        let parser = ProgressParser::new(total_duration);
        let mut reader = BufReader::new(stderr).lines();

        // Collect stderr for debugging in case of failure
        let mut stderr_lines = Vec::new();

        // Read stderr line by line and parse progress
        while let Ok(Some(line)) = reader.next_line().await {
            stderr_lines.push(line.clone());
            if let Some(progress) = parser.parse_line(&line) {
                if let Some(callback) = &progress_callback {
                    callback(progress);
                }
            }
        }

        let status = child.wait().await
            .map_err(|e| {
                log::error!("Failed to wait for FFmpeg process: {}", e);
                FFmpegError::CommandFailed(e.to_string())
            })?;

        if !status.success() {
            // Log last 10 lines of stderr for debugging
            let stderr_tail = stderr_lines.iter().rev().take(10).rev()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join("\n");

            log::error!("FFmpeg failed with exit code {:?}. Last stderr lines:\n{}", status.code(), stderr_tail);
            eprintln!("[FFMPEG ERROR] Exit code {:?}. Last stderr:\n{}", status.code(), stderr_tail);

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
