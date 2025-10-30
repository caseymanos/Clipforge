<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import Konva from 'konva';
  import ConfirmDialog from './ConfirmDialog.svelte';
  import {
    timelineStore,
    playheadTime,
    selectedClipId,
    pixelsPerSecond,
    scrollOffset,
    moveClip,
    selectClip,
    updateClipDuration,
    addMediaFileToTimeline,
    removeClip,
    type Timeline,
    type Track,
    type Clip
  } from '../stores/timelineStore';
  import { mediaLibraryStore } from '../stores/mediaLibraryStore';

  // Props
  export let width = 1200;
  export let height = 400;

  // Konva objects
  let container: HTMLDivElement;
  let stage: Konva.Stage;
  let backgroundLayer: Konva.Layer;
  let trackLayer: Konva.Layer;
  let playheadLayer: Konva.Layer;

  // Constants
  const TRACK_HEIGHT = 80;
  const TRACK_PADDING = 10;
  const RULER_HEIGHT = 30;
  const CLIP_COLORS = {
    Video: '#667eea',
    Audio: '#48bb78',
    Overlay: '#ed8936'
  };

  let currentTimeline: Timeline;
  let currentPixelsPerSecond: number;
  let currentScrollOffset: number;
  let currentPlayheadTime: number;
  let currentSelectedClipId: string | null;

  // Confirm dialog state
  let showConfirmDialog = false;
  let confirmMessage = '';
  let confirmDialogX = 0;
  let confirmDialogY = 0;
  let pendingDeleteClipId: string | null = null;

  // Subscribe to stores
  const unsubTimeline = timelineStore.subscribe(value => { currentTimeline = value; renderTimeline(); });
  const unsubPPS = pixelsPerSecond.subscribe(value => { currentPixelsPerSecond = value; renderTimeline(); });
  const unsubScroll = scrollOffset.subscribe(value => { currentScrollOffset = value; renderTimeline(); });
  const unsubPlayhead = playheadTime.subscribe(value => { currentPlayheadTime = value; renderPlayhead(); });
  const unsubSelected = selectedClipId.subscribe(value => { currentSelectedClipId = value; renderTimeline(); });

  onMount(() => {
    // Initialize Konva stage
    stage = new Konva.Stage({
      container: container,
      width: width,
      height: height,
      draggable: false,
    });

    // Create layers
    backgroundLayer = new Konva.Layer();
    trackLayer = new Konva.Layer();
    playheadLayer = new Konva.Layer();

    stage.add(backgroundLayer);
    stage.add(trackLayer);
    stage.add(playheadLayer);

    // Initial render
    renderBackground();
    renderTimeline();
    renderPlayhead();

    // Mouse wheel zoom
    stage.on('wheel', handleWheel);

    // Click to deselect
    stage.on('click', (e) => {
      if (e.target === stage) {
        selectClip(null);
      }
    });
  });

  onDestroy(() => {
    unsubTimeline();
    unsubPPS();
    unsubScroll();
    unsubPlayhead();
    unsubSelected();

    if (stage) {
      stage.destroy();
    }
  });

  function renderBackground() {
    if (!backgroundLayer || !currentTimeline || !currentTimeline.tracks) return;

    backgroundLayer.destroyChildren();

    // Draw ruler background
    const rulerBg = new Konva.Rect({
      x: 0,
      y: 0,
      width: width,
      height: RULER_HEIGHT,
      fill: '#1a1a1a',
    });
    backgroundLayer.add(rulerBg);

    // Draw track backgrounds
    currentTimeline.tracks.forEach((track, index) => {
      const y = RULER_HEIGHT + index * (TRACK_HEIGHT + TRACK_PADDING);

      const trackBg = new Konva.Rect({
        x: 0,
        y: y,
        width: width,
        height: TRACK_HEIGHT,
        fill: '#2d2d2d',
        stroke: '#3d3d3d',
        strokeWidth: 1,
      });
      backgroundLayer.add(trackBg);

      // Track label
      const label = new Konva.Text({
        x: 10,
        y: y + TRACK_HEIGHT / 2 - 10,
        text: `${track.track_type} Track`,
        fontSize: 14,
        fill: '#ffffff',
        opacity: 0.7,
      });
      backgroundLayer.add(label);
    });

    backgroundLayer.batchDraw();
  }

  function renderTimeline() {
    if (!trackLayer || !currentTimeline || !currentTimeline.tracks) return;

    trackLayer.destroyChildren();

    // Render clips for each track
    currentTimeline.tracks.forEach((track, trackIndex) => {
      const trackY = RULER_HEIGHT + trackIndex * (TRACK_HEIGHT + TRACK_PADDING);

      track.clips.forEach(clip => {
        renderClip(clip, track, trackY);
      });
    });

    trackLayer.batchDraw();
  }

  function renderClip(clip: Clip, track: Track, trackY: number) {
    const clipX = (clip.track_position * currentPixelsPerSecond) - currentScrollOffset;
    const clipWidth = clip.duration * currentPixelsPerSecond;
    const isSelected = clip.id === currentSelectedClipId;

    // Clip rectangle
    const clipRect = new Konva.Rect({
      x: clipX,
      y: trackY + 5,
      width: clipWidth,
      height: TRACK_HEIGHT - 10,
      fill: CLIP_COLORS[track.track_type] || '#667eea',
      stroke: isSelected ? '#ffffff' : '#555555',
      strokeWidth: isSelected ? 3 : 1,
      cornerRadius: 4,
      draggable: !track.locked,
      name: clip.id,
    });

    // Clip text label
    const clipText = new Konva.Text({
      x: clipX + 8,
      y: trackY + 15,
      text: clip.name || `Clip ${clip.id.substring(0, 8)}`,  // Use filename if available, fallback to ID
      fontSize: 12,
      fill: '#ffffff',
      listening: false,
    });

    // Drag event handlers
    clipRect.on('dragstart', () => {
      selectClip(clip.id);
    });

    clipRect.on('dragmove', (e) => {
      // Allow free movement during drag for smooth UX
      // Constraints will be applied on dragend when the clip is released

      // Optional: Add visual feedback during drag (e.g., snap guides)
      // const target = e.target;
      // const newX = target.x();
      // const newTime = (newX + currentScrollOffset) / currentPixelsPerSecond;
      // const snappedTime = Math.round(newTime * 4) / 4; // Snap to 0.25s grid
    });

    clipRect.on('dragend', (e) => {
      const target = e.target;
      const newX = target.x();
      const newTime = (newX + currentScrollOffset) / currentPixelsPerSecond;
      const constrainedTime = Math.max(0, newTime);

      // Find which track the clip was dropped on
      const dropY = target.y() + target.height() / 2;
      let targetTrackIndex = 0;

      for (let i = 0; i < currentTimeline.tracks.length; i++) {
        const trackTop = RULER_HEIGHT + i * (TRACK_HEIGHT + TRACK_PADDING);
        const trackBottom = trackTop + TRACK_HEIGHT;

        if (dropY >= trackTop && dropY <= trackBottom) {
          targetTrackIndex = i;
          break;
        }
      }

      const targetTrack = currentTimeline.tracks[targetTrackIndex];

      // Update clip position
      moveClip(clip.id, targetTrack.id, constrainedTime);
    });

    // Click to select
    clipRect.on('click', () => {
      selectClip(clip.id);
    });

    // Right-click context menu
    clipRect.on('contextmenu', (e) => {
      e.evt.preventDefault();
      selectClip(clip.id);

      // Show custom confirm dialog near cursor (convert to screen coordinates)
      const stageBox = container.getBoundingClientRect();
      const mousePos = stage.getPointerPosition();
      if (mousePos) {
        const clipName = clip.name || `Clip ${clip.id.substring(0, 8)}`;
        confirmMessage = `Delete "${clipName}"?`;
        // Convert Konva canvas coordinates to screen coordinates
        confirmDialogX = stageBox.left + mousePos.x;
        confirmDialogY = stageBox.top + mousePos.y;
        pendingDeleteClipId = clip.id;
        showConfirmDialog = true;
      }
    });

    trackLayer.add(clipRect);
    trackLayer.add(clipText);

    // Resize handles (trim functionality) - add AFTER clip rect/text so they appear on top
    if (isSelected) {
      // Left handle
      const leftHandle = new Konva.Rect({
        x: clipX,
        y: trackY + 5,
        width: 12,
        height: TRACK_HEIGHT - 10,
        fill: '#667eea',
        opacity: 1.0,
        cursor: 'ew-resize',
        draggable: true,
        shadowColor: 'black',
        shadowBlur: 4,
        shadowOpacity: 0.3,
        dragBoundFunc: (pos) => {
          const maxX = clipX + clipWidth - 20;
          return {
            x: Math.min(Math.max(clipX - (clip.duration - 0.5) * currentPixelsPerSecond, pos.x), maxX),
            y: trackY + 5,
          };
        },
      });

      leftHandle.on('dragmove', (e) => {
        const dx = e.target.x() - clipX;

        // Update visually
        clipRect.x(clipRect.x() + dx);
        clipRect.width(clipRect.width() - dx);
      });

      leftHandle.on('dragend', (e) => {
        const dx = e.target.x() - clipX;
        const timeDelta = dx / currentPixelsPerSecond;

        // Get source file to check bounds
        const sourceFile = $mediaLibraryStore.find(f => f.id === clip.media_file_id);
        if (!sourceFile) return;

        // Calculate new trim_start and duration
        let newTrimStart = clip.trim_start + timeDelta;

        // Constrain: can't trim before start of source file (0)
        newTrimStart = Math.max(0, newTrimStart);

        // Constrain: can't extend beyond original clip end
        const maxTrimStart = clip.trim_end - 0.5; // Must leave at least 0.5s
        newTrimStart = Math.min(maxTrimStart, newTrimStart);

        const newDuration = clip.trim_end - newTrimStart;

        // Update both trim_start and duration
        updateClipDuration(clip.id, newDuration, newTrimStart, clip.trim_end);
      });

      // Right handle
      const rightHandle = new Konva.Rect({
        x: clipX + clipWidth - 12,
        y: trackY + 5,
        width: 12,
        height: TRACK_HEIGHT - 10,
        fill: '#667eea',
        opacity: 1.0,
        cursor: 'ew-resize',
        draggable: true,
        shadowColor: 'black',
        shadowBlur: 4,
        shadowOpacity: 0.3,
        dragBoundFunc: (pos) => {
          return {
            x: Math.max(clipX + 20, pos.x),
            y: trackY + 5,
          };
        },
      });

      rightHandle.on('dragmove', (e) => {
        const newWidth = e.target.x() - clipX + 12;
        clipRect.width(Math.max(20, newWidth));
      });

      rightHandle.on('dragend', (e) => {
        const newWidth = e.target.x() - clipX + 12;
        let newDuration = Math.max(0.5, newWidth / currentPixelsPerSecond);

        // Get source file to check bounds
        const sourceFile = $mediaLibraryStore.find(f => f.id === clip.media_file_id);
        if (!sourceFile) return;

        // Calculate new trim_end based on new duration
        let newTrimEnd = clip.trim_start + newDuration;

        // Constrain: can't extend beyond source file duration
        newTrimEnd = Math.min(sourceFile.duration, newTrimEnd);

        // Recalculate duration based on constrained trim_end
        newDuration = newTrimEnd - clip.trim_start;

        // Update both duration and trim_end
        updateClipDuration(clip.id, newDuration, clip.trim_start, newTrimEnd);
      });

      trackLayer.add(leftHandle);
      trackLayer.add(rightHandle);
    }
  }

  let isDraggingPlayhead = false;
  let playheadLine: Konva.Line | null = null;
  let playheadHandle: Konva.Circle | null = null;

  function renderPlayhead() {
    if (!playheadLayer || isDraggingPlayhead) return;  // Skip rendering during drag

    playheadLayer.destroyChildren();

    const playheadX = (currentPlayheadTime * currentPixelsPerSecond) - currentScrollOffset;

    // Playhead line
    playheadLine = new Konva.Line({
      points: [playheadX, 0, playheadX, height],
      stroke: '#ff4444',
      strokeWidth: 2,
      listening: false,
    });

    // Playhead handle with larger hit area for easier grabbing
    playheadHandle = new Konva.Circle({
      x: playheadX,
      y: RULER_HEIGHT / 2,
      radius: 12,  // Increased from 8 to 12 for easier grabbing
      fill: '#ff4444',
      draggable: true,
      hitStrokeWidth: 20,  // Larger invisible hit area for easier clicking
      dragBoundFunc: (pos) => {
        // Simplified - just prevent negative positions
        return {
          x: Math.max(0, pos.x),
          y: RULER_HEIGHT / 2,
        };
      },
    });

    playheadHandle.on('dragstart', () => {
      isDraggingPlayhead = true;
    });

    playheadHandle.on('dragmove', (e) => {
      const newX = e.target.x();
      const newTime = (newX + currentScrollOffset) / currentPixelsPerSecond;

      // Update line position manually without re-rendering
      if (playheadLine) {
        playheadLine.points([newX, 0, newX, height]);
        playheadLayer.batchDraw();
      }

      // Update store (won't trigger re-render because of isDraggingPlayhead flag)
      playheadTime.set(Math.max(0, newTime));
    });

    playheadHandle.on('dragend', () => {
      isDraggingPlayhead = false;
      // Re-render to ensure everything is synced
      renderPlayhead();
    });

    playheadLayer.add(playheadLine);
    playheadLayer.add(playheadHandle);
    playheadLayer.batchDraw();
  }

  function handleWheel(e: Konva.KonvaEventObject<WheelEvent>) {
    e.evt.preventDefault();

    const oldScale = currentPixelsPerSecond;
    const pointer = stage.getPointerPosition();
    if (!pointer) return;

    const mousePointTo = {
      x: (pointer.x + currentScrollOffset) / oldScale,
    };

    const scaleBy = 1.1;
    const newScale = e.evt.deltaY > 0 ? oldScale / scaleBy : oldScale * scaleBy;

    // Clamp zoom
    const clampedScale = Math.max(10, Math.min(200, newScale));
    pixelsPerSecond.set(clampedScale);

    const newPos = mousePointTo.x * clampedScale - pointer.x;
    scrollOffset.set(Math.max(0, newPos));
  }

  // Handle horizontal scrolling
  function handleScroll(e: WheelEvent) {
    if (!e.shiftKey) return;
    e.preventDefault();

    const delta = e.deltaY;
    const newOffset = currentScrollOffset + delta;
    scrollOffset.set(Math.max(0, newOffset));
  }

  // Handle confirm dialog response
  function handleConfirmResponse(event: CustomEvent) {
    if (event.detail.confirmed && pendingDeleteClipId) {
      removeClip(pendingDeleteClipId);
    }
    // Reset state
    pendingDeleteClipId = null;
    showConfirmDialog = false;
  }

  // Drag-and-drop handlers for adding media to timeline
  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    if (e.dataTransfer) {
      e.dataTransfer.dropEffect = 'copy';
    }
  }

  async function handleDrop(e: DragEvent) {
    e.preventDefault();

    if (!e.dataTransfer) return;

    // Get MediaFile data
    const data = e.dataTransfer.getData('application/json');
    if (!data) return;

    try {
      const mediaFile = JSON.parse(data);

      // Calculate drop position from mouse X coordinate
      const rect = container.getBoundingClientRect();
      const dropX = e.clientX - rect.left;
      const timePosition = (dropX + currentScrollOffset) / currentPixelsPerSecond;

      // Calculate drop track from mouse Y coordinate
      const dropY = e.clientY - rect.top;
      const trackIndex = Math.floor((dropY - RULER_HEIGHT) / (TRACK_HEIGHT + TRACK_PADDING));

      // Get target track
      const targetTrack = currentTimeline.tracks[trackIndex];

      if (targetTrack && !targetTrack.locked) {
        // Add file to timeline at drop position
        await addMediaFileToTimeline(mediaFile, targetTrack.id, Math.max(0, timePosition));
        console.log(`Dropped ${mediaFile.filename} on track ${targetTrack.id} at ${timePosition.toFixed(2)}s`);
      } else {
        console.warn('Cannot drop on locked or invalid track');
      }
    } catch (error) {
      console.error('Failed to handle drop:', error);
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    // Delete selected clip with Delete or Backspace key
    if ((e.key === 'Delete' || e.key === 'Backspace') && currentSelectedClipId) {
      e.preventDefault();

      // Find the selected clip name
      let clipName = '';
      for (const track of currentTimeline.tracks) {
        const clip = track.clips.find(c => c.id === currentSelectedClipId);
        if (clip) {
          clipName = clip.name || `Clip ${clip.id.substring(0, 8)}`;
          break;
        }
      }

      // Show custom confirm dialog at center of screen
      confirmMessage = `Delete "${clipName}"?`;
      confirmDialogX = window.innerWidth / 2;
      confirmDialogY = window.innerHeight / 2;
      pendingDeleteClipId = currentSelectedClipId;
      showConfirmDialog = true;
    }
  }
</script>

<div class="timeline-container">
  <!-- svelte-ignore a11y-no-noninteractive-tabindex -->
  <div
    bind:this={container}
    class="timeline-canvas"
    on:wheel={handleScroll}
    on:dragover={handleDragOver}
    on:drop={handleDrop}
    on:keydown={handleKeyDown}
    role="application"
    aria-label="Timeline editor"
    tabindex="0"
  ></div>

  <div class="timeline-controls">
    <button on:click={() => pixelsPerSecond.update(v => v * 1.2)}>
      Zoom In
    </button>
    <button on:click={() => pixelsPerSecond.update(v => v / 1.2)}>
      Zoom Out
    </button>
    <button on:click={() => scrollOffset.set(0)}>
      Reset View
    </button>
  </div>
</div>

<ConfirmDialog
  bind:show={showConfirmDialog}
  message={confirmMessage}
  x={confirmDialogX}
  y={confirmDialogY}
  on:confirm={handleConfirmResponse}
/>

<style>
  .timeline-container {
    display: flex;
    flex-direction: column;
    width: 100%;
    background: #1a1a1a;
    border-radius: 8px;
    overflow: hidden;
  }

  .timeline-canvas {
    cursor: default;
    background: #1a1a1a;
  }

  .timeline-controls {
    display: flex;
    gap: 0.5rem;
    padding: 1rem;
    background: #2d2d2d;
    border-top: 1px solid #3d3d3d;
  }

  .timeline-controls button {
    padding: 0.5rem 1rem;
    background: #444;
    color: #fff;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.875rem;
    transition: background 0.2s;
  }

  .timeline-controls button:hover {
    background: #555;
  }

  .timeline-controls button:active {
    background: #666;
  }
</style>
