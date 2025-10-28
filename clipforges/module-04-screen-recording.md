# Module 4: Screen Recording

**Owner:** TBD  
**Phase:** 3 (Weeks 5-6)  
**Estimated Effort:** 6-7 days

## Platform Implementations

### macOS: AVFoundation
- Use AVCaptureScreenInput
- Request Screen Recording permission

### Windows: Graphics.Capture API
- Use windows-capture crate
- Requires Windows 10+

### Linux: GStreamer + PipeWire
- Use gstreamer-rs
- PipeWire for Wayland, X11 fallback

## Unified API

```rust
#[tauri::command]
async fn list_recording_sources() -> Result<Vec<Source>, String>

#[tauri::command]
async fn start_recording(source_id: String) -> Result<(), String>

#[tauri::command]
async fn stop_recording() -> Result<String, String>
```

## Acceptance Criteria

- [ ] List available screens/windows
- [ ] Record screen with cursor
- [ ] Save to MP4/WebM
- [ ] Works on primary platform
- [ ] Handle permissions gracefully

---

**Status:** Not Started  
**Target:** Week 5
