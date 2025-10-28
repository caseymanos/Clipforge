# Module 3: FFmpeg Integration

**Owner:** TBD  
**Dependencies:** Module 2 (File System)  
**Phase:** 1 (Weeks 1-2)  
**Estimated Effort:** 5-6 days

## Overview

Provides video processing capabilities through FFmpeg command-line wrapper. Handles trimming, concatenation, effects, and format conversion.

## Core Operations

### 1. Trim Video
```rust
pub async fn trim_video(
    input: &Path,
    output: &Path,
    start: f64,
    duration: f64
) -> Result<(), FFmpegError>
```

### 2. Concatenate Videos
```rust
pub async fn concat_videos(
    inputs: &[PathBuf],
    output: &Path
) -> Result<(), FFmpegError>
```

### 3. Extract Audio
```rust
pub async fn extract_audio(
    input: &Path,
    output: &Path
) -> Result<(), FFmpegError>
```

### 4. Apply Effects
```rust
pub async fn apply_filter(
    input: &Path,
    output: &Path,
    filter: &str
) -> Result<(), FFmpegError>
```

## FFmpeg Command Patterns

```bash
# Frame-accurate trim
ffmpeg -ss 10.0 -i input.mp4 -t 5.0 -c:v libx264 -crf 23 output.mp4

# Fast concat (no re-encode)
ffmpeg -f concat -safe 0 -i filelist.txt -c copy output.mp4

# Extract audio
ffmpeg -i input.mp4 -vn -acodec copy audio.aac

# Apply brightness
ffmpeg -i input.mp4 -vf "eq=brightness=0.1" output.mp4
```

## Progress Tracking

Parse FFmpeg stderr for progress:
```rust
async fn parse_progress(line: &str) -> Option<f64> {
    // Example: "time=00:01:23.45"
    if let Some(time_str) = extract_time(line) {
        let seconds = time_to_seconds(time_str);
        return Some(seconds / total_duration);
    }
    None
}
```

## Acceptance Criteria

- [ ] Trim video with frame accuracy
- [ ] Concatenate multiple clips
- [ ] Extract audio tracks
- [ ] Report progress during operations
- [ ] Handle FFmpeg errors gracefully
- [ ] Support cancellation

---

**Status:** Not Started  
**Target Completion:** Week 2, Mid
