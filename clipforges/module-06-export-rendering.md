# Module 6: Export & Rendering

**Owner:** TBD  
**Phase:** 3 (Weeks 5-6)  
**Estimated Effort:** 5-6 days

## Export Pipeline

1. Validate timeline
2. Build FFmpeg filter_complex
3. Execute FFmpeg with progress tracking
4. Verify output file

## Export Settings

```rust
pub struct ExportSettings {
    pub resolution: Resolution,
    pub framerate: f64,
    pub video_codec: String,
    pub video_bitrate: u32,
    pub audio_codec: String,
}
```

## Presets

- YouTube 1080p
- Instagram Post (1:1)
- Twitter Video
- Custom

## Progress Events

Emit real-time progress to frontend:
```rust
window.emit("export-progress", ExportProgress {
    percentage: 45.2,
    current_frame: 1356,
    fps: 30.0,
    time_remaining: Duration::from_secs(180),
});
```

## Acceptance Criteria

- [ ] Export timeline to MP4
- [ ] Show progress (percentage, ETA)
- [ ] Support 720p, 1080p, 4K
- [ ] Apply effects during export
- [ ] Handle cancellation

---

**Status:** Not Started  
**Target:** Week 6
