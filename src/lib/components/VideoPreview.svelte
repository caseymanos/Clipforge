<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount, onDestroy } from 'svelte';
  import { timelineStore } from '../stores/timelineStore';
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
  let currentTime = 0;
  let isPlaying = false;
  let playbackSpeed = 1.0;
  let duration = 0;
  let previewImage = '';
  let isComposite = false;
  let animationFrameId: number;
  let pendingFrameRequest: object | null = null;

  // Determine if we need composite rendering
  $: {
    isComposite = hasMultipleClips(timeline, currentTime);
  }

  // Duration from timeline
  $: duration = timeline?.duration || 0;

  function hasMultipleClips(timeline: any, time: number): boolean {
    if (!timeline?.tracks) return false;

    let activeClipCount = 0;
    for (const track of timeline.tracks) {
      // Backend uses 'muted', and we skip muted tracks
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
        currentTime = Math.min(initialTime + elapsed * playbackSpeed, duration);

        // Fire-and-forget: renderFrame manages its own race conditions
        renderFrame(currentTime).catch(err =>
          console.error('Frame render failed:', err)
        );

        if (currentTime >= duration) {
          pause();
          return;
        }

        animationFrameId = requestAnimationFrame(animate);
      }

      animationFrameId = requestAnimationFrame(animate);
    } else if (videoElement) {
      // Simple mode: let HTML5 video handle playback
      videoElement.play();
    }
  }

  function pause() {
    isPlaying = false;
    if (animationFrameId) {
      cancelAnimationFrame(animationFrameId);
    }
    if (videoElement && !isComposite) {
      videoElement.pause();
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
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const x = event.clientX - rect.left;
    const percentage = x / rect.width;
    currentTime = percentage * duration;

    if (isComposite) {
      renderFrame(currentTime);
    } else if (videoElement) {
      videoElement.currentTime = currentTime;
    }
  }

  function stepFrame(direction: number) {
    const frameDuration = 1 / (timeline?.framerate || 30);
    currentTime = Math.max(0, Math.min(duration, currentTime + direction * frameDuration));

    if (isComposite) {
      renderFrame(currentTime);
    } else if (videoElement) {
      videoElement.currentTime = currentTime;
    }
  }

  function setSpeed(speed: number) {
    playbackSpeed = speed;
    if (videoElement) {
      videoElement.playbackRate = speed;
    }
  }

  // Update composite preview when time changes
  $: if (isComposite && !isPlaying) {
    renderFrame(currentTime);
  }

  // Sync video element time
  $: if (videoElement && !isComposite) {
    videoElement.currentTime = currentTime;
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
    if (!timeline?.tracks?.[0]?.clips?.[0]) return '';
    const clipId = timeline.tracks[0].clips[0].media_file_id;
    return mediaFileSrcs[clipId] || '';  // Use asset:// URLs for video element
  }
</script>

<div class="video-preview">
  <div class="preview-container">
    {#if isComposite}
      <!-- Composite preview using rendered frames -->
      <div class="composite-preview">
        {#if previewImage}
          <img src={previewImage} alt="Preview frame" />
        {:else}
          <div class="blank-frame">No preview available</div>
        {/if}
      </div>
    {:else}
      <!-- Single clip preview using HTML5 video -->
      <video
        bind:this={videoElement}
        src={getSingleClipUrl()}
        on:timeupdate={(e) => { currentTime = e.currentTarget.currentTime; }}
        on:loadedmetadata={(e) => { duration = e.currentTarget.duration; }}
      >
        <track kind="captions" />
      </video>
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
      <div class="timeline-progress" style="width: {(currentTime / duration) * 100}%" />
      <div class="timeline-handle" style="left: {(currentTime / duration) * 100}%" />
    </div>

    <div class="time-display">
      {formatTime(currentTime)} / {formatTime(duration)}
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
  }

  .preview-container {
    flex: 1;
    background: #000;
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
    overflow: hidden;
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
