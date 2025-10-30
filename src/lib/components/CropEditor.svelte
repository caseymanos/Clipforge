<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import type { RecordingSource, CropRegion } from '../stores/recordingStore';

  export let source: RecordingSource;
  export let initialCropRegion: CropRegion | null = null;
  export let show = false;

  const dispatch = createEventDispatcher<{
    apply: CropRegion;
    cancel: void;
  }>();

  let previewContainer: HTMLDivElement;
  let previewImage: HTMLImageElement;
  let previewWidth = 0;
  let previewHeight = 0;
  let scaleX = 1;
  let scaleY = 1;

  // Selection rectangle in preview space (pixels)
  let selectionX = 0;
  let selectionY = 0;
  let selectionWidth = 0;
  let selectionHeight = 0;

  // Drag state
  let isDragging = false;
  let dragMode: 'none' | 'create' | 'move' | 'resize-nw' | 'resize-ne' | 'resize-sw' | 'resize-se' | 'resize-n' | 'resize-s' | 'resize-e' | 'resize-w' = 'none';
  let dragStartX = 0;
  let dragStartY = 0;
  let dragStartSelectionX = 0;
  let dragStartSelectionY = 0;
  let dragStartSelectionWidth = 0;
  let dragStartSelectionHeight = 0;

  $: sourceWidth = source.width || 1920;
  $: sourceHeight = source.height || 1080;

  // Convert crop region from source space to preview space
  function sourceToPreview(cropRegion: CropRegion) {
    return {
      x: cropRegion.x / scaleX,
      y: cropRegion.y / scaleY,
      width: cropRegion.width / scaleX,
      height: cropRegion.height / scaleY,
    };
  }

  // Convert selection from preview space to source space
  function previewToSource(): CropRegion {
    return {
      x: Math.round(selectionX * scaleX),
      y: Math.round(selectionY * scaleY),
      width: Math.round(selectionWidth * scaleX),
      height: Math.round(selectionHeight * scaleY),
    };
  }

  function updateScale() {
    if (previewImage && previewImage.complete) {
      previewWidth = previewImage.offsetWidth;
      previewHeight = previewImage.offsetHeight;
      scaleX = sourceWidth / previewWidth;
      scaleY = sourceHeight / previewHeight;

      // Initialize selection if we have an initial crop region
      if (initialCropRegion) {
        const preview = sourceToPreview(initialCropRegion);
        selectionX = preview.x;
        selectionY = preview.y;
        selectionWidth = preview.width;
        selectionHeight = preview.height;
      } else {
        // Default to 50% of screen in center
        selectionWidth = previewWidth * 0.5;
        selectionHeight = previewHeight * 0.5;
        selectionX = (previewWidth - selectionWidth) / 2;
        selectionY = (previewHeight - selectionHeight) / 2;
      }
    }
  }

  function handleImageLoad() {
    updateScale();
  }

  function getMousePosition(e: MouseEvent): { x: number; y: number } {
    if (!previewContainer) return { x: 0, y: 0 };
    const rect = previewContainer.getBoundingClientRect();
    return {
      x: e.clientX - rect.left,
      y: e.clientY - rect.top,
    };
  }

  function getHandleAtPosition(x: number, y: number): typeof dragMode {
    const handleSize = 12;
    const edgeThreshold = 8;

    // Check corners first
    if (Math.abs(x - selectionX) < handleSize && Math.abs(y - selectionY) < handleSize) {
      return 'resize-nw';
    }
    if (Math.abs(x - (selectionX + selectionWidth)) < handleSize && Math.abs(y - selectionY) < handleSize) {
      return 'resize-ne';
    }
    if (Math.abs(x - selectionX) < handleSize && Math.abs(y - (selectionY + selectionHeight)) < handleSize) {
      return 'resize-sw';
    }
    if (Math.abs(x - (selectionX + selectionWidth)) < handleSize && Math.abs(y - (selectionY + selectionHeight)) < handleSize) {
      return 'resize-se';
    }

    // Check edges
    if (Math.abs(y - selectionY) < edgeThreshold && x >= selectionX && x <= selectionX + selectionWidth) {
      return 'resize-n';
    }
    if (Math.abs(y - (selectionY + selectionHeight)) < edgeThreshold && x >= selectionX && x <= selectionX + selectionWidth) {
      return 'resize-s';
    }
    if (Math.abs(x - selectionX) < edgeThreshold && y >= selectionY && y <= selectionY + selectionHeight) {
      return 'resize-w';
    }
    if (Math.abs(x - (selectionX + selectionWidth)) < edgeThreshold && y >= selectionY && y <= selectionY + selectionHeight) {
      return 'resize-e';
    }

    // Check if inside selection (move)
    if (x >= selectionX && x <= selectionX + selectionWidth && y >= selectionY && y <= selectionY + selectionHeight) {
      return 'move';
    }

    return 'create';
  }

  function handleMouseDown(e: MouseEvent) {
    const pos = getMousePosition(e);
    dragMode = getHandleAtPosition(pos.x, pos.y);
    isDragging = true;
    dragStartX = pos.x;
    dragStartY = pos.y;
    dragStartSelectionX = selectionX;
    dragStartSelectionY = selectionY;
    dragStartSelectionWidth = selectionWidth;
    dragStartSelectionHeight = selectionHeight;
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isDragging) return;

    const pos = getMousePosition(e);
    const dx = pos.x - dragStartX;
    const dy = pos.y - dragStartY;

    if (dragMode === 'create') {
      // Creating new selection
      selectionX = Math.min(dragStartX, pos.x);
      selectionY = Math.min(dragStartY, pos.y);
      selectionWidth = Math.abs(pos.x - dragStartX);
      selectionHeight = Math.abs(pos.y - dragStartY);
    } else if (dragMode === 'move') {
      // Moving selection
      selectionX = Math.max(0, Math.min(previewWidth - selectionWidth, dragStartSelectionX + dx));
      selectionY = Math.max(0, Math.min(previewHeight - selectionHeight, dragStartSelectionY + dy));
    } else if (dragMode.startsWith('resize-')) {
      // Resizing
      let newX = dragStartSelectionX;
      let newY = dragStartSelectionY;
      let newWidth = dragStartSelectionWidth;
      let newHeight = dragStartSelectionHeight;

      if (dragMode.includes('n')) {
        newY = Math.max(0, dragStartSelectionY + dy);
        newHeight = dragStartSelectionHeight - (newY - dragStartSelectionY);
      }
      if (dragMode.includes('s')) {
        newHeight = Math.max(10, Math.min(previewHeight - dragStartSelectionY, dragStartSelectionHeight + dy));
      }
      if (dragMode.includes('w')) {
        newX = Math.max(0, dragStartSelectionX + dx);
        newWidth = dragStartSelectionWidth - (newX - dragStartSelectionX);
      }
      if (dragMode.includes('e')) {
        newWidth = Math.max(10, Math.min(previewWidth - dragStartSelectionX, dragStartSelectionWidth + dx));
      }

      // Apply constraints
      if (newWidth >= 10 && newHeight >= 10) {
        selectionX = newX;
        selectionY = newY;
        selectionWidth = newWidth;
        selectionHeight = newHeight;
      }
    }

    // Constrain to preview bounds
    selectionX = Math.max(0, Math.min(previewWidth - selectionWidth, selectionX));
    selectionY = Math.max(0, Math.min(previewHeight - selectionHeight, selectionY));
    selectionWidth = Math.max(10, Math.min(previewWidth - selectionX, selectionWidth));
    selectionHeight = Math.max(10, Math.min(previewHeight - selectionY, selectionHeight));
  }

  function handleMouseUp() {
    isDragging = false;
    dragMode = 'none';
  }

  function handleApply() {
    const cropRegion = previewToSource();
    dispatch('apply', cropRegion);
  }

  function handleCancel() {
    dispatch('cancel');
  }

  onMount(() => {
    if (previewImage) {
      updateScale();
    }
  });

  $: sourceCropRegion = previewToSource();
</script>

{#if show}
  <div class="modal-overlay" on:click={handleCancel} on:keydown={(e) => e.key === 'Escape' && handleCancel()}>
    <div class="modal-content" on:click|stopPropagation>
      <div class="modal-header">
        <h3>Select Crop Region</h3>
        <button class="close-btn" on:click={handleCancel}>×</button>
      </div>

      <div class="crop-editor">
        <div
          class="preview-container"
          bind:this={previewContainer}
          on:mousedown={handleMouseDown}
          on:mousemove={handleMouseMove}
          on:mouseup={handleMouseUp}
          on:mouseleave={handleMouseUp}
        >
          {#if source.preview_path}
            <img
              bind:this={previewImage}
              src={convertFileSrc(source.preview_path)}
              alt={source.name}
              class="preview-image"
              on:load={handleImageLoad}
              draggable="false"
            />
          {:else}
            <div class="preview-placeholder">
              {source.type === 'screen' ? '🖥️' : '🪟'}
            </div>
          {/if}

          {#if previewWidth > 0}
            <div
              class="selection-rectangle"
              style="left: {selectionX}px; top: {selectionY}px; width: {selectionWidth}px; height: {selectionHeight}px;"
            >
              <div class="handle handle-nw"></div>
              <div class="handle handle-ne"></div>
              <div class="handle handle-sw"></div>
              <div class="handle handle-se"></div>
              <div class="handle handle-n"></div>
              <div class="handle handle-s"></div>
              <div class="handle handle-e"></div>
              <div class="handle handle-w"></div>
            </div>
          {/if}
        </div>

        <div class="crop-info">
          <div class="info-row">
            <span class="label">Position:</span>
            <span class="value">X: {sourceCropRegion.x}px, Y: {sourceCropRegion.y}px</span>
          </div>
          <div class="info-row">
            <span class="label">Size:</span>
            <span class="value">{sourceCropRegion.width} × {sourceCropRegion.height}px</span>
          </div>
        </div>
      </div>

      <div class="modal-actions">
        <button class="btn-secondary" on:click={handleCancel}>Cancel</button>
        <button class="btn-primary" on:click={handleApply}>Apply Crop Region</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: rgba(0, 0, 0, 0.8);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 2000;
  }

  .modal-content {
    background: #1a1a1a;
    border-radius: 8px;
    max-width: 800px;
    width: 90%;
    max-height: 90vh;
    overflow-y: auto;
    border: 1px solid #2d2d2d;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem 2rem;
    border-bottom: 1px solid #2d2d2d;
  }

  .modal-header h3 {
    margin: 0;
    color: #fff;
    font-size: 1.3rem;
  }

  .close-btn {
    background: none;
    border: none;
    color: #aaa;
    font-size: 2rem;
    cursor: pointer;
    padding: 0;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: color 0.2s;
  }

  .close-btn:hover {
    color: #fff;
  }

  .crop-editor {
    padding: 2rem;
  }

  .preview-container {
    position: relative;
    display: inline-block;
    cursor: crosshair;
    user-select: none;
    max-width: 100%;
  }

  .preview-image {
    display: block;
    max-width: 100%;
    height: auto;
    border-radius: 4px;
  }

  .preview-placeholder {
    width: 600px;
    height: 400px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #2d2d2d;
    border-radius: 4px;
    font-size: 5rem;
  }

  .selection-rectangle {
    position: absolute;
    border: 2px solid #667eea;
    background: rgba(102, 126, 234, 0.2);
    box-shadow: 0 0 0 9999px rgba(0, 0, 0, 0.5);
    cursor: move;
  }

  .handle {
    position: absolute;
    width: 12px;
    height: 12px;
    background: #667eea;
    border: 2px solid #fff;
    border-radius: 50%;
  }

  .handle-nw {
    top: -6px;
    left: -6px;
    cursor: nw-resize;
  }

  .handle-ne {
    top: -6px;
    right: -6px;
    cursor: ne-resize;
  }

  .handle-sw {
    bottom: -6px;
    left: -6px;
    cursor: sw-resize;
  }

  .handle-se {
    bottom: -6px;
    right: -6px;
    cursor: se-resize;
  }

  .handle-n {
    top: -6px;
    left: 50%;
    transform: translateX(-50%);
    cursor: n-resize;
  }

  .handle-s {
    bottom: -6px;
    left: 50%;
    transform: translateX(-50%);
    cursor: s-resize;
  }

  .handle-e {
    top: 50%;
    right: -6px;
    transform: translateY(-50%);
    cursor: e-resize;
  }

  .handle-w {
    top: 50%;
    left: -6px;
    transform: translateY(-50%);
    cursor: w-resize;
  }

  .crop-info {
    margin-top: 1.5rem;
    padding: 1rem;
    background: #2d2d2d;
    border-radius: 4px;
  }

  .info-row {
    display: flex;
    justify-content: space-between;
    margin-bottom: 0.5rem;
  }

  .info-row:last-child {
    margin-bottom: 0;
  }

  .label {
    color: #aaa;
    font-size: 0.9rem;
  }

  .value {
    color: #fff;
    font-weight: 500;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    padding: 1.5rem 2rem;
    border-top: 1px solid #2d2d2d;
  }

  button {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9rem;
    transition: all 0.2s;
    font-weight: 500;
  }

  .btn-primary {
    background: #667eea;
    color: #fff;
  }

  .btn-primary:hover {
    background: #5568d3;
  }

  .btn-secondary {
    background: #3d3d3d;
    color: #fff;
  }

  .btn-secondary:hover {
    background: #4d4d4d;
  }
</style>
