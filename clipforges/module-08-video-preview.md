# Module 8: Video Preview & Playback

**Owner:** TBD  
**Phase:** 2 (Weeks 3-4)  
**Estimated Effort:** 4-5 days

## Preview Modes

### Single Clip
Use HTML5 video element directly:
```svelte
<video bind:this={videoElement} src={videoUrl} />
```

### Multiple Clips (Composite)
Generate preview frame on-demand via backend:
```rust
#[tauri::command]
async fn render_preview_frame(
    timeline: Timeline,
    time: f64
) -> Result<String, String>  // base64 JPEG
```

## Playback Controls

- Play/Pause
- Seek (click timeline)
- Frame stepping
- Speed control (0.5x, 1x, 2x)

## Frame Caching

LRU cache for scrubbing:
```rust
pub struct PreviewCache {
    frames: LruCache<u64, Vec<u8>>,  // 100 frames
}
```

## Acceptance Criteria

- [ ] Play/pause video
- [ ] Seek to any position
- [ ] Display single clips smoothly
- [ ] Composite multiple clips
- [ ] Cache frames for scrubbing

---

**Status:** Not Started  
**Target:** Week 3, End
