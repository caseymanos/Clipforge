<script lang="ts">
  import { subtitleStore, currentSubtitle, transcribeTimelineAudio, updateSubtitleSegment, deleteSubtitleSegment, toggleSubtitles, exportSubtitlesSRT, importSubtitlesSRT, setEditingSegment, setOpenAIApiKey, checkSubtitleAvailable, updateSubtitleStyle } from '../stores/subtitleStore';
  import { timelineStore } from '../stores/timelineStore';
  import { mediaLibraryStore } from '../stores/mediaLibraryStore';
  import { playheadTime } from '../stores/timelineStore';
  import { onMount } from 'svelte';
  import { open, save } from '@tauri-apps/plugin-dialog';

  let apiKey = '';
  let showApiKeyInput = false;
  let selectedLanguage = 'en';
  let editingText: { [key: number]: string } = {};

  onMount(async () => {
    // Check if API key is already configured
    await checkSubtitleAvailable();
  });

  async function handleSetApiKey() {
    if (!apiKey) return;
    try {
      await setOpenAIApiKey(apiKey);
      showApiKeyInput = false;
      apiKey = '';
      alert('API key set successfully!');
    } catch (error) {
      alert(`Failed to set API key: ${error}`);
    }
  }

  async function handleTranscribe() {
    const timeline = $timelineStore;
    const mediaFiles = $mediaLibraryStore;

    if (!timeline) {
      alert('No timeline loaded');
      return;
    }

    if (!$subtitleStore.apiKeyConfigured) {
      alert('Please set your OpenAI API key first');
      showApiKeyInput = true;
      return;
    }

    try {
      await transcribeTimelineAudio(timeline, mediaFiles, selectedLanguage);
    } catch (error) {
      alert(`Transcription failed: ${error}`);
    }
  }

  async function handleExport() {
    try {
      const path = await save({
        filters: [{
          name: 'SRT',
          extensions: ['srt']
        }],
        defaultPath: 'subtitles.srt'
      });

      if (path) {
        await exportSubtitlesSRT(path);
        alert('Subtitles exported successfully!');
      }
    } catch (error) {
      alert(`Export failed: ${error}`);
    }
  }

  async function handleImport() {
    try {
      const selected = await open({
        filters: [{
          name: 'SRT',
          extensions: ['srt']
        }],
        multiple: false
      });

      if (selected && typeof selected === 'string') {
        await importSubtitlesSRT(selected, selectedLanguage);
      }
    } catch (error) {
      alert(`Import failed: ${error}`);
    }
  }

  function startEditing(segmentId: number, currentText: string) {
    setEditingSegment(segmentId);
    editingText[segmentId] = currentText;
  }

  async function saveEdit(segmentId: number) {
    const timeline = $timelineStore;
    if (!timeline) return;

    const newText = editingText[segmentId];
    if (newText !== undefined) {
      try {
        await updateSubtitleSegment(timeline.id, segmentId, newText);
        setEditingSegment(null);
        delete editingText[segmentId];
      } catch (error) {
        alert(`Failed to save: ${error}`);
      }
    }
  }

  function cancelEdit(segmentId: number) {
    setEditingSegment(null);
    delete editingText[segmentId];
  }

  async function handleDelete(segmentId: number) {
    const timeline = $timelineStore;
    if (!timeline) return;

    if (confirm('Delete this subtitle segment?')) {
      try {
        await deleteSubtitleSegment(timeline.id, segmentId);
      } catch (error) {
        alert(`Failed to delete: ${error}`);
      }
    }
  }

  function formatTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  }

  async function handleToggle() {
    const timeline = $timelineStore;
    if (!timeline) return;

    try {
      await toggleSubtitles(timeline.id, !$subtitleStore.enabled);
    } catch (error) {
      alert(`Failed to toggle: ${error}`);
    }
  }
</script>

<div class="subtitle-editor">
  <div class="header">
    <h3>Subtitles</h3>
    <label class="toggle">
      <input type="checkbox" checked={$subtitleStore.enabled} on:change={handleToggle} />
      <span>Enable</span>
    </label>
  </div>

  {#if !$subtitleStore.apiKeyConfigured}
    <div class="api-key-section">
      {#if !showApiKeyInput}
        <button on:click={() => showApiKeyInput = true}>Set OpenAI API Key</button>
      {:else}
        <div class="api-key-input">
          <input
            type="password"
            bind:value={apiKey}
            placeholder="sk-..."
          />
          <button on:click={handleSetApiKey}>Save</button>
          <button on:click={() => showApiKeyInput = false}>Cancel</button>
        </div>
      {/if}
    </div>
  {/if}

  <div class="controls">
    <div class="control-group">
      <select bind:value={selectedLanguage}>
        <option value="en">English</option>
        <option value="es">Spanish</option>
        <option value="fr">French</option>
        <option value="de">German</option>
        <option value="auto">Auto-detect</option>
      </select>
      <button
        on:click={handleTranscribe}
        disabled={$subtitleStore.isTranscribing || !$subtitleStore.apiKeyConfigured}
      >
        {$subtitleStore.isTranscribing ? 'Transcribing...' : 'Transcribe Timeline'}
      </button>
    </div>

    <div class="control-group">
      <button on:click={handleImport} disabled={$subtitleStore.isTranscribing}>
        Import SRT
      </button>
      <button
        on:click={handleExport}
        disabled={!$subtitleStore.currentTrack}
      >
        Export SRT
      </button>
    </div>
  </div>

  {#if $subtitleStore.currentTrack}
    <div class="style-section">
      <h4>Subtitle Style</h4>
      <div class="style-controls">
        <label class="style-control">
          <span class="style-label">Font Size: {$subtitleStore.currentTrack.style.font_size}px</span>
          <input
            type="range"
            min="12"
            max="48"
            step="1"
            value={$subtitleStore.currentTrack.style.font_size}
            on:input={(e) => updateSubtitleStyle({ font_size: parseInt(e.currentTarget.value) })}
          />
        </label>
      </div>
    </div>
  {/if}

  {#if $subtitleStore.isTranscribing && $subtitleStore.transcriptionProgress}
    <div class="progress">
      <div class="progress-bar">
        <div
          class="progress-fill"
          style="width: {$subtitleStore.transcriptionProgress.progress * 100}%"
        ></div>
      </div>
      <div class="progress-text">
        {$subtitleStore.transcriptionProgress.stage} - {Math.round($subtitleStore.transcriptionProgress.progress * 100)}%
      </div>
    </div>
  {/if}

  {#if $subtitleStore.currentTrack}
    <div class="segments">
      <div class="segments-header">
        <span>{$subtitleStore.currentTrack.segments.length} segments</span>
        <span>Language: {$subtitleStore.currentTrack.language}</span>
      </div>

      <div class="segments-list">
        {#each $subtitleStore.currentTrack.segments as segment}
          <div
            class="segment"
            class:active={$currentSubtitle?.id === segment.id}
            class:editing={$subtitleStore.editingSegmentId === segment.id}
          >
            <div class="segment-time">
              {formatTime(segment.start_time)} - {formatTime(segment.end_time)}
            </div>

            {#if $subtitleStore.editingSegmentId === segment.id}
              <textarea
                bind:value={editingText[segment.id]}
                rows="3"
                on:keydown={(e) => {
                  if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
                    saveEdit(segment.id);
                  } else if (e.key === 'Escape') {
                    cancelEdit(segment.id);
                  }
                }}
              ></textarea>
              <div class="segment-actions">
                <button on:click={() => saveEdit(segment.id)}>Save</button>
                <button on:click={() => cancelEdit(segment.id)}>Cancel</button>
              </div>
            {:else}
              <div class="segment-text">
                {segment.text}
              </div>
              <div class="segment-actions">
                <button on:click={() => startEditing(segment.id, segment.text)}>Edit</button>
                <button on:click={() => handleDelete(segment.id)}>Delete</button>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  {:else if !$subtitleStore.isTranscribing}
    <div class="empty-state">
      <p>No subtitles yet. Transcribe your timeline or import an SRT file.</p>
    </div>
  {/if}
</div>

<style>
  .subtitle-editor {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    height: 100%;
    padding: 1rem;
    background: #1a1a1a;
    color: #e0e0e0;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  h3 {
    margin: 0;
    font-size: 1.2rem;
  }

  .toggle {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
  }

  .api-key-section {
    padding: 1rem;
    background: #2a2a2a;
    border-radius: 0.25rem;
  }

  .api-key-input {
    display: flex;
    gap: 0.5rem;
  }

  .api-key-input input {
    flex: 1;
    padding: 0.5rem;
    background: #1a1a1a;
    border: 1px solid #444;
    border-radius: 0.25rem;
    color: #e0e0e0;
  }

  .controls {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .control-group {
    display: flex;
    gap: 0.5rem;
  }

  select, button {
    padding: 0.5rem 1rem;
    background: #2a2a2a;
    border: 1px solid #444;
    border-radius: 0.25rem;
    color: #e0e0e0;
    cursor: pointer;
  }

  button:hover:not(:disabled) {
    background: #3a3a3a;
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .progress {
    padding: 1rem;
    background: #2a2a2a;
    border-radius: 0.25rem;
  }

  .progress-bar {
    height: 0.5rem;
    background: #444;
    border-radius: 0.25rem;
    overflow: hidden;
    margin-bottom: 0.5rem;
  }

  .progress-fill {
    height: 100%;
    background: #4a9eff;
    transition: width 0.3s ease;
  }

  .progress-text {
    font-size: 0.875rem;
    color: #aaa;
  }

  .segments {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .segments-header {
    display: flex;
    justify-content: space-between;
    padding: 0.5rem;
    background: #2a2a2a;
    border-radius: 0.25rem;
    font-size: 0.875rem;
    color: #aaa;
  }

  .segments-list {
    flex: 1;
    overflow-y: auto;
    margin-top: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .segment {
    padding: 0.75rem;
    background: #2a2a2a;
    border: 1px solid #444;
    border-radius: 0.25rem;
    transition: border-color 0.2s;
  }

  .segment.active {
    border-color: #4a9eff;
    background: #2a3a4a;
  }

  .segment.editing {
    border-color: #ffa500;
  }

  .segment-time {
    font-size: 0.75rem;
    color: #aaa;
    margin-bottom: 0.5rem;
  }

  .segment-text {
    margin-bottom: 0.5rem;
    line-height: 1.4;
  }

  textarea {
    width: 100%;
    padding: 0.5rem;
    background: #1a1a1a;
    border: 1px solid #444;
    border-radius: 0.25rem;
    color: #e0e0e0;
    font-family: inherit;
    resize: vertical;
    margin-bottom: 0.5rem;
  }

  .segment-actions {
    display: flex;
    gap: 0.5rem;
  }

  .segment-actions button {
    padding: 0.25rem 0.75rem;
    font-size: 0.875rem;
  }

  .empty-state {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #aaa;
    text-align: center;
    padding: 2rem;
  }

  .style-section {
    padding: 1rem;
    background: #2a2a2a;
    border-radius: 0.25rem;
  }

  .style-section h4 {
    margin: 0 0 0.75rem 0;
    font-size: 0.95rem;
    color: #e0e0e0;
  }

  .style-controls {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .style-control {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .style-label {
    font-size: 0.875rem;
    color: #aaa;
  }

  .style-control input[type="range"] {
    width: 100%;
    height: 6px;
    background: #444;
    border-radius: 3px;
    outline: none;
    -webkit-appearance: none;
  }

  .style-control input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 16px;
    height: 16px;
    background: #4a9eff;
    border-radius: 50%;
    cursor: pointer;
  }

  .style-control input[type="range"]::-moz-range-thumb {
    width: 16px;
    height: 16px;
    background: #4a9eff;
    border-radius: 50%;
    cursor: pointer;
    border: none;
  }
</style>
