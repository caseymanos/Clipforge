<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount, onDestroy } from 'svelte';
  import { timelineStore, playheadTime } from '../stores/timelineStore';
  import { mediaLibraryStore } from '../stores/mediaLibraryStore';
  import { convertFileSrc } from '@tauri-apps/api/core';

  // Props
  export let autoPlay = false;

  // Subscribe to stores for reactivity
  $: timeline = $timelineStore;

  // Keep raw file paths for backend FFmpeg commands
  $: mediaFiles = $mediaLibraryStore.reduce((acc, file) => {
    acc[file.id] = file.path;  // Keep original filesystem path
    return acc;
  }, {} as Record<string, string>);

  // Convert file paths to Tauri asset protocol URLs for HTML5 video playback
  $: mediaFileSrcs = $mediaLibraryStore.reduce((acc, file) => {
    acc[file.id] = convertFileSrc(file.path);
    return acc;
  }, {} as Record<string, string>);

  // State
  let videoElement: HTMLVideoElement;
  let audioElement: HTMLAudioElement;
  let isPlaying = false;
  let playbackSpeed = 1.0;
  let duration = 0;
  let previewImage = '';
  let isComposite = false;
  let animationFrameId: number;
  let pendingFrameRequest: object | null = null;
  let renderDebounceTimeout: number;

  // Audio/Video state tracking
  let currentVideoUrl = '';
  let currentAudioUrl = '';
  let hasVideo = false;
  let hasAudio = false;

  // Sync with timeline playhead
  $: currentTime = $playheadTime;

  // Determine if we need composite rendering
  $: {
    isComposite = hasMultipleClips(timeline, currentTime);
  }

  // Duration from timeline
  $: duration = timeline?.duration || 0;

  // Reactive URLs and media state
  $: {
    const _ = currentTime;
    const __ = timeline;
    const ___ = mediaFileSrcs;

    const mediaState = getActiveMediaState();
    hasVideo = mediaState.hasVideo;
    hasAudio = mediaState.hasAudio;
    currentVideoUrl = getVideoClipUrl(currentTime);
    currentAudioUrl = getAudioClipUrl(currentTime);

    // Check if video and audio clips reference the same file
    // If so, only use video element with native audio (prevent resource contention)
    const videoClip = mediaState.videoClip;
    const audioClip = mediaState.audioClip;
    const isSameFile = videoClip && audioClip &&
                       videoClip.media_file_id === audioClip.media_file_id;

    if (isSameFile) {
      // Same file: use video element with native audio, disable separate audio element
      hasAudio = false;
      currentAudioUrl = '';
    }

    console.log('[VideoPreview] Media state update:', {
      hasVideo,
      hasAudio,
      isSameFile: isSameFile || false,
      currentVideoUrl: currentVideoUrl ? 'present' : 'none',
      currentAudioUrl: currentAudioUrl ? 'present' : 'none',
      currentTime
    });
  }

  // Keep backward compatibility with videoUrl for existing code
  $: videoUrl = currentVideoUrl;

  // Reactive clip info for display and scrubber positioning
  // Explicitly reference dependencies for Svelte reactivity
  $: clipInfo = (() => {
    const _ = currentTime;           // Force reactivity on playhead changes
    const __ = timeline;              // Force reactivity on timeline changes
    const ___ = $mediaLibraryStore;  // Force reactivity on media library changes
    return getCurrentClipInfo();
  })();

  // Scrubber position (percentage within current clip, NOT timeline)
  $: scrubberPercent = clipInfo
    ? Math.min(100, Math.max(0, (clipInfo.clipRelativeTime / clipInfo.clipDuration) * 100))
    : 0;

  // Time display values (show clip time, not timeline time)
  $: displayTime = clipInfo?.clipRelativeTime || 0;
  $: displayDuration = clipInfo?.clipDuration || duration;

  // Debug logging for scrubber
  $: if (clipInfo) {
    console.log('[VideoPreview] Scrubber update:', {
      clipRelativeTime: clipInfo.clipRelativeTime,
      clipDuration: clipInfo.clipDuration,
      scrubberPercent,
      displayTime,
      displayDuration
    });
  }

  // Watch for clip changes and reload video
  let previousClipId: string | null = null;
  let shouldResumePlayback = false;

  $: {
    const currentClipId = clipInfo?.clip?.id || null;

    if (currentClipId !== previousClipId && currentClipId !== null && previousClipId !== null) {
      console.log('[VideoPreview] Clip changed from', previousClipId, 'to', currentClipId);

      if (videoElement && videoUrl && !isComposite) {
        console.log('[VideoPreview] Loading new clip, isPlaying:', isPlaying);

        // Store playback state BEFORE loading
        shouldResumePlayback = isPlaying;
        const targetTime = clipInfo?.clipRelativeTime || 0;

        // Set up listener BEFORE calling load()
        const handleLoaded = () => {
          console.log('[VideoPreview] Video loaded, seeking to:', targetTime, 'shouldResume:', shouldResumePlayback);
          videoElement.currentTime = targetTime;

          if (shouldResumePlayback) {
            // Use requestAnimationFrame to ensure DOM is ready
            requestAnimationFrame(() => {
              videoElement.play().catch(e => console.warn('Auto-play failed:', e));
            });
          }
        };

        videoElement.addEventListener('loadedmetadata', handleLoaded, { once: true });

        // Now trigger the load
        videoElement.load();
      }
    }

    previousClipId = currentClipId;
  }

  function hasMultipleClips(timeline: any, time: number): boolean {
    if (!timeline?.tracks) return false;

    let activeClipCount = 0;
    for (const track of timeline.tracks) {
      // Only check video tracks for composite detection
      if (track.track_type !== 'Video') continue;
      // Skip muted tracks
      if (track.muted) continue;

      for (const clip of track.clips) {
        const clipStart = clip.track_position;
        const clipEnd = clip.track_position + clip.duration;

        if (time >= clipStart && time < clipEnd) {
          activeClipCount++;
          if (activeClipCount > 1) return true;
        }
      }
    }

    return false;
  }

  async function renderFrame(time: number) {
    // Cancel any pending frame request to avoid race conditions
    pendingFrameRequest = null;

    // Create unique request ID for tracking
    const requestId = {};
    const frameRequest = (async () => {
      try {
        const base64Image = await invoke<string>('render_preview_frame', {
          timeline,
          time,
          mediaFiles,
        });

        // Only update if this request is still pending
        if (pendingFrameRequest === requestId) {
          previewImage = `data:image/jpeg;base64,${base64Image}`;
          pendingFrameRequest = null;
        }
      } catch (error) {
        console.error('Failed to render preview frame:', error);
        if (pendingFrameRequest === requestId) {
          pendingFrameRequest = null;
        }
      }
    })();

    pendingFrameRequest = requestId;
    await frameRequest;
  }

  function play() {
    isPlaying = true;

    if (isComposite) {
      // Composite mode: use animation frame loop
      const startTime = performance.now();
      const initialTime = currentTime;

      async function animate(currentAnimTime: number) {
        if (!isPlaying) return;

        const elapsed = (currentAnimTime - startTime) / 1000;
        const newTime = Math.min(initialTime + elapsed * playbackSpeed, duration);
        playheadTime.set(newTime);

        // Fire-and-forget: renderFrame manages its own race conditions
        renderFrame(newTime).catch(err =>
          console.error('Frame render failed:', err)
        );

        if (newTime >= duration) {
          pause();
          return;
        }

        animationFrameId = requestAnimationFrame(animate);
      }

      animationFrameId = requestAnimationFrame(animate);
    } else {
      // Simple mode: let HTML5 video/audio elements handle playback
      const info = getCurrentClipInfo();

      // Play video if available
      if (videoElement && hasVideo && currentVideoUrl) {
        if (info) {
          videoElement.currentTime = info.clipRelativeTime;
        }
        videoElement.play().catch(err => console.error('Video play error:', err));
      }

      // Play audio from independent audio track (video element is muted when hasAudio=true)
      if (audioElement && hasAudio && currentAudioUrl) {
        if (info) {
          audioElement.currentTime = info.clipRelativeTime;
        }
        audioElement.play().catch(err => console.error('Audio play error:', err));
      }
    }
  }

  function pause() {
    isPlaying = false;
    if (animationFrameId) {
      cancelAnimationFrame(animationFrameId);
    }
    // Pause both video and audio elements
    if (videoElement && !isComposite) {
      videoElement.pause();
    }
    if (audioElement && !isComposite) {
      audioElement.pause();
    }
  }

  function togglePlayPause() {
    if (isPlaying) {
      pause();
    } else {
      play();
    }
  }

  function seek(event: MouseEvent) {
    const currentClipInfo = getCurrentClipInfo();
    if (!currentClipInfo) return;  // No clip at playhead, can't seek

    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const x = event.clientX - rect.left;
    const percentage = Math.max(0, Math.min(1, x / rect.width));

    // Calculate new clip-relative time
    const newClipTime = percentage * currentClipInfo.clipDuration;

    // Convert back to timeline time
    // clipRelativeTime = (timelineTime - track_position) / speed + trim_start
    // So: timelineTime = (clipRelativeTime - trim_start) * speed + track_position
    const clip = currentClipInfo.clip;
    const offsetFromClipStart = (newClipTime - (clip.trim_start || 0)) * (clip.speed || 1.0);
    const newTimelineTime = clip.track_position + offsetFromClipStart;

    console.log('[VideoPreview] Seeking to', percentage * 100, '% of clip, timeline time:', newTimelineTime);

    playheadTime.set(newTimelineTime);

    if (isComposite) {
      renderFrame(newTimelineTime);
    } else if (videoElement) {
      videoElement.currentTime = newClipTime;
    }
  }

  function stepFrame(direction: number) {
    const frameDuration = 1 / (timeline?.framerate || 30);
    const newTime = Math.max(0, Math.min(duration, currentTime + direction * frameDuration));
    playheadTime.set(newTime);

    if (isComposite) {
      renderFrame(newTime);
    } else if (videoElement) {
      // Use clip-relative time, not timeline time
      const info = getCurrentClipInfo();
      if (info) {
        videoElement.currentTime = info.clipRelativeTime;
      }
    }
  }

  function setSpeed(speed: number) {
    playbackSpeed = speed;
    if (videoElement) {
      videoElement.playbackRate = speed;
    }
    if (audioElement) {
      audioElement.playbackRate = speed;
    }
  }

  // Update composite preview when time changes (debounced to prevent FFmpeg spam)
  $: if (isComposite && !isPlaying) {
    clearTimeout(renderDebounceTimeout);
    renderDebounceTimeout = setTimeout(() => {
      renderFrame(currentTime);
    }, 100); // 100ms debounce - only render after user stops scrubbing
  }

  // Sync video element to clip-relative time (not timeline time!)
  $: if (videoElement && !isComposite && clipInfo && hasVideo) {
    // Explicitly reference currentTime for reactivity
    const _ = currentTime;
    const targetTime = clipInfo.clipRelativeTime;

    // Only update if video is ready and difference is significant (prevents jitter)
    if (videoElement.readyState >= 2 && Math.abs(videoElement.currentTime - targetTime) > 0.05) {
      console.log('[VideoPreview] Syncing video to clip time:', targetTime);
      videoElement.currentTime = targetTime;
    }
  }

  // Sync audio element to clip-relative time
  $: if (audioElement && !isComposite && clipInfo && hasAudio) {
    // Explicitly reference currentTime for reactivity
    const _ = currentTime;
    const targetTime = clipInfo.clipRelativeTime;

    // Only update if audio is ready and difference is significant (prevents jitter)
    if (audioElement.readyState >= 2 && Math.abs(audioElement.currentTime - targetTime) > 0.05) {
      console.log('[VideoPreview] Syncing audio to clip time:', targetTime);
      audioElement.currentTime = targetTime;
    }
  }

  onMount(() => {
    if (autoPlay) {
      play();
    }
  });

  onDestroy(() => {
    pause();
  });

  function formatTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  // Get single clip URL for simple playback
  function getSingleClipUrl(): string {
    if (!timeline) {
      console.log('[getSingleClipUrl] No timeline');
      return '';
    }

    if (!timeline.tracks) {
      console.log('[getSingleClipUrl] Timeline exists but has no tracks property');
      console.log('[getSingleClipUrl] Timeline object:', timeline);
      return '';
    }

    if (timeline.tracks.length === 0) {
      console.log('[getSingleClipUrl] Timeline has empty tracks array');
      return '';
    }

    console.log(`[getSingleClipUrl] Looking for clip at playhead: ${currentTime}, timeline has ${timeline.tracks.length} tracks`);

    // Find the clip at current playhead position
    for (const track of timeline.tracks) {
      // Only check video tracks
      if (track.track_type !== 'Video') {
        console.log(`[getSingleClipUrl] Skipping non-video track: ${track.track_type}`);
        continue;
      }
      // Skip muted tracks
      if (track.muted) {
        console.log('[getSingleClipUrl] Skipping muted track');
        continue;
      }

      console.log(`[getSingleClipUrl] Checking video track with ${track.clips.length} clips`);

      for (const clip of track.clips) {
        const clipStart = clip.track_position;
        const clipEnd = clip.track_position + clip.duration;

        console.log(`[getSingleClipUrl] Checking clip ${clip.media_file_id} at ${clipStart}-${clipEnd}`);

        // Check if playhead is within this clip
        if (currentTime >= clipStart && currentTime < clipEnd) {
          const url = mediaFileSrcs[clip.media_file_id] || '';
          console.log(`[getSingleClipUrl] ✓ Found clip! media_file_id: ${clip.media_file_id}, URL exists: ${!!url}`);

          // Also log the media file info
          const mediaFile = $mediaLibraryStore.find(f => f.id === clip.media_file_id);
          if (mediaFile) {
            console.log(`[getSingleClipUrl] Media file: ${mediaFile.filename}, codec: ${mediaFile.codec?.video}`);
          }

          return url;
        }
      }
    }

    // No clip at playhead position
    console.log('[getSingleClipUrl] No clip found at playhead');
    return '';
  }

  // Get video clip URL at specific time
  function getVideoClipUrl(time: number): string {
    if (!timeline?.tracks) return '';

    for (const track of timeline.tracks) {
      if (track.track_type !== 'Video' || track.muted) continue;

      for (const clip of track.clips) {
        const clipStart = clip.track_position;
        const clipEnd = clip.track_position + clip.duration;

        if (time >= clipStart && time < clipEnd) {
          return mediaFileSrcs[clip.media_file_id] || '';
        }
      }
    }
    return '';
  }

  // Get audio clip URL at specific time
  function getAudioClipUrl(time: number): string {
    if (!timeline?.tracks) return '';

    for (const track of timeline.tracks) {
      if (track.track_type !== 'Audio' || track.muted) continue;

      for (const clip of track.clips) {
        const clipStart = clip.track_position;
        const clipEnd = clip.track_position + clip.duration;

        if (time >= clipStart && time < clipEnd) {
          return mediaFileSrcs[clip.media_file_id] || '';
        }
      }
    }
    return '';
  }

  // Get active media state at current playhead
  interface ActiveMediaState {
    hasVideo: boolean;
    hasAudio: boolean;
    videoClip: any | null;
    audioClip: any | null;
  }

  function getActiveMediaState(): ActiveMediaState {
    const state: ActiveMediaState = {
      hasVideo: false,
      hasAudio: false,
      videoClip: null,
      audioClip: null
    };

    if (!timeline?.tracks) return state;

    // Check video tracks
    for (const track of timeline.tracks) {
      if (track.track_type === 'Video' && !track.muted) {
        for (const clip of track.clips) {
          const clipStart = clip.track_position;
          const clipEnd = clip.track_position + clip.duration;

          if (currentTime >= clipStart && currentTime < clipEnd) {
            state.hasVideo = true;
            state.videoClip = clip;
            break;
          }
        }
      }
    }

    // Check audio tracks
    for (const track of timeline.tracks) {
      if (track.track_type === 'Audio' && !track.muted) {
        for (const clip of track.clips) {
          const clipStart = clip.track_position;
          const clipEnd = clip.track_position + clip.duration;

          if (currentTime >= clipStart && currentTime < clipEnd) {
            state.hasAudio = true;
            state.audioClip = clip;
            break;
          }
        }
      }
    }

    return state;
  }

  // Check if there's any media at current playhead position
  function hasMediaAtPlayhead(): boolean {
    if (!timeline?.tracks) {
      console.log('[VideoPreview] No timeline or tracks');
      return false;
    }

    for (const track of timeline.tracks) {
      // Check both video AND audio tracks
      if (track.track_type !== 'Video' && track.track_type !== 'Audio') continue;
      if (track.muted) continue;

      for (const clip of track.clips) {
        const clipStart = clip.track_position;
        const clipEnd = clip.track_position + clip.duration;

        console.log(`[VideoPreview] Checking ${track.track_type} clip at ${clipStart}-${clipEnd}, playhead: ${currentTime}`);

        if (currentTime >= clipStart && currentTime < clipEnd) {
          console.log(`[VideoPreview] Found ${track.track_type} media at playhead!`);
          return true;
        }
      }
    }

    console.log('[VideoPreview] No media at playhead');
    return false;
  }

  // Get the current clip at playhead position
  function getCurrentClip() {
    if (!timeline?.tracks) return null;

    for (const track of timeline.tracks) {
      if (track.track_type !== 'Video' || track.muted) continue;

      for (const clip of track.clips) {
        if (currentTime >= clip.track_position &&
            currentTime < clip.track_position + clip.duration) {
          return clip;
        }
      }
    }
    return null;
  }

  // Calculate time within the current clip (accounting for trim_start, speed)
  function getClipRelativeTime(clip: any, timelineTime: number): number {
    const offsetFromClipStart = timelineTime - clip.track_position;
    const speedAdjusted = offsetFromClipStart / (clip.speed || 1.0);
    return speedAdjusted + (clip.trim_start || 0);
  }

  // Get current clip info for debugging/display (enhanced with clip-relative time)
  function getCurrentClipInfo(): {
    clip: any;
    mediaFile: any;
    clipRelativeTime: number;
    clipDuration: number;
    clipId: string;
    fileName: string;
    codec: string;
  } | null {
    const clip = getCurrentClip();
    if (!clip) return null;

    const mediaFile = $mediaLibraryStore.find(f => f.id === clip.media_file_id);
    if (!mediaFile) return null;

    return {
      clip,
      mediaFile,
      clipRelativeTime: getClipRelativeTime(clip, currentTime),
      clipDuration: clip.duration,
      clipId: clip.media_file_id,
      fileName: mediaFile.path.split('/').pop() || 'Unknown',
      codec: mediaFile.codec?.video || 'Unknown'
    };
  }
</script>

<div class="video-preview">
  <div class="preview-container">
    {#if !hasMediaAtPlayhead()}
      <!-- No media at playhead - show empty state -->
      <div class="no-media-view">
        <div class="no-media-icon">📹</div>
        <div class="no-media-text">No Media Selected</div>
        <div class="no-media-hint">Move playhead over a clip to preview</div>
      </div>
    {:else if isComposite}
      <!-- Composite preview using rendered frames -->
      <div class="composite-preview">
        {#if previewImage}
          <img src={previewImage} alt="Preview frame" />
        {:else}
          <div class="blank-frame">Rendering preview...</div>
        {/if}
      </div>
    {:else}
      <!-- Single clip preview using HTML5 video and audio -->
      {#if hasVideo}
        <video
          bind:this={videoElement}
          src={currentVideoUrl}
          muted={hasAudio}
          preload="metadata"
          playsinline
          on:loadedmetadata={(e) => { duration = e.currentTarget.duration; }}
          on:error={(e) => console.error('Video error:', e)}
        >
          <track kind="captions" />
        </video>
      {:else if hasAudio}
        <!-- Audio-only view: Show placeholder -->
        <div class="audio-only-view">
          <div class="audio-icon">🎵</div>
          <div class="audio-text">Audio Only</div>
        </div>
      {/if}

      <!-- Hidden audio element for independent audio track playback -->
      <audio
        bind:this={audioElement}
        src={hasAudio ? currentAudioUrl : ''}
        preload="metadata"
        style="display: none;"
        on:error={(e) => console.error('Audio error:', e)}
      />
    {/if}

    <!-- Preview state indicators -->
    {#if hasMediaAtPlayhead()}
      <div class="preview-indicators">
        <div class="indicator mode-indicator" class:composite={isComposite}>
          {isComposite ? '🎬 Composite' : '▶️ Direct'}
        </div>
        {#if getCurrentClipInfo()}
          <div class="indicator codec-indicator">
            {getCurrentClipInfo()?.codec}
          </div>
          <div class="indicator file-indicator" title={getCurrentClipInfo()?.fileName}>
            📄 {getCurrentClipInfo()?.fileName}
          </div>
        {/if}
      </div>
    {/if}
  </div>

  <div class="controls">
    <button on:click={togglePlayPause} class="play-pause-btn">
      {isPlaying ? '⏸' : '▶'}
    </button>

    <button on:click={() => stepFrame(-1)} class="frame-step-btn" title="Previous frame">
      ⏮
    </button>

    <button on:click={() => stepFrame(1)} class="frame-step-btn" title="Next frame">
      ⏭
    </button>

    <div class="timeline-scrubber" on:click={seek} role="slider" tabindex="0">
      <div class="timeline-progress" style="width: {scrubberPercent}%" />
      <div class="timeline-handle" style="left: {scrubberPercent}%" />
    </div>

    <div class="time-display">
      {formatTime(displayTime)} / {formatTime(displayDuration)}
    </div>

    <div class="speed-controls">
      <button on:click={() => setSpeed(0.5)} class:active={playbackSpeed === 0.5}>
        0.5x
      </button>
      <button on:click={() => setSpeed(1.0)} class:active={playbackSpeed === 1.0}>
        1x
      </button>
      <button on:click={() => setSpeed(2.0)} class:active={playbackSpeed === 2.0}>
        2x
      </button>
    </div>
  </div>
</div>

<style>
  .video-preview {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    width: 100%;
    height: 100%;
    max-height: 600px;  /* Prevent container from growing beyond this */
  }

  .preview-container {
    flex: 1;
    background: #000;
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
    overflow: hidden;
    min-height: 400px;  /* Ensure minimum height */
  }

  video {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  .composite-preview {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .composite-preview img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
  }

  .blank-frame {
    color: #666;
    font-size: 1.2rem;
  }

  .audio-only-view {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
  }

  .audio-icon {
    font-size: 6rem;
    opacity: 0.7;
  }

  .audio-text {
    font-size: 1.5rem;
    color: #888;
    font-weight: 500;
  }

  .no-media-view {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    color: #666;
    text-align: center;
  }

  .no-media-icon {
    font-size: 4rem;
    opacity: 0.5;
  }

  .no-media-text {
    font-size: 1.5rem;
    font-weight: 500;
    color: #888;
  }

  .no-media-hint {
    font-size: 1rem;
    color: #666;
    font-style: italic;
  }

  .preview-indicators {
    position: absolute;
    top: 1rem;
    right: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    align-items: flex-end;
    pointer-events: none;
  }

  .indicator {
    padding: 0.5rem 0.75rem;
    background: rgba(0, 0, 0, 0.7);
    color: #fff;
    font-size: 0.85rem;
    font-family: monospace;
    border-radius: 0.25rem;
    backdrop-filter: blur(10px);
    white-space: nowrap;
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .mode-indicator {
    background: rgba(102, 126, 234, 0.8);
    font-weight: 600;
  }

  .mode-indicator.composite {
    background: rgba(234, 102, 126, 0.8);
  }

  .codec-indicator {
    background: rgba(52, 211, 153, 0.8);
    font-weight: 600;
    text-transform: uppercase;
  }

  .file-indicator {
    font-size: 0.75rem;
    background: rgba(0, 0, 0, 0.6);
  }

  .controls {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 1rem;
    background: #1a1a1a;
    border-radius: 0.5rem;
  }

  button {
    padding: 0.5rem 1rem;
    background: #333;
    color: #fff;
    border: none;
    border-radius: 0.25rem;
    cursor: pointer;
    font-size: 1rem;
    transition: background 0.2s;
  }

  button:hover {
    background: #444;
  }

  button:active {
    background: #555;
  }

  button.active {
    background: #667eea;
  }

  .play-pause-btn {
    font-size: 1.5rem;
    width: 3rem;
    height: 3rem;
  }

  .frame-step-btn {
    font-size: 1.2rem;
  }

  .timeline-scrubber {
    flex: 1;
    height: 2rem;
    background: #333;
    border-radius: 1rem;
    position: relative;
    cursor: pointer;
  }

  .timeline-progress {
    height: 100%;
    background: #667eea;
    border-radius: 1rem;
    transition: width 0.1s linear;
  }

  .timeline-handle {
    position: absolute;
    top: 50%;
    transform: translate(-50%, -50%);
    width: 1rem;
    height: 1rem;
    background: #fff;
    border-radius: 50%;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
  }

  .time-display {
    color: #fff;
    font-family: monospace;
    min-width: 7rem;
    text-align: center;
  }

  .speed-controls {
    display: flex;
    gap: 0.25rem;
  }

  .speed-controls button {
    padding: 0.5rem;
    min-width: 3rem;
  }
</style>
