<script lang="ts">
  import { onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import {
    mediaLibraryStore,
    isLoadingLibrary,
    selectedMediaFile,
    mediaLibraryError,
    loadMediaLibrary,
    importMediaFile,
    deleteMediaFile,
    selectMediaFile,
    formatFileSize,
    formatDuration,
    type MediaFile
  } from '../stores/mediaLibraryStore';
  import { addMediaFileToTimeline } from '../stores/timelineStore';

  let searchQuery = '';
  let sortBy: 'name' | 'date' | 'duration' | 'size' = 'date';

  onMount(async () => {
    // Load media library on mount
    await loadMediaLibrary();
  });

  // Filtered and sorted media files
  $: filteredFiles = $mediaLibraryStore
    .filter(file => {
      if (!searchQuery) return true;
      return file.filename.toLowerCase().includes(searchQuery.toLowerCase());
    })
    .sort((a, b) => {
      switch (sortBy) {
        case 'name':
          return a.filename.localeCompare(b.filename);
        case 'date':
          return new Date(b.imported_at).getTime() - new Date(a.imported_at).getTime();
        case 'duration':
          return b.duration - a.duration;
        case 'size':
          return b.file_size - a.file_size;
        default:
          return 0;
      }
    });

  async function handleImportClick() {
    try {
      const selected = await open({
        multiple: true,
        filters: [
          {
            name: 'Video Files',
            extensions: ['mp4', 'mov', 'avi', 'mkv', 'webm', 'flv', 'm4v']
          }
        ]
      });

      if (!selected) return;

      const paths = Array.isArray(selected) ? selected : [selected];

      for (const path of paths) {
        await importMediaFile(path);
      }
    } catch (error) {
      console.error('Import failed:', error);
    }
  }

  function handleFileClick(file: MediaFile) {
    selectMediaFile(file.id);
  }

  async function handleFileDoubleClick(file: MediaFile) {
    try {
      // Add file to timeline (default video track, at end)
      await addMediaFileToTimeline(file);
      console.log('Added file to timeline:', file.filename);
      // TODO: Show success toast notification
    } catch (error) {
      console.error('Failed to add file to timeline:', error);
      // TODO: Show error toast notification
    }
  }

  function handleDragStart(event: DragEvent, file: MediaFile) {
    if (!event.dataTransfer) return;

    // Set drag data (MediaFile as JSON)
    event.dataTransfer.effectAllowed = 'copy';
    event.dataTransfer.setData('application/json', JSON.stringify(file));

    // Add visual feedback
    if (event.target instanceof HTMLElement) {
      event.target.style.opacity = '0.5';
    }
  }

  function handleDragEnd(event: DragEvent) {
    // Remove visual feedback
    if (event.target instanceof HTMLElement) {
      event.target.style.opacity = '1';
    }
  }

  async function handleDeleteClick(fileId: string, event: Event) {
    event.stopPropagation();

    if (confirm('Are you sure you want to remove this file from the library?')) {
      await deleteMediaFile(fileId);
    }
  }

  function getThumbnailUrl(file: MediaFile): string {
    if (file.thumbnail_path) {
      const url = convertFileSrc(file.thumbnail_path);
      console.log('Thumbnail for', file.filename, ':', file.thumbnail_path, '-> URL:', url);
      return url;
    }
    console.log('No thumbnail_path for', file.filename);
    return ''; // Placeholder for files without thumbnails
  }
</script>

<div class="media-library">
  <div class="toolbar">
    <h3>Media Library</h3>

    <div class="toolbar-actions">
      <input
        type="text"
        placeholder="Search files..."
        bind:value={searchQuery}
        class="search-input"
      />

      <select bind:value={sortBy} class="sort-select">
        <option value="date">Sort by Date</option>
        <option value="name">Sort by Name</option>
        <option value="duration">Sort by Duration</option>
        <option value="size">Sort by Size</option>
      </select>

      <button on:click={handleImportClick} class="import-btn" disabled={$isLoadingLibrary}>
        {$isLoadingLibrary ? 'Importing...' : '+ Import Media'}
      </button>
    </div>
  </div>

  {#if $mediaLibraryError}
    <div class="error-message">
      Error: {$mediaLibraryError}
    </div>
  {/if}

  <div class="media-grid">
    {#if $isLoadingLibrary && $mediaLibraryStore.length === 0}
      <div class="loading-state">
        <div class="spinner"></div>
        <p>Loading media library...</p>
      </div>
    {:else if filteredFiles.length === 0}
      <div class="empty-state">
        <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor">
          <path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z"/>
          <circle cx="12" cy="13" r="4"/>
        </svg>
        <h4>{searchQuery ? 'No files found' : 'No media files yet'}</h4>
        <p>{searchQuery ? 'Try a different search term' : 'Click "Import Media" to add video files'}</p>
      </div>
    {:else}
      {#each filteredFiles as file (file.id)}
        <div
          class="media-item"
          class:selected={$selectedMediaFile === file.id}
          draggable="true"
          on:click={() => handleFileClick(file)}
          on:dblclick={() => handleFileDoubleClick(file)}
          on:dragstart={(e) => handleDragStart(e, file)}
          on:dragend={handleDragEnd}
        >
          <div class="thumbnail">
            {#if file.thumbnail_path}
              <img src={getThumbnailUrl(file)} alt={file.filename} />
            {:else}
              <div class="thumbnail-placeholder">
                <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                  <polygon points="5 3 19 12 5 21 5 3"/>
                </svg>
              </div>
            {/if}
            <div class="duration-badge">
              {formatDuration(file.duration)}
            </div>
          </div>

          <div class="media-info">
            <div class="filename" title={file.filename}>
              {file.filename}
            </div>
            <div class="metadata">
              <span class="resolution">
                {file.resolution.width}x{file.resolution.height}
              </span>
              <span class="codec">
                {file.codec.video}
              </span>
              <span class="filesize">
                {formatFileSize(file.file_size)}
              </span>
            </div>
          </div>

          <button
            class="delete-btn"
            on:click={(e) => handleDeleteClick(file.id, e)}
            title="Remove from library"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor">
              <polyline points="3 6 5 6 21 6"/>
              <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
            </svg>
          </button>
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .media-library {
    background: #1a1a1a;
    border-radius: 0.5rem;
    border: 1px solid #2d2d2d;
    overflow: hidden;
  }

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.5rem;
    background: #151515;
    border-bottom: 1px solid #2d2d2d;
  }

  .toolbar h3 {
    margin: 0;
    font-size: 1.1rem;
    color: #fff;
  }

  .toolbar-actions {
    display: flex;
    gap: 0.75rem;
    align-items: center;
  }

  .search-input {
    padding: 0.5rem 0.75rem;
    background: #0f0f0f;
    border: 1px solid #2d2d2d;
    border-radius: 0.375rem;
    color: #fff;
    font-size: 0.875rem;
    width: 200px;
  }

  .search-input:focus {
    outline: none;
    border-color: #667eea;
  }

  .sort-select {
    padding: 0.5rem 0.75rem;
    background: #0f0f0f;
    border: 1px solid #2d2d2d;
    border-radius: 0.375rem;
    color: #fff;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .sort-select:focus {
    outline: none;
    border-color: #667eea;
  }

  .import-btn {
    padding: 0.5rem 1rem;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    border: none;
    border-radius: 0.375rem;
    color: #fff;
    font-weight: 600;
    font-size: 0.875rem;
    cursor: pointer;
    transition: opacity 0.2s;
  }

  .import-btn:hover:not(:disabled) {
    opacity: 0.9;
  }

  .import-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .error-message {
    padding: 1rem 1.5rem;
    background: rgba(239, 68, 68, 0.1);
    border-bottom: 1px solid rgba(239, 68, 68, 0.3);
    color: #ef4444;
    font-size: 0.875rem;
  }

  .media-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 1rem;
    padding: 1.5rem;
    max-height: 400px;
    overflow-y: auto;
  }

  .loading-state,
  .empty-state {
    grid-column: 1 / -1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem 1rem;
    color: #666;
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid #2d2d2d;
    border-top-color: #667eea;
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: 1rem;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .empty-state svg {
    color: #2d2d2d;
    margin-bottom: 1rem;
  }

  .empty-state h4 {
    margin: 0 0 0.5rem;
    font-size: 1.1rem;
    color: #aaa;
  }

  .empty-state p {
    margin: 0;
    font-size: 0.875rem;
    color: #666;
  }

  .media-item {
    position: relative;
    background: #0f0f0f;
    border: 2px solid #2d2d2d;
    border-radius: 0.5rem;
    overflow: hidden;
    cursor: pointer;
    transition: all 0.2s;
  }

  .media-item:hover {
    border-color: #667eea;
    transform: translateY(-2px);
  }

  .media-item.selected {
    border-color: #667eea;
    box-shadow: 0 0 0 2px rgba(102, 126, 234, 0.2);
  }

  .thumbnail {
    position: relative;
    width: 100%;
    aspect-ratio: 16 / 9;
    background: #000;
    overflow: hidden;
  }

  .thumbnail img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .thumbnail-placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #2d2d2d;
  }

  .duration-badge {
    position: absolute;
    bottom: 0.5rem;
    right: 0.5rem;
    padding: 0.25rem 0.5rem;
    background: rgba(0, 0, 0, 0.8);
    border-radius: 0.25rem;
    font-size: 0.75rem;
    font-weight: 600;
    color: #fff;
  }

  .media-info {
    padding: 0.75rem;
  }

  .filename {
    font-size: 0.875rem;
    font-weight: 500;
    color: #fff;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-bottom: 0.5rem;
  }

  .metadata {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
    font-size: 0.75rem;
    color: #666;
  }

  .metadata span {
    background: #1a1a1a;
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
  }

  .delete-btn {
    position: absolute;
    top: 0.5rem;
    right: 0.5rem;
    padding: 0.375rem;
    background: rgba(0, 0, 0, 0.8);
    border: none;
    border-radius: 0.25rem;
    color: #fff;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.2s;
  }

  .media-item:hover .delete-btn {
    opacity: 1;
  }

  .delete-btn:hover {
    background: rgba(239, 68, 68, 0.9);
  }

  .delete-btn svg {
    display: block;
  }

  /* Scrollbar styling */
  .media-grid::-webkit-scrollbar {
    width: 8px;
  }

  .media-grid::-webkit-scrollbar-track {
    background: #0f0f0f;
  }

  .media-grid::-webkit-scrollbar-thumb {
    background: #2d2d2d;
    border-radius: 4px;
  }

  .media-grid::-webkit-scrollbar-thumb:hover {
    background: #3d3d3d;
  }
</style>
