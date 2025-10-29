<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { save } from '@tauri-apps/plugin-dialog';
  import { timelineStore } from '../stores/timelineStore';
  import { mediaLibraryStore } from '../stores/mediaLibraryStore';

  // Export preset types
  interface ExportSettings {
    resolution: { width: number; height: number };
    framerate: number;
    video_codec: string;
    video_bitrate: number;
    audio_codec: string;
    audio_bitrate: number;
    preset: string;
  }

  interface ExportProgress {
    percentage: number;
    current_frame: number;
    fps: number;
    time_remaining_secs: number;
  }

  // Component state
  let showDialog = false;
  let presets: [string, ExportSettings][] = [];
  let selectedPreset = 0;
  let customSettings: ExportSettings | null = null;
  let outputPath = '';
  let exporting = false;
  let progress: ExportProgress = {
    percentage: 0,
    current_frame: 0,
    fps: 0,
    time_remaining_secs: 0,
  };

  // Export function
  export function open() {
    showDialog = true;
    loadPresets();
  }

  async function loadPresets() {
    try {
      presets = await invoke('get_export_presets');
      if (presets.length > 0) {
        customSettings = { ...presets[0][1] };
      }
    } catch (error) {
      console.error('Failed to load presets:', error);
    }
  }

  async function selectOutputFile() {
    try {
      const filePath = await save({
        filters: [{
          name: 'Video',
          extensions: ['mp4']
        }],
        defaultPath: 'export.mp4',
      });

      if (filePath) {
        outputPath = filePath;
      }
    } catch (error) {
      console.error('Failed to select output file:', error);
    }
  }

  async function startExport() {
    if (!outputPath) {
      alert('Please select an output file');
      return;
    }

    try {
      exporting = true;
      progress = { percentage: 0, current_frame: 0, fps: 0, time_remaining_secs: 0 };

      const timeline = $timelineStore;
      const settings = presets[selectedPreset][1];

      // Build media files map from media library store
      const mediaFiles = $mediaLibraryStore;
      const mediaFilesMap = mediaFiles.reduce((map, file) => {
        map[file.id] = file;
        return map;
      }, {} as Record<string, any>);

      // Listen for progress events
      const unlisten = await listen('export-progress', (event) => {
        progress = event.payload as ExportProgress;
      });

      // Listen for completion
      const unlistenComplete = await listen('export-complete', () => {
        exporting = false;
        alert('Export completed successfully!');
        close();
      });

      // Start export
      await invoke('export_timeline', {
        timeline,
        settings,
        outputPath,
        mediaFilesMap,
      });

      unlisten();
      unlistenComplete();
    } catch (error) {
      exporting = false;
      alert(`Export failed: ${error}`);
    }
  }

  async function cancelExport() {
    try {
      await invoke('cancel_export');
      exporting = false;
    } catch (error) {
      console.error('Failed to cancel export:', error);
    }
  }

  function close() {
    showDialog = false;
    exporting = false;
    progress = { percentage: 0, current_frame: 0, fps: 0, time_remaining_secs: 0 };
  }

  function selectPreset(index: number) {
    selectedPreset = index;
    customSettings = { ...presets[index][1] };
  }

  function formatTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  onMount(() => {
    loadPresets();
  });
</script>

{#if showDialog}
  <div class="dialog-overlay" on:click={close}>
    <div class="dialog-content" on:click|stopPropagation>
      <h2>Export Timeline</h2>

      {#if !exporting}
        <!-- Preset Selection -->
        <div class="section">
          <h3>Export Preset</h3>
          <div class="preset-buttons">
            {#each presets as [name, _], index}
              <button
                class="preset-btn"
                class:active={selectedPreset === index}
                on:click={() => selectPreset(index)}
              >
                {name}
              </button>
            {/each}
          </div>
        </div>

        <!-- Settings Preview -->
        {#if customSettings}
          <div class="section settings-preview">
            <h3>Settings</h3>
            <div class="settings-grid">
              <div class="setting">
                <span class="label">Resolution:</span>
                <span class="value">
                  {customSettings.resolution.width}x{customSettings.resolution.height}
                </span>
              </div>
              <div class="setting">
                <span class="label">Framerate:</span>
                <span class="value">{customSettings.framerate} fps</span>
              </div>
              <div class="setting">
                <span class="label">Video Codec:</span>
                <span class="value">{customSettings.video_codec}</span>
              </div>
              <div class="setting">
                <span class="label">Video Bitrate:</span>
                <span class="value">{customSettings.video_bitrate} kbps</span>
              </div>
              <div class="setting">
                <span class="label">Audio Codec:</span>
                <span class="label">{customSettings.audio_codec}</span>
              </div>
              <div class="setting">
                <span class="label">Audio Bitrate:</span>
                <span class="value">{customSettings.audio_bitrate} kbps</span>
              </div>
            </div>
          </div>
        {/if}

        <!-- Output File -->
        <div class="section">
          <h3>Output File</h3>
          <div class="file-select">
            <input
              type="text"
              readonly
              value={outputPath || 'No file selected'}
              placeholder="Select output file..."
            />
            <button on:click={selectOutputFile}>Browse...</button>
          </div>
        </div>

        <!-- Actions -->
        <div class="actions">
          <button class="btn-secondary" on:click={close}>Cancel</button>
          <button class="btn-primary" on:click={startExport} disabled={!outputPath}>
            Start Export
          </button>
        </div>
      {:else}
        <!-- Export Progress -->
        <div class="section progress-section">
          <h3>Exporting...</h3>

          <div class="progress-bar">
            <div class="progress-fill" style="width: {progress.percentage}%"></div>
          </div>

          <div class="progress-stats">
            <div class="stat">
              <span class="label">Progress:</span>
              <span class="value">{progress.percentage.toFixed(1)}%</span>
            </div>
            <div class="stat">
              <span class="label">Frame:</span>
              <span class="value">{progress.current_frame}</span>
            </div>
            <div class="stat">
              <span class="label">FPS:</span>
              <span class="value">{progress.fps.toFixed(1)}</span>
            </div>
            <div class="stat">
              <span class="label">Time Remaining:</span>
              <span class="value">{formatTime(progress.time_remaining_secs)}</span>
            </div>
          </div>

          <div class="actions">
            <button class="btn-danger" on:click={cancelExport}>Cancel Export</button>
          </div>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .dialog-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
  }

  .dialog-content {
    background: #1a1a1a;
    border-radius: 8px;
    padding: 2rem;
    max-width: 600px;
    width: 90%;
    max-height: 90vh;
    overflow-y: auto;
    border: 1px solid #2d2d2d;
  }

  h2 {
    margin-top: 0;
    margin-bottom: 1.5rem;
    color: #fff;
    font-size: 1.8rem;
  }

  h3 {
    margin-top: 0;
    margin-bottom: 1rem;
    color: #ccc;
    font-size: 1.2rem;
  }

  .section {
    margin-bottom: 2rem;
  }

  .preset-buttons {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .preset-btn {
    padding: 0.75rem 1.5rem;
    background: #2d2d2d;
    color: #fff;
    border: 2px solid #3d3d3d;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9rem;
    transition: all 0.2s;
  }

  .preset-btn:hover {
    background: #3d3d3d;
  }

  .preset-btn.active {
    background: #667eea;
    border-color: #667eea;
  }

  .settings-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
  }

  .setting {
    display: flex;
    justify-content: space-between;
    padding: 0.5rem;
    background: #2d2d2d;
    border-radius: 4px;
  }

  .label {
    color: #aaa;
  }

  .value {
    color: #fff;
    font-weight: 500;
  }

  .file-select {
    display: flex;
    gap: 0.5rem;
  }

  .file-select input {
    flex: 1;
    padding: 0.75rem;
    background: #2d2d2d;
    border: 1px solid #3d3d3d;
    border-radius: 4px;
    color: #fff;
  }

  .file-select button {
    padding: 0.75rem 1.5rem;
    background: #3d3d3d;
    color: #fff;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    transition: background 0.2s;
  }

  .file-select button:hover {
    background: #4d4d4d;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 2rem;
  }

  button {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9rem;
    transition: all 0.2s;
  }

  .btn-primary {
    background: #667eea;
    color: #fff;
  }

  .btn-primary:hover:not(:disabled) {
    background: #5568d3;
  }

  .btn-primary:disabled {
    background: #3d3d3d;
    cursor: not-allowed;
    opacity: 0.5;
  }

  .btn-secondary {
    background: #3d3d3d;
    color: #fff;
  }

  .btn-secondary:hover {
    background: #4d4d4d;
  }

  .btn-danger {
    background: #e53e3e;
    color: #fff;
  }

  .btn-danger:hover {
    background: #c53030;
  }

  .progress-section {
    min-height: 300px;
    display: flex;
    flex-direction: column;
    justify-content: center;
  }

  .progress-bar {
    width: 100%;
    height: 24px;
    background: #2d2d2d;
    border-radius: 12px;
    overflow: hidden;
    margin: 1.5rem 0;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #667eea 0%, #764ba2 100%);
    transition: width 0.3s ease;
  }

  .progress-stats {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
    margin-bottom: 2rem;
  }

  .stat {
    display: flex;
    justify-content: space-between;
    padding: 1rem;
    background: #2d2d2d;
    border-radius: 4px;
  }

  .stat .label {
    color: #aaa;
  }

  .stat .value {
    color: #fff;
    font-weight: 600;
  }
</style>
