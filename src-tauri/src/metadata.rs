use std::path::Path;
use std::process::Command;
use serde_json::Value;
use crate::models::{FileMetadata, MediaType, Resolution, MediaCodec, MetadataError};

/// Extract video metadata using FFprobe
pub async fn extract_metadata(path: &Path) -> Result<FileMetadata, MetadataError> {
    let path_str = path.to_str()
        .ok_or_else(|| MetadataError::IoError(
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path contains invalid UTF-8 characters"
            )
        ))?;

    let output = Command::new("ffprobe")
        .args([
            "-v", "quiet",
            "-print_format", "json",
            "-show_streams",
            "-show_format",
            path_str,
        ])
        .output()?;

    if !output.status.success() {
        log::error!("FFprobe failed for file: {:?}", path);
        return Err(MetadataError::FFprobeError);
    }

    let json: Value = serde_json::from_slice(&output.stdout)?;

    // Extract stream info
    let streams = json["streams"]
        .as_array()
        .ok_or(MetadataError::FFprobeError)?;

    let video_stream = streams
        .iter()
        .find(|s| s["codec_type"] == "video");

    let audio_stream = streams
        .iter()
        .find(|s| s["codec_type"] == "audio");

    let format = &json["format"];

    // Determine media type based on available streams
    let has_video = video_stream.is_some();
    let has_audio = audio_stream.is_some();

    let media_type = if has_video && has_audio {
        MediaType::Video
    } else if has_audio {
        MediaType::Audio
    } else if has_video {
        MediaType::Video  // Video-only (no audio track)
    } else {
        return Err(MetadataError::FFprobeError);  // No valid streams
    };

    // Parse duration
    let duration = format["duration"]
        .as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    // Parse resolution (only for video files)
    let resolution = video_stream.map(|vs| {
        let width = vs["width"].as_u64().unwrap_or(0) as u32;
        let height = vs["height"].as_u64().unwrap_or(0) as u32;
        Resolution { width, height }
    });

    // Parse codecs
    let video_codec = video_stream
        .and_then(|s| s["codec_name"].as_str())
        .map(|s| s.to_string());

    let audio_codec = audio_stream
        .and_then(|s| s["codec_name"].as_str())
        .map(|s| s.to_string());

    // Parse bitrate
    let bitrate = format["bit_rate"]
        .as_str()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    // Parse framerate (only for video files)
    let framerate = video_stream
        .and_then(|vs| vs["r_frame_rate"].as_str())
        .map(parse_framerate);

    Ok(FileMetadata {
        media_type,
        duration,
        resolution,
        codec: MediaCodec {
            video: video_codec,
            audio: audio_codec,
        },
        bitrate,
        framerate,
        has_audio,
        has_video,
    })
}

/// Parse framerate string like "30/1" or "24000/1001"
fn parse_framerate(fps_str: &str) -> f64 {
    let parts: Vec<&str> = fps_str.split('/').collect();
    if parts.len() == 2 {
        let num: f64 = parts[0].parse().unwrap_or(30.0);
        let den: f64 = parts[1].parse().unwrap_or(1.0);
        if den != 0.0 {
            return num / den;
        }
    }
    30.0 // Default fallback
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_framerate() {
        assert_eq!(parse_framerate("30/1"), 30.0);
        assert_eq!(parse_framerate("60/1"), 60.0);
        // 23.976 fps (common for film)
        assert!((parse_framerate("24000/1001") - 23.976).abs() < 0.001);
    }
}
