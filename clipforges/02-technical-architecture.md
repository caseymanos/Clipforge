# Technical Architecture

## System Overview

ClipForge uses a **hybrid architecture** combining Rust backend for performance-critical operations with Svelte frontend for rich UI interactions.

```
┌─────────────────────────────────────────────────────────┐
│                    Frontend (Svelte)                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │   Timeline   │  │    Video     │  │    Media     │ │
│  │      UI      │  │   Preview    │  │   Library    │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
│         │                  │                  │         │
│         └──────────────────┼──────────────────┘         │
│                            │                            │
│                    IPC Layer (Tauri)                    │
│                            │                            │
├────────────────────────────┼────────────────────────────┤
│                    Backend (Rust)                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │   Timeline   │  │    FFmpeg    │  │  Recording   │ │
│  │    Engine    │  │  Integration │  │    System    │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
│         │                  │                  │         │
│         └──────────────────┼──────────────────┘         │
│                            │                            │
│                   File System / OS APIs                 │
└─────────────────────────────────────────────────────────┘
```

## Technology Stack

### Frontend Layer

**Framework:** Svelte 4 + TypeScript
- Compile-time optimization
- Smallest bundle size (~15KB)
- Built-in reactivity
- Excellent performance

**Build Tool:** Vite 5
- Lightning-fast HMR
- Optimized production builds
- Native ES modules

**UI Components:**
- Konva.js - Canvas-based timeline rendering
- WaveSurfer.js - Audio waveform visualization
- Custom Svelte components

**State Management:**
- Svelte Stores for UI state
- Tauri State for persistent data
- Reactive subscriptions

### Backend Layer

**Language:** Rust 1.75+
- Memory safety without garbage collection
- Zero-cost abstractions
- Fearless concurrency
- Rich type system

**Framework:** Tauri v2
- Smaller bundle than Electron (8MB vs 150MB)
- Better security model
- System WebView integration
- Native performance

**Async Runtime:** Tokio
- Multi-threaded work-stealing scheduler
- Async I/O for file operations
- Channel-based communication

### Video Processing

**FFmpeg:** Command-line wrapper approach
- Faster development than FFI bindings
- Battle-tested stability
- Cross-platform compatibility
- Trade 50-100ms startup overhead for simplicity

**Alternative considered:** `ffmpeg-next` crate
- Pros: No process spawning overhead
- Cons: Complex FFI, harder debugging, longer dev time
- Decision: Use command wrapper for MVP, evaluate FFI post-launch

### Data Storage

**Project Files:** JSON serialization via `serde`
- Human-readable format
- Version control friendly
- Easy debugging

**Media Library:** SQLite via `rusqlite`
- Embedded database
- No external dependencies
- Fast metadata queries
- ACID transactions

**Thumbnail Cache:** File system
- JPEG thumbnails in app data directory
- LRU eviction policy
- Lazy generation

## Architecture Principles

### 1. Non-Destructive Editing

**Never modify source files.** All edits stored in Edit Decision List (EDL).

```rust
// Good: Store edit decisions
struct Clip {
    source_path: PathBuf,
    trim_start: f64,  // Don't modify file
    trim_end: f64,    // Just track where to read
}

// Bad: Modifying original files
// fn trim_source_file(path: &Path) { ... }
```

**Benefits:**
- Unlimited undo/redo
- No quality loss from re-encoding
- Fast preview generation
- Multiple versions from same source

### 2. Streaming Architecture

**Never load entire videos into memory.** Pass file paths to processing systems.

```rust
// Good: Stream processing
fn process_video(input_path: &Path, output_path: &Path) {
    Command::new("ffmpeg")
        .args(&["-i", input_path.to_str().unwrap()])
        .spawn()
}

// Bad: Loading entire file
// let video_data = std::fs::read(input_path)?; // OOM on 4K video!
```

**Benefits:**
- Handle files larger than RAM
- Lower memory footprint
- Better performance
- Scalable to 4K/8K content

### 3. Command Pattern for Video Operations

**All video processing via FFmpeg CLI,** not FFI bindings (initially).

**Rationale:**
- Faster development velocity
- Easier debugging (can test commands manually)
- Less platform-specific code
- Acceptable 50-100ms overhead per operation

### 4. Platform Abstraction

**Unified API with platform-specific implementations** where necessary.

```rust
pub trait ScreenRecorder {
    async fn list_sources() -> Result<Vec<Source>>;
    async fn start_recording(&mut self, source: Source) -> Result<()>;
    async fn stop_recording(&mut self) -> Result<PathBuf>;
}

#[cfg(target_os = "macos")]
pub struct MacOSRecorder { /* AVFoundation impl */ }

#[cfg(target_os = "windows")]
pub struct WindowsRecorder { /* Graphics.Capture impl */ }

#[cfg(target_os = "linux")]
pub struct LinuxRecorder { /* GStreamer impl */ }
```

### 5. Optimistic UI Updates

**Update UI immediately,** sync backend asynchronously.

```typescript
// Svelte store updates optimistically
timelineStore.update(timeline => {
    timeline.tracks[0].clips.push(newClip);
    return timeline;
});

// Backend sync happens async
invoke('add_clip_to_timeline', { clip: newClip })
    .catch(err => {
        // Rollback on error
        timelineStore.update(timeline => {
            timeline.tracks[0].clips.pop();
            return timeline;
        });
    });
```

## Data Flow

### Video Import Flow

```
User drops file
    ↓
Frontend validates (file type, size)
    ↓
invoke('import_media_file', {path})
    ↓
Backend:
  1. Calculate SHA-256 hash
  2. Check for duplicate
  3. Extract metadata (ffprobe)
  4. Generate thumbnail
  5. Insert to SQLite
  6. Emit progress events
    ↓
Frontend receives MediaFile object
    ↓
Update media library UI
```

### Timeline Edit Flow

```
User drags clip on timeline
    ↓
Frontend updates local state (optimistic)
    ↓
invoke('move_clip', {clipId, position})
    ↓
Backend:
  1. Validate new position
  2. Check for overlaps
  3. Update timeline state
  4. Serialize to disk
    ↓
Frontend receives confirmation
    ↓
(On error: rollback local state)
```

### Export Flow

```
User clicks Export
    ↓
invoke('export_timeline', {timeline, settings})
    ↓
Backend:
  1. Validate timeline
  2. Build FFmpeg filter_complex
  3. Spawn FFmpeg process
  4. Parse progress from stderr
  5. Emit progress events (percentage, ETA)
    ↓
Frontend listens to progress events
    ↓
Update progress bar in real-time
    ↓
Backend emits 'export-complete' with path
    ↓
Frontend shows success notification
```

### Screen Recording Flow

```
User clicks Record Screen
    ↓
invoke('list_recording_sources')
    ↓
Backend returns [screens, windows]
    ↓
User selects source
    ↓
invoke('start_recording', {sourceId, outputPath})
    ↓
Backend (platform-specific):
  macOS: AVFoundation capture session
  Windows: Graphics.Capture API
  Linux: GStreamer pipeline
    ↓
Recording active (emit duration events)
    ↓
User clicks Stop
    ↓
invoke('stop_recording')
    ↓
Backend finalizes file, returns path
    ↓
Auto-import to media library
```

## IPC Communication

### Standard Commands

Most operations use Tauri's standard command system:

```rust
#[tauri::command]
async fn my_command(param: String) -> Result<ReturnType, String> {
    // Automatically serializes to/from JSON
}
```

**Pros:**
- Simple API
- Type-safe with serde
- Automatic error handling

**Cons:**
- JSON serialization overhead
- Not suitable for binary data
- Blocking for large payloads

### Custom Protocols for Binary Data

For video streaming, use custom URI protocol:

```rust
app.register_uri_scheme_protocol("stream", move |_app, request| {
    let path = request.uri().path();
    let file = std::fs::read(path)?;
    
    ResponseBuilder::new()
        .header("Content-Type", "video/mp4")
        .body(file)
});
```

Frontend access:
```typescript
const videoUrl = convertFileSrc('/path/to/video.mp4');
videoElement.src = videoUrl; // Streams via custom protocol
```

**Pros:**
- No JSON serialization
- Native browser video decoding
- Efficient memory usage

### Event System for Progress

For long-running operations, emit events:

```rust
#[tauri::command]
async fn export_video(window: Window) -> Result<(), String> {
    for progress in 0..100 {
        window.emit("export-progress", ExportProgress {
            percentage: progress as f64,
            // ... other fields
        })?;
    }
    Ok(())
}
```

Frontend listener:
```typescript
await listen('export-progress', (event) => {
    console.log('Progress:', event.payload.percentage);
});
```

## Performance Optimizations

### Timeline Rendering

**Problem:** DOM-based timeline lags with 50+ clips

**Solution:** Canvas-based rendering with Konva.js

```javascript
// Render clips to canvas layer
const layer = new Konva.Layer();
clips.forEach(clip => {
    const rect = new Konva.Rect({
        x: clip.position * pixelsPerSecond,
        width: clip.duration * pixelsPerSecond,
        height: 60,
        fill: '#667eea',
        draggable: true,
    });
    layer.add(rect);
});
stage.add(layer);
```

**Performance:** 60 FPS with 200+ objects

### Thumbnail Generation

**Problem:** Generating thumbnails blocks UI

**Solution:** Async generation with caching

```rust
pub struct ThumbnailCache {
    cache: Arc<Mutex<LruCache<String, PathBuf>>>,
}

impl ThumbnailCache {
    pub async fn get_or_generate(
        &self,
        video_path: &Path,
        timestamp: f64
    ) -> Result<PathBuf> {
        let key = format!("{}:{}", video_path.display(), timestamp);
        
        if let Some(thumb) = self.cache.lock().await.get(&key) {
            return Ok(thumb.clone());
        }
        
        // Generate in background
        let thumb = self.generate_thumbnail(video_path, timestamp).await?;
        self.cache.lock().await.put(key, thumb.clone());
        
        Ok(thumb)
    }
}
```

### Preview Frame Caching

**Problem:** Scrubbing timeline re-renders same frames

**Solution:** LRU cache for rendered frames

```rust
use lru::LruCache;

pub struct PreviewCache {
    frames: LruCache<u64, Vec<u8>>, // timestamp -> JPEG bytes
}

// Cache 100 frames (~15MB for 1080p JPEGs)
let cache = PreviewCache::new(100);
```

### FFmpeg Process Pooling

**Problem:** Spawning FFmpeg process has 50-100ms overhead

**Solution:** Keep warm FFmpeg process for quick operations

```rust
pub struct FFmpegPool {
    processes: Vec<Child>,
}

impl FFmpegPool {
    pub async fn execute(&mut self, command: Command) -> Result<Output> {
        // Reuse existing process if available
        // Spawn new one if pool empty
        // Limit pool size to prevent resource exhaustion
    }
}
```

**Note:** Implement only if profiling shows process spawning is bottleneck

## Security Considerations

### File System Access

Tauri's allowlist restricts file access:

```json
{
  "tauri": {
    "allowlist": {
      "fs": {
        "scope": ["$APPDATA/*", "$HOME/*"]
      }
    }
  }
}
```

**Implications:**
- Can only read files user explicitly selects
- Cannot access system directories
- Sandbox prevents malicious file access

### Command Injection Prevention

**Never** pass user input directly to shell:

```rust
// Bad: Command injection vulnerability
Command::new("sh")
    .arg("-c")
    .arg(format!("ffmpeg -i {}", user_input)) // DANGEROUS!

// Good: Use proper argument passing
Command::new("ffmpeg")
    .arg("-i")
    .arg(user_input) // Safe: treated as single argument
```

### CSP (Content Security Policy)

Restrict what frontend can execute:

```json
{
  "tauri": {
    "security": {
      "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'"
    }
  }
}
```

## Error Handling Strategy

### Rust Error Types

Use `thiserror` for custom errors:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClipForgeError {
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
    
    #[error("Invalid video format: {0}")]
    InvalidFormat(String),
    
    #[error("FFmpeg error: {0}")]
    FFmpegError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

### Error Propagation to Frontend

Convert Rust errors to user-friendly messages:

```rust
#[tauri::command]
async fn import_video(path: String) -> Result<MediaFile, String> {
    do_import(&path)
        .await
        .map_err(|e| match e {
            ClipForgeError::FileNotFound(_) => 
                "Video file not found. Please check the path.".to_string(),
            ClipForgeError::InvalidFormat(fmt) => 
                format!("Unsupported video format: {}", fmt),
            _ => format!("Import failed: {}", e),
        })
}
```

### Frontend Error Display

Show errors in UI without technical details:

```svelte
<script>
  async function importVideo(path) {
    try {
      await invoke('import_video', { path });
    } catch (error) {
      // Error is user-friendly string from backend
      showNotification('error', error);
    }
  }
</script>
```

## Testing Strategy

### Unit Tests (Rust)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_timeline_add_clip() {
        let mut timeline = Timeline::new("test", 30.0);
        let clip = Clip::new("media1", 0.0, 10.0);
        
        assert!(timeline.add_clip("track1", clip).is_ok());
        assert_eq!(timeline.tracks[0].clips.len(), 1);
    }
    
    #[tokio::test]
    async fn test_ffmpeg_trim() {
        let ffmpeg = FFmpegService::new();
        let result = ffmpeg.trim_video(
            Path::new("test.mp4"),
            Path::new("trimmed.mp4"),
            5.0,
            10.0,
            |_| {}
        ).await;
        
        assert!(result.is_ok());
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_full_export_pipeline() {
    // Create timeline
    let mut timeline = Timeline::new("test", 30.0);
    
    // Add clips
    timeline.add_clip(/* ... */);
    
    // Export
    let result = export_timeline(timeline, settings).await;
    
    // Verify output file
    assert!(Path::new("output.mp4").exists());
    
    // Verify video properties
    let info = get_video_info("output.mp4").await.unwrap();
    assert_eq!(info.resolution.width, 1920);
}
```

### Frontend Tests (Vitest)

```typescript
import { render } from '@testing-library/svelte';
import Timeline from './Timeline.svelte';

test('renders timeline with clips', () => {
    const { getByText } = render(Timeline, {
        props: {
            timeline: {
                tracks: [/* mock data */]
            }
        }
    });
    
    expect(getByText('Track 1')).toBeInTheDocument();
});
```

## Deployment Architecture

### Build Process

```bash
# Frontend build
npm run build
# Outputs to: dist/

# Rust backend build
cargo build --release
# Outputs to: src-tauri/target/release/

# Bundle with Tauri
npm run tauri build
# Creates platform-specific installers
```

### Bundle Contents

```
ClipForge.app/  (macOS)
├── Contents/
│   ├── MacOS/
│   │   ├── ClipForge           # Main executable
│   │   └── ffmpeg              # Bundled FFmpeg
│   ├── Resources/
│   │   ├── frontend assets
│   │   └── icon.icns
│   └── Info.plist
```

### Update Strategy

**Phase 1:** Manual downloads from website

**Phase 2:** Built-in updater (Tauri updater)
```rust
tauri::Builder::default()
    .plugin(tauri_plugin_updater::Builder::new().build())
    .run(tauri::generate_context!())
```

**Phase 3:** App store distribution (Mac App Store, Microsoft Store)

## Scalability Considerations

### Current Design Limits

- **Timeline:** 1000 clips (Canvas rendering limit)
- **Video Size:** 100GB (file system dependent)
- **Export Length:** 4 hours (reasonable upper bound)
- **Concurrent Exports:** 1 (FFmpeg process limitation)

### Future Optimizations

1. **Worker Pool for Exports**  
   Support multiple concurrent exports

2. **Distributed Rendering**  
   Split export across multiple machines

3. **Cloud Export**  
   Offload rendering to cloud infrastructure

4. **Timeline Virtualization**  
   Only render visible portion of timeline

## Monitoring & Debugging

### Logging Strategy

```rust
use tracing::{info, warn, error};

#[tauri::command]
async fn export_video() -> Result<()> {
    info!("Starting video export");
    
    match do_export().await {
        Ok(_) => {
            info!("Export completed successfully");
            Ok(())
        }
        Err(e) => {
            error!("Export failed: {}", e);
            Err(e)
        }
    }
}
```

### Performance Profiling

**Rust:** Use `cargo flamegraph`
```bash
cargo install flamegraph
cargo flamegraph --bin clipforge
```

**Frontend:** Chrome DevTools Performance tab

### Crash Reporting

**Phase 1:** Manual bug reports

**Phase 2:** Integrate Sentry or similar
```rust
use sentry_tauri;

tauri::Builder::default()
    .plugin(sentry_tauri::plugin())
```

---

**Document Owner:** Tech Lead  
**Last Updated:** October 27, 2025  
**Status:** Approved  
**Next Review:** After Phase 2 (Week 4)

## Data Storage: SQLite vs In-Memory HashMap

### Decision: Hybrid Approach (SQLite + Optional Cache)

**Primary Storage: SQLite**
- Persistent media library across app restarts
- Indexed lookups for deduplication (hash-based)
- Handles 1000+ files without memory constraints
- SQL queries for filtering by codec, date, size
- No external dependencies (bundled SQLite)

**Optional Cache: HashMap**
- Fast lookups for frequently accessed files
- Reduces disk I/O for active editing sessions
- Add only if profiling shows bottleneck

### Why Not Pure HashMap?

**HashMap-only approach problems:**
```rust
// Lost on restart - user frustration
pub struct FileService {
    media_files: HashMap<String, MediaFile>,  // ❌ In-memory only
}
// User imports 50 videos → Closes app → All metadata lost!
```

**SQLite advantages:**
```rust
// Survives restarts - professional UX
pub struct FileService {
    db: Database,                           // ✅ Persistent
    cache: HashMap<String, MediaFile>,      // ✅ Fast (optional)
}
// User imports 50 videos → Metadata saved → Fast reload on next launch
```

### Performance Comparison

| Operation | HashMap | SQLite | SQLite + Index |
|-----------|---------|--------|----------------|
| Insert | O(1) ~10ns | O(log n) ~1ms | O(log n) ~1ms |
| Lookup by ID | O(1) ~10ns | O(log n) ~100μs | O(log n) ~100μs |
| Duplicate check | O(n) linear scan | O(log n) indexed | O(log n) ~100μs |
| Persist | Manual serialize | Automatic | Automatic |
| Memory | Always in RAM | Disk + cache | Disk + cache |

**With 1000 files:**
- HashMap: O(n) = 1000 iterations for duplicate check
- SQLite: O(log n) = ~10 comparisons with index

### Implementation Strategy

**Phase 1-4 (MVP):** SQLite only
```rust
pub struct FileService {
    db: Database,
    thumbnail_generator: ThumbnailGenerator,
}
```

**Phase 7-8 (Optimization):** Add cache if needed
```rust
pub struct FileService {
    db: Database,
    cache: Arc<RwLock<HashMap<String, MediaFile>>>,
    thumbnail_generator: ThumbnailGenerator,
}
```

**Decision point:** Profile during Week 7
- If `get_by_id()` calls < 100ms: Keep SQLite only
- If `get_by_id()` calls > 100ms: Add HashMap cache

### When to Use Each Approach

| Scenario | Recommendation |
|----------|----------------|
| Prototype/demo (<100 files) | HashMap acceptable |
| Production app | SQLite required |
| High-frequency reads (1000+ lookups/sec) | SQLite + cache |
| Complex queries (filter by codec, date) | SQLite only |

**ClipForge uses: SQLite (MVP) → SQLite + cache (if needed)**
