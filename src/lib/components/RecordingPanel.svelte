<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import {
    recordingState,
    availableSources,
    selectedSource,
    recordingConfig,
    isLoadingSources,
    hasPermissions,
    recordingError,
    isRecording,
    isPaused,
    isIdle,
    isFinalizing,
    loadRecordingSources,
    checkRecordingPermissions,
    requestRecordingPermissions,
    startRecording,
    stopRecording,
    initializeRecordingListeners,
    cleanupRecordingListeners,
    formatRecordingDuration,
    type AudioInputType,
  } from '../stores/recordingStore';
  import { importMediaFile } from '../stores/mediaLibraryStore';

  // Component state
  let showPanel = false;
  let permissionCheckDone = false;
  let showPermissionDialog = false;

  // Quality options
  const qualityOptions = ['Low', 'Medium', 'High', 'Ultra'];
  const framerateOptions = [15, 24, 30, 60];
  const audioOptions: { value: AudioInputType; label: string }[] = [
    { value: 'None', label: 'No Audio' },
    { value: 'Microphone', label: 'Microphone' },
    { value: 'SystemAudio', label: 'System Audio' },
    { value: 'Both', label: 'System + Microphone' },
  ];

  export function open() {
    showPanel = true;
    initPanel();
  }

  export function close() {
    showPanel = false;
    if ($isRecording) {
      handleStop();
    }
  }

  async function initPanel() {
    // Initialize event listeners
    await initializeRecordingListeners();

    // Check permissions
    if (!permissionCheckDone) {
      const hasPerms = await checkRecordingPermissions();
      permissionCheckDone = true;

      if (!hasPerms) {
        showPermissionDialog = true;
      } else {
        await loadRecordingSources();
      }
    }
  }

  async function handleRequestPermissions() {
    const granted = await requestRecordingPermissions();
    if (granted) {
      showPermissionDialog = false;
      await loadRecordingSources();
    } else {
      // Show platform-specific instructions
      alert('Please grant screen recording permission in System Preferences > Privacy & Security > Screen Recording');
    }
  }

  async function handleStart() {
    const success = await startRecording();
    if (!success && $recordingError) {
      alert(`Failed to start recording: ${$recordingError}`);
    }
  }

  async function handleStop() {
    const filePath = await stopRecording();
    if (filePath) {
      // Auto-import the recorded file to media library
      const mediaFile = await importMediaFile(filePath);
      if (mediaFile) {
        alert(`Recording saved and imported to media library!`);
      }
    } else if ($recordingError) {
      alert(`Failed to stop recording: ${$recordingError}`);
    }
  }

  function updateConfig(key: string, value: any) {
    recordingConfig.update(config => ({
      ...config,
      [key]: value,
    }));
  }

  onMount(() => {
    initPanel();
  });

  onDestroy(() => {
    cleanupRecordingListeners();
  });
</script>

{#if showPanel}
  <div class="panel-overlay" on:click={close}>
    <div class="panel-content" on:click|stopPropagation>
      <div class="panel-header">
        <h2>Screen Recording</h2>
        <button class="close-btn" on:click={close}>×</button>
      </div>

      {#if showPermissionDialog}
        <!-- Permission Request Dialog -->
        <div class="permission-section">
          <div class="permission-icon">🔒</div>
          <h3>Screen Recording Permission Required</h3>
          <p>ClipForge needs permission to record your screen.</p>
          <p class="permission-hint">
            Click "Request Permission" and grant access in System Preferences.
          </p>
          <div class="actions">
            <button class="btn-primary" on:click={handleRequestPermissions}>
              Request Permission
            </button>
            <button class="btn-secondary" on:click={close}>Cancel</button>
          </div>
        </div>
      {:else if $isLoadingSources}
        <!-- Loading State -->
        <div class="loading-section">
          <div class="spinner"></div>
          <p>Loading recording sources...</p>
        </div>
      {:else}
        <!-- Recording Controls -->
        <div class="recording-controls">
          {#if !$isRecording && !$isFinalizing}
            <!-- Source Selection -->
            <div class="section">
              <label for="source-select">Recording Source</label>
              <select
                id="source-select"
                bind:value={$selectedSource}
                disabled={$isRecording}
              >
                {#each $availableSources as source}
                  <option value={source.id}>
                    {source.type === 'screen' ? 'Screen' : 'Window'}: {source.name}
                  </option>
                {/each}
              </select>
            </div>

            <!-- Audio Input -->
            <div class="section">
              <label for="audio-select">Audio Input</label>
              <select
                id="audio-select"
                value={$recordingConfig.audio_input || 'None'}
                on:change={(e) => updateConfig('audio_input', e.target.value)}
                disabled={$isRecording}
              >
                {#each audioOptions as option}
                  <option value={option.value}>{option.label}</option>
                {/each}
              </select>
            </div>

            <!-- Quality Settings -->
            <div class="section">
              <label for="quality-select">Quality</label>
              <div class="quality-grid">
                {#each qualityOptions as quality}
                  <button
                    class="quality-btn"
                    class:active={$recordingConfig.quality === quality}
                    on:click={() => updateConfig('quality', quality)}
                    disabled={$isRecording}
                  >
                    {quality}
                  </button>
                {/each}
              </div>
            </div>

            <!-- Framerate -->
            <div class="section">
              <label for="framerate-select">Framerate</label>
              <div class="framerate-grid">
                {#each framerateOptions as fps}
                  <button
                    class="framerate-btn"
                    class:active={$recordingConfig.framerate === fps}
                    on:click={() => updateConfig('framerate', fps)}
                    disabled={$isRecording}
                  >
                    {fps} fps
                  </button>
                {/each}
              </div>
            </div>

            <!-- Capture Cursor -->
            <div class="section checkbox-section">
              <label>
                <input
                  type="checkbox"
                  checked={$recordingConfig.capture_cursor !== false}
                  on:change={(e) => updateConfig('capture_cursor', e.target.checked)}
                  disabled={$isRecording}
                />
                <span>Show cursor in recording</span>
              </label>
            </div>

            <!-- Start Recording Button -->
            <div class="actions">
              <button
                class="btn-primary btn-record"
                on:click={handleStart}
                disabled={!$selectedSource || $isRecording}
              >
                <span class="record-icon">●</span>
                Start Recording
              </button>
            </div>
          {:else if $isRecording}
            <!-- Recording In Progress -->
            <div class="recording-active">
              <div class="recording-indicator">
                <span class="recording-dot"></span>
                <span class="recording-text">RECORDING</span>
              </div>

              <div class="duration-display">
                {formatRecordingDuration($recordingState.duration)}
              </div>

              <div class="recording-info">
                <div class="info-item">
                  <span class="label">Source:</span>
                  <span class="value">
                    {$availableSources.find(s => s.id === $selectedSource)?.name || 'Unknown'}
                  </span>
                </div>
                <div class="info-item">
                  <span class="label">Audio:</span>
                  <span class="value">{$recordingConfig.audio_input || 'None'}</span>
                </div>
                <div class="info-item">
                  <span class="label">Quality:</span>
                  <span class="value">{$recordingConfig.quality || 'High'}</span>
                </div>
                <div class="info-item">
                  <span class="label">FPS:</span>
                  <span class="value">{$recordingConfig.framerate || 30}</span>
                </div>
              </div>

              <div class="actions">
                <button class="btn-danger btn-stop" on:click={handleStop}>
                  <span class="stop-icon">■</span>
                  Stop Recording
                </button>
              </div>
            </div>
          {:else if $isFinalizing}
            <!-- Finalizing -->
            <div class="finalizing-section">
              <div class="spinner"></div>
              <h3>Finalizing Recording...</h3>
              <p>Please wait while we process your recording.</p>
            </div>
          {/if}

          {#if $recordingError}
            <div class="error-message">
              {$recordingError}
            </div>
          {/if}
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .panel-overlay {
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

  .panel-content {
    background: #1a1a1a;
    border-radius: 8px;
    max-width: 500px;
    width: 90%;
    max-height: 90vh;
    overflow-y: auto;
    border: 1px solid #2d2d2d;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem 2rem;
    border-bottom: 1px solid #2d2d2d;
  }

  .panel-header h2 {
    margin: 0;
    color: #fff;
    font-size: 1.5rem;
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

  .recording-controls {
    padding: 2rem;
  }

  .section {
    margin-bottom: 1.5rem;
  }

  label {
    display: block;
    color: #aaa;
    margin-bottom: 0.5rem;
    font-size: 0.9rem;
  }

  select {
    width: 100%;
    padding: 0.75rem;
    background: #2d2d2d;
    border: 1px solid #3d3d3d;
    border-radius: 4px;
    color: #fff;
    font-size: 0.9rem;
    cursor: pointer;
  }

  select:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .quality-grid,
  .framerate-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 0.5rem;
  }

  .quality-btn,
  .framerate-btn {
    padding: 0.75rem;
    background: #2d2d2d;
    color: #fff;
    border: 2px solid #3d3d3d;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9rem;
    transition: all 0.2s;
  }

  .quality-btn:hover:not(:disabled),
  .framerate-btn:hover:not(:disabled) {
    background: #3d3d3d;
  }

  .quality-btn.active,
  .framerate-btn.active {
    background: #667eea;
    border-color: #667eea;
  }

  .quality-btn:disabled,
  .framerate-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .checkbox-section label {
    display: flex;
    align-items: center;
    cursor: pointer;
  }

  .checkbox-section input[type="checkbox"] {
    margin-right: 0.5rem;
    width: 18px;
    height: 18px;
    cursor: pointer;
  }

  .checkbox-section span {
    color: #fff;
  }

  .actions {
    display: flex;
    justify-content: center;
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
    font-weight: 500;
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

  .btn-record {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 1rem 2rem;
    font-size: 1rem;
  }

  .record-icon {
    color: #e53e3e;
    font-size: 1.2rem;
  }

  .btn-stop {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 1rem 2rem;
    font-size: 1rem;
  }

  .stop-icon {
    font-size: 1rem;
  }

  /* Recording Active State */
  .recording-active {
    text-align: center;
  }

  .recording-indicator {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    margin-bottom: 1rem;
  }

  .recording-dot {
    width: 12px;
    height: 12px;
    background: #e53e3e;
    border-radius: 50%;
    animation: pulse 1.5s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
      transform: scale(1);
    }
    50% {
      opacity: 0.5;
      transform: scale(1.2);
    }
  }

  .recording-text {
    color: #e53e3e;
    font-weight: 600;
    font-size: 0.9rem;
    letter-spacing: 1px;
  }

  .duration-display {
    font-size: 3rem;
    font-weight: 300;
    color: #fff;
    font-variant-numeric: tabular-nums;
    margin: 2rem 0;
  }

  .recording-info {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
    margin: 2rem 0;
  }

  .info-item {
    display: flex;
    flex-direction: column;
    padding: 0.75rem;
    background: #2d2d2d;
    border-radius: 4px;
  }

  .info-item .label {
    color: #aaa;
    font-size: 0.8rem;
    margin-bottom: 0.25rem;
  }

  .info-item .value {
    color: #fff;
    font-weight: 500;
  }

  /* Permission Dialog */
  .permission-section {
    padding: 3rem 2rem;
    text-align: center;
  }

  .permission-icon {
    font-size: 4rem;
    margin-bottom: 1rem;
  }

  .permission-section h3 {
    color: #fff;
    margin: 1rem 0;
    font-size: 1.3rem;
  }

  .permission-section p {
    color: #aaa;
    margin: 0.5rem 0;
    line-height: 1.6;
  }

  .permission-hint {
    font-size: 0.9rem;
    margin-top: 1rem;
  }

  /* Loading State */
  .loading-section,
  .finalizing-section {
    padding: 3rem 2rem;
    text-align: center;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid #2d2d2d;
    border-top: 3px solid #667eea;
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin: 0 auto 1rem;
  }

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }

  .loading-section p,
  .finalizing-section p {
    color: #aaa;
    margin-top: 0.5rem;
  }

  .finalizing-section h3 {
    color: #fff;
    margin: 1rem 0;
  }

  /* Error Message */
  .error-message {
    margin-top: 1rem;
    padding: 1rem;
    background: rgba(229, 62, 62, 0.1);
    border: 1px solid #e53e3e;
    border-radius: 4px;
    color: #e53e3e;
    text-align: center;
    font-size: 0.9rem;
  }
</style>
