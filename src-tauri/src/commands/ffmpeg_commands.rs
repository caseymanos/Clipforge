use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use crate::ffmpeg::{FFmpegService, ProgressCallback};

/// Trim a video clip to a specific time range
#[tauri::command]
pub async fn trim_video_clip(
    input: String,
    output: String,
    start_time: f64,
    duration: f64,
    app: AppHandle,
    ffmpeg: State<'_, FFmpegService>,
) -> Result<(), String> {
    log::info!("Command: trim_video_clip({}, {:.2}s, {:.2}s)", input, start_time, duration);

    let input_path = PathBuf::from(&input);
    let output_path = PathBuf::from(&output);

    // Create progress callback that emits Tauri events
    let app_handle = app.clone();
    let progress_callback: ProgressCallback = Arc::new(move |progress| {
        app_handle.emit("ffmpeg:progress", progress).ok();
    });

    ffmpeg
        .trim_video(
            &input_path,
            &output_path,
            start_time,
            duration,
            Some(progress_callback),
        )
        .await
        .map_err(|e| e.to_string())?;

    // Emit completion event
    app.emit("ffmpeg:complete", output).ok();

    Ok(())
}

/// Concatenate multiple video clips into one
#[tauri::command]
pub async fn concatenate_clips(
    inputs: Vec<String>,
    output: String,
    app: AppHandle,
    ffmpeg: State<'_, FFmpegService>,
) -> Result<(), String> {
    log::info!("Command: concatenate_clips({} files)", inputs.len());

    let input_paths: Vec<PathBuf> = inputs.iter().map(PathBuf::from).collect();
    let output_path = PathBuf::from(&output);

    let app_handle = app.clone();
    let progress_callback: ProgressCallback = Arc::new(move |progress| {
        app_handle.emit("ffmpeg:progress", progress).ok();
    });

    ffmpeg
        .concat_videos(&input_paths, &output_path, Some(progress_callback))
        .await
        .map_err(|e| e.to_string())?;

    app.emit("ffmpeg:complete", output).ok();

    Ok(())
}

/// Extract a single frame from a video
#[tauri::command]
pub async fn extract_video_frame(
    input: String,
    timestamp: f64,
    output: String,
    ffmpeg: State<'_, FFmpegService>,
) -> Result<(), String> {
    log::info!("Command: extract_video_frame({}, {:.2}s)", input, timestamp);

    let input_path = PathBuf::from(&input);
    let output_path = PathBuf::from(&output);

    ffmpeg
        .extract_frame(&input_path, timestamp, &output_path)
        .await
        .map_err(|e| e.to_string())
}

/// Apply a video filter
#[tauri::command]
pub async fn apply_video_filter(
    input: String,
    output: String,
    filter: String,
    duration: f64,
    app: AppHandle,
    ffmpeg: State<'_, FFmpegService>,
) -> Result<(), String> {
    log::info!("Command: apply_video_filter({}, filter='{}')", input, filter);

    let input_path = PathBuf::from(&input);
    let output_path = PathBuf::from(&output);

    let app_handle = app.clone();
    let progress_callback: ProgressCallback = Arc::new(move |progress| {
        app_handle.emit("ffmpeg:progress", progress).ok();
    });

    ffmpeg
        .apply_filter(&input_path, &output_path, &filter, duration, Some(progress_callback))
        .await
        .map_err(|e| e.to_string())?;

    app.emit("ffmpeg:complete", output).ok();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_signatures() {
        // Just verify that the command signatures are correct
        // Actual testing requires Tauri runtime and FFmpeg
        assert_eq!(std::any::type_name_of_val(&trim_video_clip),
                   "clipforge::commands::ffmpeg_commands::trim_video_clip");
    }
}
