<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import Konva from 'konva';
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
    type Timeline,
    type Track,
    type Clip
  } from '../stores/timelineStore';

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
    if (!trackLayer) return;

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
      text: `Clip ${clip.id.substring(0, 8)}`,
      fontSize: 12,
      fill: '#ffffff',
      listening: false,
    });

    // Drag event handlers
    clipRect.on('dragstart', () => {
      selectClip(clip.id);
    });

    clipRect.on('dragmove', (e) => {
      const target = e.target;
      const newX = target.x();
      const newTime = (newX + currentScrollOffset) / currentPixelsPerSecond;

      // Constrain to timeline bounds
      const constrainedTime = Math.max(0, newTime);
      target.x((constrainedTime * currentPixelsPerSecond) - currentScrollOffset);

      // Snap to grid (optional)
      // const snappedTime = Math.round(constrainedTime * 4) / 4; // Snap to 0.25s
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

    // Resize handles (trim functionality)
    if (isSelected) {
      // Left handle
      const leftHandle = new Konva.Rect({
        x: clipX,
        y: trackY + 5,
        width: 8,
        height: TRACK_HEIGHT - 10,
        fill: '#ffffff',
        opacity: 0.8,
        cursor: 'ew-resize',
        draggable: true,
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
        const newTrimStart = clip.trim_start - (dx / currentPixelsPerSecond);
        const newDuration = clip.duration + (dx / currentPixelsPerSecond);

        // Update visually
        clipRect.x(clipRect.x() + dx);
        clipRect.width(clipRect.width() - dx);
      });

      leftHandle.on('dragend', (e) => {
        const dx = e.target.x() - clipX;
        const newDuration = Math.max(0.5, clip.duration + (dx / currentPixelsPerSecond));
        updateClipDuration(clip.id, newDuration);
      });

      // Right handle
      const rightHandle = new Konva.Rect({
        x: clipX + clipWidth - 8,
        y: trackY + 5,
        width: 8,
        height: TRACK_HEIGHT - 10,
        fill: '#ffffff',
        opacity: 0.8,
        cursor: 'ew-resize',
        draggable: true,
        dragBoundFunc: (pos) => {
          return {
            x: Math.max(clipX + 20, pos.x),
            y: trackY + 5,
          };
        },
      });

      rightHandle.on('dragmove', (e) => {
        const newWidth = e.target.x() - clipX + 8;
        clipRect.width(Math.max(20, newWidth));
      });

      rightHandle.on('dragend', (e) => {
        const newWidth = e.target.x() - clipX + 8;
        const newDuration = Math.max(0.5, newWidth / currentPixelsPerSecond);
        updateClipDuration(clip.id, newDuration);
      });

      trackLayer.add(leftHandle);
      trackLayer.add(rightHandle);
    }

    trackLayer.add(clipRect);
    trackLayer.add(clipText);
  }

  function renderPlayhead() {
    if (!playheadLayer) return;

    playheadLayer.destroyChildren();

    const playheadX = (currentPlayheadTime * currentPixelsPerSecond) - currentScrollOffset;

    // Playhead line
    const line = new Konva.Line({
      points: [playheadX, 0, playheadX, height],
      stroke: '#ff4444',
      strokeWidth: 2,
      listening: false,
    });

    // Playhead handle
    const handle = new Konva.Circle({
      x: playheadX,
      y: RULER_HEIGHT / 2,
      radius: 8,
      fill: '#ff4444',
      draggable: true,
      dragBoundFunc: (pos) => {
        return {
          x: Math.max(0, Math.min(width, pos.x)),
          y: RULER_HEIGHT / 2,
        };
      },
    });

    handle.on('dragmove', (e) => {
      const newX = e.target.x();
      const newTime = (newX + currentScrollOffset) / currentPixelsPerSecond;
      playheadTime.set(Math.max(0, newTime));
    });

    playheadLayer.add(line);
    playheadLayer.add(handle);
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
</script>

<div class="timeline-container">
  <!-- svelte-ignore a11y-no-noninteractive-tabindex -->
  <div
    bind:this={container}
    class="timeline-canvas"
    on:wheel={handleScroll}
    on:dragover={handleDragOver}
    on:drop={handleDrop}
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
