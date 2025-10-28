# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ClipForge is a desktop video editor built with **Rust/Tauri + Svelte**. The application enables screen recording, video import, non-linear timeline editing, and professional export capabilities with native performance.

**Tech Stack:**
- Backend: Rust (Tauri v2, Tokio async runtime)
- Frontend: Svelte 4 + TypeScript + Vite 5
- Video Processing: FFmpeg (command-line wrapper approach)
- Storage: SQLite (via rusqlite) + JSON project files
- UI: Canvas-based rendering with Konva.js for timeline

## Development Commands

### Initial Setup
```bash
# Install dependencies (when project is initialized)
npm install

# Install Tauri CLI
npm install -D @tauri-apps/cli

# Initialize Tauri (first time only)
npm create tauri-app
```

### Development
```bash
# Run in development mode (hot reload)
npm run tauri dev

# Build frontend only
npm run build

# Build Rust backend only
cd src-tauri && cargo build

# Run Rust tests
cd src-tauri && cargo test

# Run Rust linter
cd src-tauri && cargo clippy

# Format Rust code
cd src-tauri && cargo fmt
```

### Production
```bash
# Create production build with installers
npm run tauri build

# Bundle outputs to: src-tauri/target/release/bundle/
```

### Testing
```bash
# Backend tests
cd src-tauri && cargo test

# Frontend tests (when implemented)
npm run test

# Integration tests
cd src-tauri && cargo test --test integration
```

## Architecture

### Core Principles

1. **Non-Destructive Editing**: Never modify source video files. All edits stored in Edit Decision List (EDL).

2. **Streaming Architecture**: Pass file paths to FFmpeg instead of loading entire videos into memory.

3. **Command Pattern for FFmpeg**: Use CLI wrapper (not FFI bindings) for faster development. Acceptable 50-100ms overhead per operation.

4. **Platform Abstraction**: Unified trait-based API with platform-specific implementations where necessary (e.g., screen recording).

5. **Optimistic UI Updates**: Update Svelte stores immediately, sync backend asynchronously with rollback on error.

### System Layers

```
Frontend (Svelte)
    ├── Timeline UI (Canvas-based with Konva.js)
    ├── Video Preview (HTML5 video with custom protocol)
    └── Media Library
           ↓ IPC (Tauri Commands + Events)
Backend (Rust)
    ├── Timeline Engine (EDL state management)
    ├── FFmpeg Integration (video processing)
    ├── File System & Media (SQLite metadata + thumbnails)
    └── Screen Recording (platform-specific traits)
```

### Data Flow Patterns

**Video Import:**
1. Frontend validates file → invoke('import_media_file')
2. Backend: hash → check duplicate → ffprobe metadata → generate thumbnail → SQLite insert
3. Frontend receives MediaFile object → updates UI

**Timeline Edit:**
1. Frontend updates local state optimistically → invoke('move_clip')
2. Backend validates → updates timeline state → serializes to disk
3. On error: frontend rolls back local state

**Export:**
1. invoke('export_timeline') → Backend builds FFmpeg filter_complex
2. Backend emits progress events (percentage, ETA)
3. Frontend updates progress bar in real-time

### IPC Communication

- **Standard Commands**: JSON-serialized Tauri commands for most operations
- **Custom Protocol**: `stream://` URI for binary video streaming (no JSON overhead)
- **Event System**: Long-running operations emit progress events to frontend

## Module Structure (8 Modules)

The project is organized into 8 independent modules across 4 development phases:

**Phase 1 - Foundation (Critical):**
1. Application Shell - Tauri setup, window management
2. File System & Media - Import, metadata, SQLite
3. FFmpeg Integration - Video processing wrapper
5. Timeline Engine - Edit Decision List

**Phase 2 - Core Editing (Critical):**
7. Timeline UI - Visual editor, drag-drop, Canvas rendering
8. Video Preview - Playback, composite preview

**Phase 3 - Recording & Export (High Priority):**
4. Screen Recording - Platform-specific capture
6. Export & Rendering - Timeline to video conversion

See `clipforges/` directory for detailed module specifications.

## Key Data Structures

Located in `src-tauri/src/` (when implemented):

```rust
// Core types
pub struct MediaFile {
    pub id: String,
    pub path: PathBuf,
    pub duration: f64,
    pub resolution: Resolution,
    pub codec: VideoCodec,
    pub hash: String,
    pub thumbnail_path: Option<PathBuf>,
}

pub struct Timeline {
    pub id: String,
    pub framerate: f64,
    pub tracks: Vec<Track>,
}

pub struct Track {
    pub id: String,
    pub track_type: TrackType,  // Video | Audio | Overlay
    pub clips: Vec<Clip>,
}

pub struct Clip {
    pub media_file_id: String,
    pub track_position: f64,
    pub duration: f64,
    pub trim_start: f64,
    pub trim_end: f64,
}
```

## Performance Requirements

- Timeline responsiveness: 30 FPS with 20+ clips
- Export speed: 1x real-time for 1080p
- Memory usage: <300MB during editing
- Launch time: <3 seconds
- Bundle size: <15MB per platform

## Important Constraints

1. **File System Access**: Tauri allowlist restricts to `$APPDATA/*` and `$HOME/*`
2. **Command Injection**: Always pass FFmpeg arguments separately, never via shell string interpolation
3. **CSP**: Content Security Policy restricts script/style sources
4. **Platform Support**: macOS 11+, Windows 10+, Ubuntu 20.04+

## Security Considerations

**Never do this:**
```rust
// BAD: Command injection vulnerability
Command::new("sh").arg("-c").arg(format!("ffmpeg -i {}", user_input))
```

**Always do this:**
```rust
// GOOD: Safe argument passing
Command::new("ffmpeg").arg("-i").arg(user_input)
```

## Error Handling

- Use `thiserror` for custom error types
- Convert Rust errors to user-friendly strings before sending to frontend
- Frontend displays errors without technical details

## Testing Strategy

- **Unit tests**: All core functions (Timeline, FFmpeg wrapper, File service)
- **Integration tests**: Full export pipeline, import workflow
- **Acceptance criteria**: Each module has specific acceptance tests in its spec

## Documentation Reference

Comprehensive documentation in `clipforges/`:
- `README.md` - Quick navigation
- `HANDOFF.md` - Complete project overview
- `INDEX.md` - Full documentation index
- `02-technical-architecture.md` - Deep technical design
- `data-structures.md` - Type definitions
- `dependencies.md` - Required packages
- `module-*.md` - 8 detailed module specifications

## Development Workflow

1. Read relevant module specification in `clipforges/modules/`
2. Check dependencies - Module 1 must complete before others
3. Implement incrementally following acceptance criteria
4. Write tests as you go (`cargo test`)
5. Run clippy for linting (`cargo clippy`)
6. Submit PRs with tests passing

## Progress Tracking

Use `progress.md` file to track implementation status across modules and phases. Mark completed items as you progress through the 8-week timeline.
