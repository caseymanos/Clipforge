# ClipForge API Reference

**Tauri Command Reference for Frontend Developers**

Version: 0.1.0 (MVP)
Last Updated: October 28, 2025

---

## Overview

ClipForge uses [Tauri](https://tauri.app/) to provide a bridge between the Rust backend and Svelte frontend. All backend functionality is exposed through **Tauri commands** which can be invoked from JavaScript/TypeScript.

### Basic Usage

```typescript
import { invoke } from '@tauri-apps/api/core';

// Call a backend command
const result = await invoke('command_name', {
  parameter1: value1,
  parameter2: value2
});
```

---

## Table of Contents

1. [Application Commands](#application-commands)
2. [File & Media Commands](#file--media-commands)
3. [FFmpeg Commands](#ffmpeg-commands)
4. [Recording Commands](#recording-commands)
5. [Timeline Commands](#timeline-commands)
6. [Export Commands](#export-commands)
7. [Preview Commands](#preview-commands)
8. [Event System](#event-system)

---

## Application Commands

### get_app_version

Get the current application version.

**Parameters:** None

**Returns:** `string` - Version number (e.g., "0.1.0")

**Example:**
```typescript
const version = await invoke('get_app_version');
console.log('ClipForge version:', version);
// Output: "ClipForge version: 0.1.0"
```

---

### open_devtools

Open the browser developer tools (DevTools console).

**Parameters:** None

**Returns:** `void`

**Example:**
```typescript
await invoke('open_devtools');
// DevTools window opens
```

**Note:** Only works in development builds.

---

### close_splashscreen

Close the application splashscreen (if shown on startup).

**Parameters:** None

**Returns:** `void`

**Example:**
```typescript
await invoke('close_splashscreen');
```

---

## File & Media Commands

### import_media_file

Import a video file into the media library.

**Parameters:**
- `file_path: string` - Absolute path to video file

**Returns:** `MediaFile` object

**Example:**
```typescript
interface MediaFile {
  id: string;
  path: string;
  filename: string;
  duration: number;
  resolution: { width: number; height: number };
  codec: { video: string; audio: string };
  file_size: number;
  thumbnail_path: string | null;
  hash: string;
  imported_at: string;
}

const mediaFile = await invoke('import_media_file', {
  file_path: '/Users/name/Videos/clip.mp4'
});

console.log('Imported:', mediaFile.filename);
console.log('Duration:', mediaFile.duration, 'seconds');
console.log('Resolution:', `${mediaFile.resolution.width}x${mediaFile.resolution.height}`);
```

**Errors:**
- File not found
- Unsupported format
- Metadata extraction failed
- Duplicate file (returns existing)

---

### get_media_library

Get all imported media files.

**Parameters:** None

**Returns:** `MediaFile[]` - Array of all media files

**Example:**
```typescript
const library = await invoke('get_media_library');
console.log(`Library has ${library.length} files`);

library.forEach(file => {
  console.log(`- ${file.filename} (${file.duration}s)`);
});
```

---

### get_media_file

Get a specific media file by ID.

**Parameters:**
- `file_id: string` - UUID of the media file

**Returns:** `MediaFile | null`

**Example:**
```typescript
const file = await invoke('get_media_file', {
  file_id: 'abc-123-def-456'
});

if (file) {
  console.log('Found:', file.filename);
} else {
  console.log('File not found');
}
```

---

### delete_media_file

Remove a media file from the library.

**Parameters:**
- `file_id: string` - UUID of the media file

**Returns:** `void`

**Example:**
```typescript
await invoke('delete_media_file', {
  file_id: 'abc-123-def-456'
});

console.log('File deleted from library');
```

**Note:** Original file on disk is NOT deleted. Only removed from ClipForge's database.

---

### get_file_metadata

Extract metadata from a video file without importing it.

**Parameters:**
- `file_path: string` - Absolute path to video file

**Returns:** `FileMetadata` object

**Example:**
```typescript
interface FileMetadata {
  duration: number;
  resolution: { width: number; height: number };
  codec: { video: string; audio: string };
  bitrate: number;
  framerate: number;
  has_audio: boolean;
}

const metadata = await invoke('get_file_metadata', {
  file_path: '/path/to/video.mp4'
});

console.log('Video info:');
console.log('Duration:', metadata.duration, 'seconds');
console.log('Resolution:', `${metadata.resolution.width}x${metadata.resolution.height}`);
console.log('Codec:', metadata.codec.video);
console.log('Framerate:', metadata.framerate, 'fps');
```

---

### generate_thumbnail

Generate a thumbnail image for a video file.

**Parameters:**
- `file_path: string` - Absolute path to video file
- `timestamp: number` - Time in seconds to extract frame

**Returns:** `string` - Path to generated thumbnail

**Example:**
```typescript
const thumbnailPath = await invoke('generate_thumbnail', {
  file_path: '/path/to/video.mp4',
  timestamp: 5.0  // Extract frame at 5 seconds
});

console.log('Thumbnail saved to:', thumbnailPath);
// Use in <img> tag: <img src={convertFileSrc(thumbnailPath)} />
```

---

### generate_thumbnail_sequence

Generate multiple thumbnails from a video (for timeline scrubbing).

**Parameters:**
- `file_path: string` - Absolute path to video file
- `count: number` - Number of thumbnails to generate
- `start_time: number` - Start time in seconds (default: 0)
- `end_time: number` - End time in seconds (default: video duration)

**Returns:** `string[]` - Array of thumbnail paths

**Example:**
```typescript
const thumbnails = await invoke('generate_thumbnail_sequence', {
  file_path: '/path/to/video.mp4',
  count: 10,
  start_time: 0,
  end_time: 60
});

console.log(`Generated ${thumbnails.length} thumbnails`);
// Use for scrubber preview bar
```

---

## FFmpeg Commands

### trim_video_clip

Trim a video clip (extract segment).

**Parameters:**
- `input_path: string` - Input video file path
- `output_path: string` - Output video file path
- `start_time: number` - Start time in seconds
- `duration: number` - Duration in seconds

**Returns:** `void`

**Example:**
```typescript
await invoke('trim_video_clip', {
  input_path: '/path/to/input.mp4',
  output_path: '/path/to/output.mp4',
  start_time: 10.0,
  duration: 30.0
});

console.log('Trimmed video: 30 seconds from 0:10');
```

**Note:** Re-encodes video for frame-accurate trimming.

---

### concatenate_clips

Concatenate multiple video clips into one file.

**Parameters:**
- `input_paths: string[]` - Array of input video file paths
- `output_path: string` - Output video file path

**Returns:** `void`

**Example:**
```typescript
await invoke('concatenate_clips', {
  input_paths: [
    '/path/to/clip1.mp4',
    '/path/to/clip2.mp4',
    '/path/to/clip3.mp4'
  ],
  output_path: '/path/to/combined.mp4'
});

console.log('Concatenated 3 clips');
```

**Note:** Uses fast concat (no re-encoding) if codecs match.

---

### extract_video_frame

Extract a single frame as an image.

**Parameters:**
- `input_path: string` - Input video file path
- `output_path: string` - Output image file path
- `timestamp: number` - Time in seconds

**Returns:** `void`

**Example:**
```typescript
await invoke('extract_video_frame', {
  input_path: '/path/to/video.mp4',
  output_path: '/path/to/frame.jpg',
  timestamp: 15.5
});

console.log('Extracted frame at 15.5 seconds');
```

---

### apply_video_filter

Apply FFmpeg video filter to a clip.

**Parameters:**
- `input_path: string` - Input video file path
- `output_path: string` - Output video file path
- `filter: string` - FFmpeg filter string

**Returns:** `void`

**Example:**
```typescript
// Apply grayscale filter
await invoke('apply_video_filter', {
  input_path: '/path/to/input.mp4',
  output_path: '/path/to/output.mp4',
  filter: 'hue=s=0'
});

// Apply brightness adjustment
await invoke('apply_video_filter', {
  input_path: '/path/to/input.mp4',
  output_path: '/path/to/output.mp4',
  filter: 'eq=brightness=0.2:contrast=1.2'
});
```

**Common filters:**
- `hue=s=0` - Grayscale
- `eq=brightness=0.2` - Increase brightness
- `boxblur=2:1` - Blur effect
- `unsharp=5:5:1.0` - Sharpen

---

## Recording Commands

### list_recording_sources

List available screen/window recording sources.

**Parameters:** None

**Returns:** `RecordingSource[]`

**Example:**
```typescript
interface RecordingSource {
  id: string;
  name: string;
  source_type: 'Screen' | 'Window' | 'Application';
}

const sources = await invoke('list_recording_sources');
console.log('Available sources:');
sources.forEach(source => {
  console.log(`- ${source.name} (${source.source_type})`);
});
```

**Note:** macOS only in v0.1.0. Returns empty array on Windows/Linux.

---

### check_recording_permissions

Check if screen recording permission is granted.

**Parameters:** None

**Returns:** `boolean`

**Example:**
```typescript
const hasPermission = await invoke('check_recording_permissions');

if (!hasPermission) {
  console.warn('Screen recording permission required');
  // Prompt user to grant permission
}
```

---

### request_recording_permissions

Request screen recording permission from the OS.

**Parameters:** None

**Returns:** `boolean` - True if granted

**Example:**
```typescript
const granted = await invoke('request_recording_permissions');

if (granted) {
  console.log('Permission granted');
} else {
  console.log('Permission denied - user must enable in System Settings');
}
```

---

### start_recording

Start screen recording.

**Parameters:**
- `source_id: string` - ID from `list_recording_sources`
- `output_path: string` - Where to save recording

**Returns:** `void`

**Example:**
```typescript
const sources = await invoke('list_recording_sources');
const mainDisplay = sources[0];

await invoke('start_recording', {
  source_id: mainDisplay.id,
  output_path: '/Users/name/Desktop/recording.mp4'
});

console.log('Recording started');
```

---

### stop_recording

Stop the current recording.

**Parameters:** None

**Returns:** `string` - Path to saved recording

**Example:**
```typescript
const filePath = await invoke('stop_recording');
console.log('Recording saved to:', filePath);

// Optionally import it
await invoke('import_media_file', { file_path: filePath });
```

---

### get_recording_state

Get current recording state.

**Parameters:** None

**Returns:** `'Idle' | 'Recording' | 'Paused' | 'Failed'`

**Example:**
```typescript
const state = await invoke('get_recording_state');

if (state === 'Recording') {
  console.log('Currently recording');
}
```

---

### get_recording_duration

Get duration of current recording in seconds.

**Parameters:** None

**Returns:** `number`

**Example:**
```typescript
const duration = await invoke('get_recording_duration');
console.log(`Recording duration: ${duration.toFixed(1)}s`);
```

---

## Timeline Commands

### create_timeline

Create a new timeline.

**Parameters:**
- `name: string` - Timeline name
- `framerate: number` - Frames per second (e.g., 30.0)
- `width: number` - Resolution width
- `height: number` - Resolution height

**Returns:** `Timeline` object

**Example:**
```typescript
const timeline = await invoke('create_timeline', {
  name: 'My Project',
  framerate: 30.0,
  width: 1920,
  height: 1080
});

console.log('Created timeline:', timeline.id);
```

---

### get_current_timeline

Get the active timeline.

**Parameters:** None

**Returns:** `Timeline | null`

**Example:**
```typescript
const timeline = await invoke('get_current_timeline');

if (timeline) {
  console.log('Timeline:', timeline.name);
  console.log('Duration:', timeline.duration, 'seconds');
  console.log('Tracks:', timeline.tracks.length);
}
```

---

### add_track

Add a new track to the timeline.

**Parameters:**
- `track_type: 'Video' | 'Audio' | 'Overlay'`

**Returns:** `string` - Track ID

**Example:**
```typescript
const trackId = await invoke('add_track', {
  track_type: 'Video'
});

console.log('Added video track:', trackId);
```

---

### remove_track

Remove a track from the timeline.

**Parameters:**
- `track_id: string`

**Returns:** `void`

**Example:**
```typescript
await invoke('remove_track', {
  track_id: 'track-abc-123'
});

console.log('Track removed');
```

---

### add_clip_to_timeline

Add a clip to the timeline.

**Parameters:**
- `media_file_id: string` - ID of imported media file
- `track_id: string` - Target track ID
- `position: number` - Time position in seconds

**Returns:** `string` - Clip ID

**Example:**
```typescript
const clipId = await invoke('add_clip_to_timeline', {
  media_file_id: 'file-abc-123',
  track_id: 'track-def-456',
  position: 0.0
});

console.log('Added clip:', clipId);
```

---

### remove_clip_from_timeline

Remove a clip from the timeline.

**Parameters:**
- `clip_id: string`

**Returns:** `void`

**Example:**
```typescript
await invoke('remove_clip_from_timeline', {
  clip_id: 'clip-abc-123'
});
```

---

### move_clip_on_timeline

Move a clip to a new position (same or different track).

**Parameters:**
- `clip_id: string`
- `new_track_id: string`
- `new_position: number` - Time in seconds

**Returns:** `void`

**Example:**
```typescript
await invoke('move_clip_on_timeline', {
  clip_id: 'clip-abc-123',
  new_track_id: 'track-def-456',
  new_position: 15.5
});

console.log('Clip moved to 15.5 seconds');
```

---

### trim_clip_on_timeline

Adjust a clip's trim points and duration.

**Parameters:**
- `clip_id: string`
- `new_trim_start: number` - Trim from start in seconds
- `new_trim_end: number` - Trim from end in seconds

**Returns:** `void`

**Example:**
```typescript
await invoke('trim_clip_on_timeline', {
  clip_id: 'clip-abc-123',
  new_trim_start: 2.0,  // Skip first 2 seconds
  new_trim_end: 1.0     // Skip last 1 second
});
```

---

### split_clip_at_time

Split a clip into two clips at the specified time.

**Parameters:**
- `clip_id: string`
- `split_time: number` - Time relative to clip start

**Returns:** `{ first_id: string, second_id: string }`

**Example:**
```typescript
const result = await invoke('split_clip_at_time', {
  clip_id: 'clip-abc-123',
  split_time: 5.0  // Split at 5 seconds into the clip
});

console.log('Split into:', result.first_id, result.second_id);
```

---

### get_clips_at_playhead

Get all clips at a specific timeline position.

**Parameters:**
- `time: number` - Timeline time in seconds

**Returns:** `Clip[]`

**Example:**
```typescript
const clips = await invoke('get_clips_at_playhead', {
  time: 10.5
});

console.log(`${clips.length} clips at 10.5 seconds`);
```

---

### save_timeline_project

Save timeline to a project file (.cfp).

**Parameters:**
- `file_path: string` - Path to save project file

**Returns:** `void`

**Example:**
```typescript
await invoke('save_timeline_project', {
  file_path: '/Users/name/Documents/my-project.cfp'
});

console.log('Project saved');
```

---

### load_timeline_project

Load timeline from a project file.

**Parameters:**
- `file_path: string` - Path to project file

**Returns:** `Timeline`

**Example:**
```typescript
const timeline = await invoke('load_timeline_project', {
  file_path: '/Users/name/Documents/my-project.cfp'
});

console.log('Loaded project:', timeline.name);
```

---

## Export Commands

### export_timeline

Export the timeline to a video file.

**Parameters:**
- `timeline: Timeline` - Timeline object
- `settings: ExportSettings` - Export configuration
- `output_path: string` - Where to save export
- `media_files_map: Record<string, MediaFile>` - Map of file IDs to MediaFile objects

**Returns:** `string` - Path to exported file

**Example:**
```typescript
const presets = await invoke('get_export_presets');
const youtubePreset = presets.find(p => p[0] === 'YouTube 1080p')[1];

const result = await invoke('export_timeline', {
  timeline: currentTimeline,
  settings: youtubePreset,
  output_path: '/Users/name/Desktop/video.mp4',
  media_files_map: mediaFilesObject
});

console.log('Export complete:', result);
```

**Note:** Emits 'export-progress' events during export.

---

### cancel_export

Cancel an ongoing export.

**Parameters:** None

**Returns:** `void`

**Example:**
```typescript
await invoke('cancel_export');
console.log('Export cancelled');
```

---

### get_export_presets

Get available export presets.

**Parameters:** None

**Returns:** `Array<[string, ExportSettings]>` - Tuples of (name, settings)

**Example:**
```typescript
const presets = await invoke('get_export_presets');

presets.forEach(([name, settings]) => {
  console.log(`- ${name}: ${settings.resolution.width}x${settings.resolution.height} @ ${settings.video_bitrate}kbps`);
});

// Output:
// - YouTube 1080p: 1920x1080 @ 8000kbps
// - Instagram Post: 1080x1080 @ 5000kbps
// - Twitter Video: 1280x720 @ 6000kbps
```

---

### validate_timeline_for_export

Check if timeline is ready to export.

**Parameters:**
- `timeline: Timeline`
- `media_files_map: Record<string, MediaFile>`

**Returns:** `boolean`

**Example:**
```typescript
const isValid = await invoke('validate_timeline_for_export', {
  timeline: currentTimeline,
  media_files_map: mediaFilesObject
});

if (!isValid) {
  console.error('Timeline validation failed');
}
```

---

## Preview Commands

### render_preview_frame

Render a single frame from the timeline (for composite preview).

**Parameters:**
- `timeline: Timeline`
- `time: number` - Time in seconds
- `media_files: Record<string, string>` - Map of file IDs to file paths

**Returns:** `string` - Base64-encoded JPEG image

**Example:**
```typescript
const base64Image = await invoke('render_preview_frame', {
  timeline: currentTimeline,
  time: 10.5,
  media_files: mediaFilesPathMap
});

// Display in <img> tag
const imgSrc = `data:image/jpeg;base64,${base64Image}`;
```

---

### clear_preview_cache

Clear the preview frame cache.

**Parameters:** None

**Returns:** `void`

**Example:**
```typescript
await invoke('clear_preview_cache');
console.log('Preview cache cleared');
```

**Use when:** Memory usage is high or preview seems stale.

---

### get_cache_stats

Get statistics about the preview cache.

**Parameters:** None

**Returns:** `{ size: number, capacity: number }`

**Example:**
```typescript
const stats = await invoke('get_cache_stats');
console.log(`Cache: ${stats.size}/${stats.capacity} frames`);
```

---

## Event System

ClipForge emits events for long-running operations.

### Listening to Events

```typescript
import { listen } from '@tauri-apps/api/event';

// Listen for export progress
const unlisten = await listen('export-progress', (event) => {
  const progress = event.payload;
  console.log(`Export: ${progress.percentage.toFixed(1)}%`);
  console.log(`FPS: ${progress.fps}, Remaining: ${progress.time_remaining_secs}s`);
});

// Stop listening
unlisten();
```

### Available Events

**export-progress**
- Emitted during video export
- Payload: `{ percentage: number, current_frame: number, fps: number, time_remaining_secs: number }`

**export-complete**
- Emitted when export finishes
- Payload: `null`

**import-progress** *(Planned)*
- Emitted during file import
- Payload: `{ file_name: string, progress: number }`

---

## Error Handling

All commands can throw errors. Use try-catch:

```typescript
try {
  const file = await invoke('import_media_file', {
    file_path: '/path/to/video.mp4'
  });
  console.log('Import success');
} catch (error) {
  console.error('Import failed:', error);
  // Show error to user
}
```

**Common error types:**
- File not found
- Permission denied
- Invalid parameters
- FFmpeg errors
- Database errors

---

## Type Definitions

Full TypeScript type definitions:

```typescript
// Located in: src/lib/stores/timelineStore.ts

interface Timeline {
  id: string;
  name: string;
  framerate: number;
  resolution: Resolution;
  tracks: Track[];
  duration: number;
}

interface Track {
  id: string;
  track_type: 'Video' | 'Audio' | 'Overlay';
  clips: Clip[];
  muted: boolean;
  locked: boolean;
}

interface Clip {
  id: string;
  media_file_id: string;
  name: string | null;
  track_position: number;
  duration: number;
  trim_start: number;
  trim_end: number;
  effects: Effect[];
  volume: number;
  speed: number;
}

interface Effect {
  id: string;
  effect_type: EffectType;
  enabled: boolean;
}

type EffectType =
  | { type: 'Brightness'; value: number }
  | { type: 'Contrast'; value: number }
  | { type: 'Saturation'; value: number }
  | { type: 'Blur'; radius: number }
  | { type: 'Sharpen'; amount: number }
  | { type: 'Normalize' }
  | { type: 'FadeIn'; duration: number }
  | { type: 'FadeOut'; duration: number };
```

---

## Related Documentation

- [User Guide](user-guide.md) - How to use ClipForge
- [Technical Architecture](../clipforges/02-technical-architecture.md) - System design
- [Data Structures](../clipforges/data-structures.md) - Type definitions
- [Module Specifications](../clipforges/) - Detailed implementation docs

---

**Last Updated:** October 28, 2025
**Version:** 0.1.0 MVP
**Rust API Docs:** Generate with `cargo doc --open` in `src-tauri/`
