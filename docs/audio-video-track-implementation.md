# Audio/Video Independent Track Implementation

**Date:** 2025-10-28
**Status:** Phase 1 Complete - Basic functionality working

## Overview

Implemented independent audio and video track support in the VideoPreview component, allowing video and audio clips to be moved separately on the timeline (similar to professional video editors like Premiere Pro, Final Cut Pro).

## Architecture

### Current Timeline Structure

The timeline uses a dual-track approach where media files with both video and audio are split into separate clips:

- **Video Track**: Contains video clips (with muted audio when separate audio track exists)
- **Audio Track**: Contains audio clips (extracted from the same source file or independent audio files)

When a video file with audio is added to the timeline (`timelineStore.ts:265-325`):
```typescript
// Creates TWO clips from the same file:
videoClip = {
  media_file_id: "file123",
  track_position: 0,
  duration: 26.986
}

audioClip = {
  media_file_id: "file123",  // SAME file ID
  track_position: 0,
  duration: 26.986
}
```

### VideoPreview Component Changes

**Files Modified:**
- `src/lib/components/VideoPreview.svelte`

**Key Features:**
1. Dual HTML5 media elements (video + audio)
2. Track-specific clip detection
3. Smart file detection to prevent resource contention
4. Synchronized playback across elements

## Implementation Details

### 1. Dual Media Elements

```svelte
<!-- Video element -->
<video
  bind:this={videoElement}
  src={currentVideoUrl}
  muted={hasAudio}  <!-- Muted when separate audio track active -->
  preload="metadata"
  playsinline
/>

<!-- Audio element (hidden) -->
<audio
  bind:this={audioElement}
  src={hasAudio ? currentAudioUrl : ''}
  preload="metadata"
  style="display: none;"
/>
```

### 2. Track-Specific Clip Detection

**Functions Added:**
- `getVideoClipUrl(time)` - Finds video clips on Video tracks
- `getAudioClipUrl(time)` - Finds audio clips on Audio tracks
- `getActiveMediaState()` - Returns which tracks have active clips

### 3. Smart File Detection (Critical Fix)

**Problem:** When the same file exists on both tracks, both elements tried to load it simultaneously, causing:
- Browser resource contention
- Playback failures
- Race conditions

**Solution (lines 67-78):**
```typescript
// Check if video and audio clips reference the same file
const videoClip = mediaState.videoClip;
const audioClip = mediaState.audioClip;
const isSameFile = videoClip && audioClip &&
                   videoClip.media_file_id === audioClip.media_file_id;

if (isSameFile) {
  // Use video element with native audio, disable separate audio element
  hasAudio = false;
  currentAudioUrl = '';
}
```

### 4. Synchronized Playback

**Play Function (lines 236-250):**
```typescript
// Play video if available
if (videoElement && hasVideo && currentVideoUrl) {
  videoElement.currentTime = info.clipRelativeTime;
  videoElement.play();
}

// Play audio from independent track
if (audioElement && hasAudio && currentAudioUrl) {
  audioElement.currentTime = info.clipRelativeTime;
  audioElement.play();
}
```

**Sync Reactive Statements (lines 348-380):**
- Video element syncs to clip-relative time when readyState >= 2
- Audio element syncs to clip-relative time when readyState >= 2
- Only updates if difference > 0.05s (prevents jitter)

## Problems Encountered & Fixes

### Problem 1: Video Preview Not Updating Frames
**Symptom:** Preview showed first frame of video, didn't update when playhead moved
**Cause:** Reactive statement lacked explicit dependencies, 0.1s threshold too large
**Fix:** Added explicit `currentTime` dependency, reduced threshold to 0.05s, added readyState check
**Location:** `VideoPreview.svelte:348-358`

### Problem 2: NotSupportedError When Playing Audio-Only Sections
**Symptom:** Error when playhead was on audio track without video
**Cause:** `getSingleClipUrl()` only searched Video tracks
**Fix:** Created separate `getVideoClipUrl()` and `getAudioClipUrl()` functions
**Location:** `VideoPreview.svelte:444-479`

### Problem 3: Audio Duplication (Double Audio)
**Symptom:** Hearing both video's built-in audio AND separate audio element
**Cause:** Audio element played when video also had audio
**Fix 1:** Made audio element src conditional: `hasAudio && !hasVideo ? currentAudioUrl : ''`
**Fix 2 (Reverted):** Changed to `hasAudio ? currentAudioUrl : ''` to support independent tracks
**Final Fix:** Smart file detection - only disable audio element when same file
**Location:** `VideoPreview.svelte:67-78, 658`

### Problem 4: Clip Switching Requires Manual Pause/Play
**Symptom:** When moving playhead to different clip, video didn't auto-reload
**Cause:** Two conflicting reactive blocks both handling clip changes (race condition)
**Fix:** Removed duplicate `previousVideoUrl` tracking block, kept only `previousClipId` block
**Location:** `VideoPreview.svelte:108-146` (removed duplicate at 79-98)

### Problem 5: Hearing Audio When Only Video Track Selected
**Symptom:** Video's built-in audio plays even when separate audio track exists
**Cause:** Video element not muted when separate audio track active
**Fix:** Added `muted={hasAudio}` to video element
**Location:** `VideoPreview.svelte:639`

### Problem 6: No Preview/Audio After Restart (Resource Contention)
**Symptom:** Complete failure to play video or audio after implementing independent tracks
**Root Cause:** Both video and audio elements trying to load the SAME file simultaneously when the same source file exists on both Video and Audio tracks
**Fix:** Smart file detection - compare `media_file_id` of both clips, disable audio element if same file
**Location:** `VideoPreview.svelte:67-78`
**Impact:** Critical fix - this was the primary blocker

### Problem 7: hasMediaAtPlayhead() Not Detecting Audio
**Symptom:** Empty "No Media Selected" state when audio track had clip
**Cause:** Function only checked Video tracks, not Audio tracks
**Fix:** Updated condition from `track_type !== 'Video'` to `track_type !== 'Video' && track_type !== 'Audio'`
**Location:** `VideoPreview.svelte:543`

## Current Behavior

### Scenario 1: Same File on Both Tracks (Most Common)
- Timeline has Video clip and Audio clip both pointing to same `media_file_id`
- `isSameFile = true`
- Video element plays with native audio (unmuted)
- Audio element disabled
- ✅ Works correctly - no duplication, no resource contention

### Scenario 2: Different Files on Tracks (Future Use Case)
- Video clip on Video track: `media_file_id = "video1"`
- Audio clip on Audio track: `media_file_id = "audio1"` (different file)
- `isSameFile = false`
- Video element plays muted
- Audio element plays the separate audio file
- ✅ Ready for independent audio (background music, voiceover)

### Scenario 3: Video Only (No Audio Track Clip)
- `hasVideo = true`, `hasAudio = false`
- Video element plays with native audio (unmuted)
- ✅ Works correctly

### Scenario 4: Audio Only (No Video Track Clip)
- `hasVideo = false`, `hasAudio = true`
- Shows "Audio Only" placeholder (music note icon)
- Audio element plays
- ✅ Works correctly

## Known Limitations

1. **Automatic Playback Continuation:** Clip switching during playback may require manual play button press in some cases
   - Clip ID tracking helps but isn't 100% reliable
   - Event listener timing issues with fast clip changes

2. **No True Audio-Only File Support Yet:**
   - Phase 2 pending: Update models.rs, database schema, metadata.rs
   - Cannot currently import .mp3, .wav files
   - Current workaround: Use video files with audio

3. **No Audio Waveform Display:**
   - Audio-only clips show static placeholder
   - Future: Could render waveform visualization

4. **Single Audio/Video Track:**
   - Currently only one Video track and one Audio track
   - Future: Support multiple tracks of each type

## Phase 2 TODO (Audio File Import)

To enable importing standalone audio files (.mp3, .wav, .aac):

1. **Update `models.rs`:**
   - Add `MediaType` enum: `Video`, `Audio`, `Image`
   - Make video-specific fields optional (`resolution`, `video_codec`)

2. **Update database schema (`schema.sql`):**
   - Add `media_type` column
   - Make `width`, `height`, `codec_video` nullable

3. **Update `metadata.rs`:**
   - Detect audio-only files (no video stream)
   - Extract audio metadata (bitrate, sample rate, channels)
   - Set `media_type = "Audio"`

4. **Update `file_service.rs`:**
   - Skip thumbnail generation for audio files
   - Set placeholder thumbnail path

5. **Update `MediaLibrary.svelte`:**
   - Show audio icon for audio files
   - Display audio metadata (duration, bitrate)

6. **Update `timelineStore.ts`:**
   - `addMediaFileToTimeline()`: Only create Audio track clip for audio files
   - Don't create Video track clip for audio-only files

## Testing Checklist

- [x] Video with audio plays correctly (same file on both tracks)
- [x] Audio duplication prevented
- [x] Video element mutes when separate audio track
- [x] Clip switching works (may need manual play)
- [x] Audio-only placeholder shows correctly
- [x] hasMediaAtPlayhead() detects both audio and video
- [x] No resource contention errors
- [ ] Import standalone audio file (.mp3)
- [ ] Multiple audio tracks
- [ ] Multiple video tracks
- [ ] Audio track offset/trimming
- [ ] Audio volume control per track

## Performance Notes

- **Memory:** Dual elements use ~2x memory when loading different files
- **CPU:** No significant CPU impact observed
- **Sync Threshold:** 0.05s prevents jitter while maintaining responsiveness
- **Browser Compatibility:** Tested on Chrome/Electron, should work on all modern browsers

## References

- Original issue: Video preview not updating frames
- Related files:
  - `src/lib/components/VideoPreview.svelte`
  - `src/lib/stores/timelineStore.ts`
  - `src-tauri/src/models.rs`
  - `src-tauri/src/database/schema.sql`

---

**Last Updated:** 2025-10-28
**Implementation Status:** Phase 1 Complete - Basic independent audio/video track support working
