# ClipForge User Guide

**Complete guide to using ClipForge desktop video editor**

Version: 0.1.0 (MVP)
Last Updated: October 28, 2025

---

## Table of Contents

1. [Introduction](#introduction)
2. [Getting Started](#getting-started)
3. [Interface Overview](#interface-overview)
4. [Importing Media](#importing-media)
5. [Timeline Editing](#timeline-editing)
6. [Video Preview](#video-preview)
7. [Exporting Videos](#exporting-videos)
8. [Screen Recording](#screen-recording)
9. [Project Management](#project-management)
10. [Tips & Tricks](#tips--tricks)
11. [Keyboard Shortcuts](#keyboard-shortcuts)
12. [Troubleshooting](#troubleshooting)

---

## Introduction

### What is ClipForge?

ClipForge is a modern desktop video editor designed for speed, simplicity, and professional results. Whether you're creating content for YouTube, editing screen recordings, or producing social media videos, ClipForge provides the tools you need without the complexity of traditional video editors.

### Key Benefits

- **Fast:** Built with Rust for native performance
- **Non-Destructive:** Your original files are never modified
- **Cross-Platform:** Works on macOS, Windows, and Linux
- **Open Source:** Free and customizable
- **Lightweight:** Small app size, minimal system requirements

---

## Getting Started

### System Requirements

**Minimum:**
- 8 GB RAM
- Dual-core processor
- 500 MB disk space
- FFmpeg installed

**Recommended:**
- 16 GB RAM
- Quad-core processor (i5/Ryzen 5 or better)
- SSD with 50+ GB free space
- Dedicated GPU

### Installing FFmpeg

ClipForge requires FFmpeg for video processing.

**macOS:**
```bash
brew install ffmpeg
```

**Windows:**
1. Download from [ffmpeg.org](https://ffmpeg.org/download.html)
2. Extract to `C:\Program Files\ffmpeg`
3. Add to system PATH

**Linux (Ubuntu):**
```bash
sudo apt update
sudo apt install ffmpeg
```

**Verify installation:**
```bash
ffmpeg -version
```

### First Launch

1. Open ClipForge from Applications (macOS) or Start Menu (Windows)
2. You'll see three main sections:
   - **Media Library** (top)
   - **Video Preview** (middle)
   - **Timeline Editor** (bottom)

---

## Interface Overview

### Media Library

**Purpose:** Manage all your imported video files

**Features:**
- Thumbnail previews
- File metadata (resolution, codec, duration, size)
- Search by filename
- Sort by date, name, duration, or size
- Select files (click)
- Add to timeline (double-click)
- Delete files (trash icon on hover)

**Location:** Top section of the app

---

### Video Preview

**Purpose:** Watch your timeline in real-time

**Features:**
- Play/pause button (▶/⏸)
- Frame stepping (⏮/⏭)
- Timeline scrubber (drag to seek)
- Time display (current/total)
- Playback speed controls (0.5x, 1x, 2x)
- Automatic composite rendering (multiple clips)

**Controls:**
- **Play/Pause:** Click button or Space bar *(planned)*
- **Seek:** Click or drag on scrubber
- **Frame Step:** Click ⏮/⏭ buttons or arrow keys *(planned)*
- **Speed:** Click 0.5x, 1x, or 2x buttons

---

### Timeline Editor

**Purpose:** Visual editing workspace for arranging clips

**Features:**
- Canvas-based rendering (smooth 60 FPS)
- Multi-track support (video, audio, overlay)
- Drag-and-drop clips
- Trim clips with edge handles
- Zoom in/out (mouse wheel)
- Scroll horizontally (Shift + mouse wheel)
- Playhead control (red circle)
- Time ruler with markers

**Track Types:**
- **Video Track:** Main video content
- **Audio Track:** Audio-only clips
- **Overlay Track:** Graphics, text, effects *(planned)*

---

## Importing Media

### Supported Formats

**Video:**
- MP4 (.mp4)
- MOV (.mov)
- WebM (.webm)
- AVI (.avi)
- MKV (.mkv)

**Audio:** *(Coming soon)*
- MP3, WAV, AAC

**Images:** *(Coming soon)*
- PNG, JPEG, GIF

---

### How to Import

**Method 1: Import Button**
1. Click **"Import Media"** in Media Library section
2. Select one or more video files
3. Click "Open"
4. Wait for thumbnails to generate

**Method 2: Drag-and-Drop** *(Planned)*
1. Drag video files from Finder/Explorer
2. Drop onto Media Library section
3. Thumbnails generate automatically

---

### What Happens During Import

1. **File Analysis:** FFprobe extracts metadata
   - Duration
   - Resolution
   - Codec information
   - Framerate
   - Bitrate

2. **Duplicate Detection:** SHA-256 hash checks for existing files
   - Prevents importing the same file twice
   - Saves storage space

3. **Thumbnail Generation:** Creates preview image
   - Extracted from video midpoint
   - Saved to app data directory

4. **Database Storage:** File metadata stored in SQLite
   - Enables fast searching
   - Persists across app restarts

---

### Managing Imported Files

**View Details:**
- Hover over thumbnail to see filename
- File size shown below thumbnail
- Resolution, codec displayed in UI

**Search:**
- Type in search box to filter by filename
- Search is case-insensitive

**Sort:**
- By Date: Newest or oldest first
- By Name: Alphabetical
- By Duration: Longest or shortest
- By Size: Largest or smallest

**Delete:**
1. Hover over file
2. Click trash icon (appears on hover)
3. Confirm deletion
4. File removed from library (original file untouched)

---

## Timeline Editing

### Adding Clips to Timeline

**Method 1: Double-Click** *(Planned)*
1. Double-click file in Media Library
2. Clip appears on first available track
3. Positioned after existing clips

**Method 2: Drag-and-Drop**
1. Click and hold clip in Media Library
2. Drag to timeline
3. Drop on desired track and position
4. Release mouse

---

### Selecting Clips

**Single Selection:**
- Click any clip on timeline
- Selected clip highlighted with white border

**Multi-Selection:** *(Planned)*
- Hold Shift and click multiple clips
- Or drag selection rectangle

**Deselect:**
- Click empty space on timeline

---

### Moving Clips

1. Click clip to select
2. Drag to new position
3. Can move to different track
4. Clips automatically snap *(planned)*

**Constraints:**
- Clips cannot overlap on same track
- Visual feedback shows valid drop zones

---

### Trimming Clips

**Purpose:** Adjust start/end points without cutting

**How to Trim:**
1. Select clip (click to select)
2. White handles appear on clip edges
3. Drag left handle to trim start
4. Drag right handle to trim end
5. Trimmed portions hidden, not deleted

**Visual Feedback:**
- Clip width changes as you trim
- Duration updates in real-time
- Original file unchanged (non-destructive)

---

### Splitting Clips

**Purpose:** Cut one clip into two separate clips

**How to Split:** *(Planned - using command)*
1. Position playhead where you want to split
2. Select clip
3. Invoke `split_clip_at_time` command
4. One clip becomes two

---

### Zooming and Scrolling

**Zoom In/Out:**
- **Mouse Wheel:** Scroll up to zoom in, down to zoom out
- Centers on mouse cursor position
- Zoom range: 1x to 10x

**Horizontal Scroll:**
- **Shift + Mouse Wheel:** Scroll timeline left/right
- Or use scrollbar at bottom *(if visible)*

**Tips:**
- Zoom in for precise edits
- Zoom out to see full timeline

---

### Playhead Control

**Purpose:** Shows current time position

**Moving Playhead:**
- Drag the red circle
- Click anywhere on time ruler
- Use frame step buttons in preview

**Playhead Sync:**
- Preview automatically updates to playhead position
- Playhead follows during video playback

---

### Track Management

**Add Track:** *(Planned)*
```typescript
// Backend command (developers)
await invoke('add_track', { track_type: 'Video' });
```

**Remove Track:** *(Planned)*
```typescript
// Backend command (developers)
await invoke('remove_track', { track_id: 'track-uuid' });
```

**Mute Track:**
- Muted tracks excluded from export
- Useful for disabling audio tracks temporarily

---

## Video Preview

### Preview Modes

**Single Clip Mode:**
- Activated when only one clip at current time
- Uses HTML5 video player
- Fast, efficient playback

**Composite Mode:**
- Activated when multiple clips overlap
- Renders frame-by-frame from backend
- Combines all visible clips
- Shows final export result

---

### Playback Controls

**Play/Pause:**
- Click ▶ button to start playback
- Click ⏸ to pause
- *(Keyboard: Space bar - planned)*

**Frame Stepping:**
- Click ⏮ to go back one frame
- Click ⏭ to go forward one frame
- Useful for precise editing

**Seek/Scrub:**
- Drag scrubber handle
- Click anywhere on scrubber bar
- Real-time preview updates

**Playback Speed:**
- **0.5x:** Half speed (slow motion)
- **1x:** Normal speed
- **2x:** Double speed

---

### Timeline Sync

Preview always shows:
- Current playhead position
- All active clips at that time
- Applied effects *(if any)*
- Correct track layering

**Performance:**
- Target: 30 FPS preview rendering
- Smooth playback for most timelines
- May slow with 20+ clips or effects

---

## Exporting Videos

### Export Settings

**Presets Available:**
1. **YouTube 1080p**
   - Resolution: 1920x1080
   - Codec: H.264
   - Bitrate: 8000 kbps video, 192 kbps audio
   - Format: MP4

2. **Instagram Post**
   - Resolution: 1080x1080 (square)
   - Codec: H.264
   - Bitrate: 5000 kbps video, 128 kbps audio
   - Format: MP4

3. **Twitter Video**
   - Resolution: 1280x720
   - Codec: H.264
   - Bitrate: 6000 kbps video, 128 kbps audio
   - Format: MP4

---

### How to Export

**Using Export Dialog:** *(Planned UI component)*

1. Click **"Export"** button
2. Choose output location
3. Select preset or customize settings
4. Click "Start Export"
5. Monitor progress bar
6. Export completes to selected location

**Using Backend Command (Current):**

```typescript
// For developers/testing
const result = await invoke('export_timeline', {
  timeline: currentTimeline,
  settings: ExportSettings.youtube_1080p(),
  output_path: '/path/to/output.mp4',
  media_files_map: mediaFilesObject
});
```

---

### Export Progress

**Real-Time Updates:**
- Percentage complete (0-100%)
- Current frame being processed
- Encoding FPS
- Estimated time remaining

**Export Speed:**
- **Target:** ≥1.0x real-time for 1080p
- **Example:** 60-second video exports in <60 seconds
- Depends on CPU, clip count, effects

**Canceling Export:**
- Click "Cancel" button *(planned)*
- Or call `cancel_export` command
- Partial file deleted

---

### Export Quality

**Video Quality:**
- Controlled by bitrate setting
- Higher bitrate = larger file, better quality
- 8000 kbps good for YouTube 1080p

**Audio Quality:**
- 192 kbps: High quality
- 128 kbps: Good quality (smaller file)
- AAC codec widely supported

---

## Screen Recording

### Platform Support

**macOS:**
- ✅ Fully supported (AVFoundation)
- Native screen capture
- Permission required

**Windows:**
- 🟡 Stub implementation (Windows.Graphics.Capture)
- Coming in future release

**Linux:**
- 🟡 Stub implementation (GStreamer)
- Coming in future release

---

### Recording on macOS

**First Time Setup:**
1. Launch ClipForge
2. Click "Start Recording" *(when UI added)*
3. macOS prompts for Screen Recording permission
4. Open System Settings > Privacy & Security > Screen Recording
5. Enable ClipForge
6. Restart ClipForge

**Starting a Recording:**
1. Click "Start Recording" button
2. Select screen/window to record
3. Click "Start"
4. ClipForge minimizes (optional)
5. Recording begins

**Stopping a Recording:**
1. Click ClipForge in Dock
2. Click "Stop Recording"
3. Video saved to default location
4. Auto-imported to Media Library *(optional)*

**Settings:**
- Resolution: Match screen resolution
- Framerate: 30 fps (default)
- Codec: H.264
- Quality: High

---

## Project Management

### Saving Projects

**How to Save:**
1. Click **"Save Project"** button
2. Choose location and filename
3. File saved with `.cfp` extension
4. JSON format, human-readable

**What's Saved:**
- Timeline structure
- All tracks and clips
- Clip positions and trim points
- Effects applied *(if any)*
- Project metadata

**What's NOT Saved:**
- Original video files (referenced by path)
- Generated thumbnails
- Preview cache

---

### Loading Projects

**How to Load:**
1. Click **"Load Project"** button
2. Select `.cfp` file
3. Project loads into timeline

**Missing Media:**
- If original files moved, import will fail
- Keep media files in same location
- Or use relative paths *(planned)*

---

### Project File Format

**Example `.cfp` file:**
```json
{
  "id": "timeline-uuid",
  "name": "My Project",
  "framerate": 30.0,
  "resolution": { "width": 1920, "height": 1080 },
  "tracks": [
    {
      "id": "track-uuid",
      "track_type": "Video",
      "clips": [
        {
          "id": "clip-uuid",
          "media_file_id": "file-uuid",
          "track_position": 0.0,
          "duration": 10.5,
          "trim_start": 2.0,
          "trim_end": 0.5,
          "volume": 1.0,
          "speed": 1.0,
          "effects": []
        }
      ],
      "muted": false,
      "locked": false
    }
  ],
  "duration": 30.0
}
```

---

## Tips & Tricks

### Performance

**Faster Timeline:**
- Close unused apps
- Import lower resolution proxies *(planned)*
- Reduce clip count
- Disable effects during editing

**Faster Export:**
- Use presets instead of custom settings
- Close other apps during export
- Export to SSD, not external drive

**Memory Management:**
- Restart app if memory grows
- Clear preview cache: invoke `clear_preview_cache`
- Reload timeline to free memory

---

### Workflow Tips

**Organizing Media:**
- Use descriptive filenames
- Sort by date for chronological editing
- Delete unused files to reduce clutter

**Editing Efficiently:**
- Zoom in for precise trims
- Use frame stepping for accuracy
- Group related clips on same track

**Before Export:**
- Watch full timeline in preview
- Check audio levels *(when supported)*
- Verify no gaps between clips

---

### Keyboard Shortcuts

**Current Shortcuts:**
- Mouse wheel: Zoom timeline
- Shift + Mouse wheel: Scroll timeline

**Planned Shortcuts:**
- Space: Play/Pause
- Left/Right Arrow: Frame step
- Delete: Remove selected clip
- Cmd/Ctrl + Z: Undo
- Cmd/Ctrl + S: Save project
- Cmd/Ctrl + O: Open project
- Cmd/Ctrl + E: Export

See [keyboard-shortcuts.md](keyboard-shortcuts.md) for full list.

---

## Troubleshooting

### Common Issues

**FFmpeg not found:**
- Install FFmpeg (see "Getting Started")
- Verify with `ffmpeg -version`
- Ensure it's in system PATH

**Screen recording permission denied (macOS):**
- System Settings > Privacy & Security > Screen Recording
- Enable ClipForge
- Restart app

**Video won't import:**
- Check file format is supported
- Verify file isn't corrupted
- Ensure sufficient disk space
- Check file permissions

**Timeline lag:**
- Reduce clip count
- Close other apps
- Lower preview quality *(planned)*
- Update graphics drivers

**Export fails:**
- Check output path is writable
- Ensure enough disk space
- Verify all media files exist
- Check FFmpeg is installed

**Memory warnings:**
- Restart ClipForge
- Close other apps
- Clear preview cache
- Reduce timeline complexity

For detailed troubleshooting, see [troubleshooting.md](troubleshooting.md).

---

## Getting Help

**Documentation:**
- [Quickstart Tutorial](quickstart.md)
- [Troubleshooting Guide](troubleshooting.md)
- [API Reference](api-reference.md)

**Community:**
- GitHub Issues: Report bugs
- GitHub Discussions: Ask questions
- Email: support@clipforge.dev *(if applicable)*

**Developer Docs:**
- [Technical Architecture](../clipforges/02-technical-architecture.md)
- [Module Specifications](../clipforges/)
- [CLAUDE.md](../CLAUDE.md)

---

**Last Updated:** October 28, 2025
**Version:** 0.1.0 MVP
**Next Update:** After v1.0 release
