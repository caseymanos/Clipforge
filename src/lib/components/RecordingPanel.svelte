<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { convertFileSrc } from '@tauri-apps/api/core';
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
    loadAudioDevices,
    availableAudioDevices,
    checkRecordingPermissions,
    requestRecordingPermissions,
    startRecording,
    stopRecording,
    initializeRecordingListeners,
    cleanupRecordingListeners,
    formatRecordingDuration,
    recordingMode,
    selectedWebcam,
    isDualMode,
    isWebcamMode,
    webcamSources,
    screenSources,
    webcamOverlayConfig,
    isCompositing,
    compositingProgress,
    compositeWebcamRecording,
    type AudioInputType,
    type CropRegion,
    type RecordingMode,
    type WebcamPosition,
    type WebcamShape,
  } from '../stores/recordingStore';
  import { importMediaFile } from '../stores/mediaLibraryStore';
  import { addMediaFileToTimeline } from '../stores/timelineStore';
  import { invoke } from '@tauri-apps/api/core';
  import CropEditor from './CropEditor.svelte';

  // Component state
  let showPanel = false;
  let permissionCheckDone = false;
  let showPermissionDialog = false;

  // Crop region state
  let enableCrop = false;
  let currentCropRegion: CropRegion | null = null;
  let showCropEditor = false;

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
    // Sources already loaded from onMount - no need to reload
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
        return;
      }
    }

    // Always reload sources when panel opens to ensure fresh data
    if ($hasPermissions) {
      await loadRecordingSources('all');
      await loadAudioDevices();
    }
  }

  async function handleRequestPermissions() {
    const granted = await requestRecordingPermissions();
    if (granted) {
      showPermissionDialog = false;
      await loadRecordingSources('all');
    } else {
      // Show platform-specific instructions
      alert('Please grant screen recording permission in System Preferences > Privacy & Security > Screen Recording');
    }
  }

  function handleOpenCropEditor() {
    showCropEditor = true;
  }

  function handleCropApply(event: CustomEvent<CropRegion>) {
    currentCropRegion = event.detail;
    showCropEditor = false;
  }

  function handleCropCancel() {
    showCropEditor = false;
  }

  async function handleStart() {
    // Update config with crop region if enabled
    if (enableCrop && currentCropRegion) {
      updateConfig('crop_region', currentCropRegion);
    } else {
      updateConfig('crop_region', undefined);
    }

    const success = await startRecording();
    if (success) {
      // Auto-close panel when recording starts successfully
      showPanel = false;
    } else if ($recordingError) {
      alert(`Failed to start recording: ${$recordingError}`);
    }
  }

  async function handleStop() {
    console.log('[RecordingPanel] handleStop called');
    const filePath = await stopRecording();
    console.log('[RecordingPanel] Recording stopped, filePath:', filePath);
    console.log('[RecordingPanel] Current recordingMode:', $recordingMode);

    if (filePath && $recordingMode === 'ScreenAndWebcam') {
      console.log('[RecordingPanel] Dual mode detected - starting compositing workflow');

      // Auto-composite the screen and webcam recordings
      const timestamp = new Date().toISOString().replace(/[:.]/g, '-').substring(0, 19);
      const outputPath = filePath.replace('screen-', 'composite-');
      const webcamPath = filePath.replace('screen-', 'webcam-');

      console.log('[RecordingPanel] Paths calculated:');
      console.log('  Screen:', filePath);
      console.log('  Webcam:', webcamPath);
      console.log('  Output:', outputPath);

      try {
        // Check if webcam file exists
        console.log('[RecordingPanel] Checking if webcam file exists...');
        const webcamExists = await invoke<boolean>('file_exists', { path: webcamPath });
        console.log('[RecordingPanel] Webcam file exists:', webcamExists);

        if (!webcamExists) {
          throw new Error(`Webcam recording file not found: ${webcamPath}`);
        }

        console.log('[RecordingPanel] Starting compositing with config:', $webcamOverlayConfig);
        const compositePath = await compositeWebcamRecording(
          filePath,
          webcamPath,
          outputPath,
          $webcamOverlayConfig
        );
        console.log('[RecordingPanel] Compositing complete:', compositePath);

        if (compositePath) {
          console.log('[RecordingPanel] Importing composite to media library');
          const mediaFile = await importMediaFile(compositePath);
          console.log('[RecordingPanel] Imported media file:', mediaFile);

          if (mediaFile) {
            console.log('[RecordingPanel] Adding to timeline');
            await addMediaFileToTimeline(mediaFile);
            alert(`Recording composited and added to timeline!`);
          }
        }
      } catch (error) {
        console.error('[RecordingPanel] Failed to composite webcam recording:', error);
        console.error('[RecordingPanel] Error details:', error);

        // Fall back to importing the screen recording only
        console.log('[RecordingPanel] Falling back to screen-only import');
        const mediaFile = await importMediaFile(filePath);
        if (mediaFile) {
          console.log('[RecordingPanel] Adding screen-only recording to timeline');
          await addMediaFileToTimeline(mediaFile);
          alert(`Recording saved and added to timeline (compositing failed: ${error.message || error})`);
        }
      }
    } else if (filePath) {
      // Auto-import the recorded file to media library (single source recording)
      console.log('[RecordingPanel] Single source recording - importing to media library');
      const mediaFile = await importMediaFile(filePath);
      console.log('[RecordingPanel] Imported media file:', mediaFile);

      if (mediaFile) {
        console.log('[RecordingPanel] Adding to timeline');
        await addMediaFileToTimeline(mediaFile);
        alert(`Recording saved and added to timeline!`);
      }
    } else if ($recordingError) {
      console.error('[RecordingPanel] Recording error:', $recordingError);
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

  // Get the currently selected source for the crop editor
  $: currentSource = $availableSources.find(s => s.id === $selectedSource);
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
            <!-- Screen/Window Source Selection (shown for ScreenOnly and ScreenAndWebcam modes) -->
            {#if $recordingMode === 'ScreenOnly' || $recordingMode === 'ScreenAndWebcam'}
              <div class="section">
                <label>Recording Source</label>
                <div class="sources-grid">
                  {#each $screenSources as source}
                    <button
                      class="source-card"
                      class:selected={$selectedSource === source.id}
                      on:click={() => selectedSource.set(source.id)}
                      disabled={$isRecording}
                    >
                      {#if source.preview_path}
                        <img
                          src={convertFileSrc(source.preview_path)}
                          alt={source.name}
                          class="source-preview"
                        />
                      {:else}
                        <div class="source-placeholder">
                          {source.type === 'screen' ? '🖥️' : '🪟'}
                        </div>
                      {/if}
                      <div class="source-info">
                        <div class="source-name">{source.name}</div>
                        {#if source.width && source.height}
                          <div class="source-resolution">{source.width}×{source.height}</div>
                        {/if}
                      </div>
                    </button>
                  {/each}
                </div>
              </div>
            {/if}

            <!-- Recording Mode Selection -->
            <div class="section">
              <label class="section-label">Recording Mode</label>
              <div class="mode-selector">
                <button
                  class="mode-btn"
                  class:active={$recordingMode === 'ScreenOnly'}
                  on:click={() => recordingMode.set('ScreenOnly')}
                  disabled={$isRecording}
                >
                  <span class="mode-icon">🖥️</span>
                  <span class="mode-label">Screen Only</span>
                </button>
                <button
                  class="mode-btn"
                  class:active={$recordingMode === 'WebcamOnly'}
                  on:click={() => recordingMode.set('WebcamOnly')}
                  disabled={$isRecording || $webcamSources.length === 0}
                >
                  <span class="mode-icon">📹</span>
                  <span class="mode-label">Webcam Only</span>
                </button>
                <button
                  class="mode-btn"
                  class:active={$recordingMode === 'ScreenAndWebcam'}
                  on:click={() => recordingMode.set('ScreenAndWebcam')}
                  disabled={$isRecording || $webcamSources.length === 0}
                >
                  <span class="mode-icon">🖥️+📹</span>
                  <span class="mode-label">Both</span>
                </button>
              </div>
            </div>

            <!-- Webcam Selection (shown if webcam mode enabled) -->
            {#if $isWebcamMode}
              <div class="section">
                <label class="section-label">Select Webcam</label>
                {#if $webcamSources.length > 0}
                  <div class="sources-grid">
                    {#each $webcamSources as webcam}
                      <button
                        class="source-card"
                        class:selected={$selectedWebcam === webcam.id}
                        on:click={() => selectedWebcam.set(webcam.id)}
                        disabled={$isRecording}
                      >
                        {#if webcam.preview_path}
                          <img
                            src={convertFileSrc(webcam.preview_path)}
                            alt={webcam.name}
                            class="source-preview"
                          />
                        {:else}
                          <div class="source-placeholder">
                            <span class="placeholder-icon">📹</span>
                          </div>
                        {/if}
                        <div class="source-info">
                          <div class="source-name">{webcam.name}</div>
                        </div>
                      </button>
                    {/each}
                  </div>
                {:else}
                  <!-- Loading or no webcams found -->
                  {#if $isLoadingSources}
                    <div class="loading-section">
                      <div class="spinner"></div>
                      <p>Loading webcam sources...</p>
                    </div>
                  {:else}
                    <div class="warning-message">
                      No webcam detected. Please connect a camera to use webcam recording modes.
                    </div>
                  {/if}
                {/if}
              </div>
            {/if}

            <!-- Webcam Overlay Settings (shown only for ScreenAndWebcam mode) -->
            {#if $isDualMode}
              <div class="section">
                <label class="section-label">Webcam Overlay Settings</label>

                <div class="overlay-settings">
                  <div class="setting-group">
                    <label class="setting-label">Position</label>
                    <div class="position-grid">
                      <button
                        class="position-btn"
                        class:selected={$webcamOverlayConfig.position === 'TopLeft'}
                        on:click={() => webcamOverlayConfig.update(c => ({...c, position: 'TopLeft'}))}
                        disabled={$isRecording}
                        title="Top Left"
                      >
                        <span class="position-icon">↖</span>
                      </button>
                      <button
                        class="position-btn"
                        class:selected={$webcamOverlayConfig.position === 'TopRight'}
                        on:click={() => webcamOverlayConfig.update(c => ({...c, position: 'TopRight'}))}
                        disabled={$isRecording}
                        title="Top Right"
                      >
                        <span class="position-icon">↗</span>
                      </button>
                      <button
                        class="position-btn"
                        class:selected={$webcamOverlayConfig.position === 'BottomLeft'}
                        on:click={() => webcamOverlayConfig.update(c => ({...c, position: 'BottomLeft'}))}
                        disabled={$isRecording}
                        title="Bottom Left"
                      >
                        <span class="position-icon">↙</span>
                      </button>
                      <button
                        class="position-btn"
                        class:selected={$webcamOverlayConfig.position === 'BottomRight'}
                        on:click={() => webcamOverlayConfig.update(c => ({...c, position: 'BottomRight'}))}
                        disabled={$isRecording}
                        title="Bottom Right"
                      >
                        <span class="position-icon">↘</span>
                      </button>
                    </div>
                  </div>

                  <div class="setting-group">
                    <label class="setting-label">Shape</label>
                    <div class="shape-buttons">
                      <button
                        class="shape-btn"
                        class:selected={$webcamOverlayConfig.shape === 'Square'}
                        on:click={() => webcamOverlayConfig.update(c => ({...c, shape: 'Square'}))}
                        disabled={$isRecording}
                      >
                        <span class="shape-icon">⬜</span>
                        <span>Square</span>
                      </button>
                      <button
                        class="shape-btn"
                        class:selected={$webcamOverlayConfig.shape === 'Circle'}
                        on:click={() => webcamOverlayConfig.update(c => ({...c, shape: 'Circle'}))}
                        disabled={$isRecording}
                      >
                        <span class="shape-icon">⭕</span>
                        <span>Circle</span>
                      </button>
                    </div>
                  </div>

                  <div class="setting-group">
                    <label class="setting-label">Size: {$webcamOverlayConfig.size}px</label>
                    <input
                      type="range"
                      min="200"
                      max="600"
                      bind:value={$webcamOverlayConfig.size}
                      disabled={$isRecording}
                      class="size-slider"
                    />
                  </div>
                </div>
              </div>
            {/if}

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

            <!-- Microphone Device Selector (only show when Microphone or Both is selected) -->
            {#if ($recordingConfig.audio_input === 'Microphone' || $recordingConfig.audio_input === 'Both') && $availableAudioDevices.length > 0}
              <div class="section">
                <label for="microphone-select">Microphone Device</label>
                <select
                  id="microphone-select"
                  value={$recordingConfig.audio_device_id || $availableAudioDevices[0]?.id}
                  on:change={(e) => updateConfig('audio_device_id', e.currentTarget.value)}
                  disabled={$isRecording}
                >
                  {#each $availableAudioDevices as device}
                    <option value={device.id}>{device.name}</option>
                  {/each}
                </select>
              </div>
            {/if}

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
                    class:active={$recordingConfig.fps === fps}
                    on:click={() => updateConfig('fps', fps)}
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
                  checked={$recordingConfig.show_cursor !== false}
                  on:change={(e) => updateConfig('show_cursor', e.target.checked)}
                  disabled={$isRecording}
                />
                <span>Show cursor in recording</span>
              </label>
            </div>

            <!-- Crop Region -->
            <div class="section">
              <label class="crop-toggle">
                <input
                  type="checkbox"
                  bind:checked={enableCrop}
                  disabled={$isRecording}
                />
                <span>Record specific region</span>
              </label>

              {#if enableCrop}
                <div class="crop-controls">
                  <button
                    class="btn-crop-editor"
                    on:click={handleOpenCropEditor}
                    disabled={$isRecording || !$selectedSource}
                  >
                    {currentCropRegion ? 'Edit Crop Region' : 'Select Crop Region'}
                  </button>

                  {#if currentCropRegion}
                    <div class="crop-info-compact">
                      <span class="crop-info-text">
                        {currentCropRegion.width} × {currentCropRegion.height}px at ({currentCropRegion.x}, {currentCropRegion.y})
                      </span>
                    </div>
                  {/if}
                </div>
              {/if}
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
                  <span class="value">{$recordingConfig.fps || 30}</span>
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

<!-- Crop Editor Modal -->
{#if currentSource}
  <CropEditor
    source={currentSource}
    initialCropRegion={currentCropRegion}
    bind:show={showCropEditor}
    on:apply={handleCropApply}
    on:cancel={handleCropCancel}
  />
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

  /* Sources Grid */
  .sources-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 0.75rem;
    margin-top: 0.75rem;
  }

  .source-card {
    display: flex;
    flex-direction: column;
    background: #2d2d2d;
    border: 2px solid #3d3d3d;
    border-radius: 8px;
    padding: 0;
    cursor: pointer;
    transition: all 0.2s;
    overflow: hidden;
  }

  .source-card:hover:not(:disabled) {
    background: #3d3d3d;
    border-color: #667eea;
  }

  .source-card.selected {
    border-color: #667eea;
    background: #3d3d3d;
  }

  .source-card:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .source-preview {
    width: 100%;
    height: 100px;
    object-fit: cover;
    background: #1a1a1a;
  }

  .source-placeholder {
    width: 100%;
    height: 100px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #1a1a1a;
    font-size: 3rem;
  }

  .source-info {
    padding: 0.75rem;
  }

  .source-name {
    color: #fff;
    font-size: 0.85rem;
    font-weight: 500;
    margin-bottom: 0.25rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .source-resolution {
    color: #aaa;
    font-size: 0.75rem;
  }

  /* Crop Controls */
  .crop-toggle {
    display: flex;
    align-items: center;
    cursor: pointer;
  }

  .crop-toggle input[type="checkbox"] {
    margin-right: 0.5rem;
    width: 18px;
    height: 18px;
    cursor: pointer;
  }

  .crop-toggle span {
    color: #fff;
  }

  .crop-controls {
    margin-top: 1rem;
  }

  .btn-crop-editor {
    width: 100%;
    padding: 0.75rem 1.5rem;
    background: #667eea;
    color: #fff;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9rem;
    font-weight: 500;
    transition: all 0.2s;
  }

  .btn-crop-editor:hover:not(:disabled) {
    background: #5568d3;
  }

  .btn-crop-editor:disabled {
    background: #3d3d3d;
    cursor: not-allowed;
    opacity: 0.5;
  }

  .crop-info-compact {
    margin-top: 0.75rem;
    padding: 0.75rem;
    background: #2d2d2d;
    border-radius: 4px;
    text-align: center;
  }

  .crop-info-text {
    color: #aaa;
    font-size: 0.85rem;
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

  /* Recording Mode Selector */
  .mode-selector {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.75rem;
  }

  .mode-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    padding: 1rem;
    background: #2d2d2d;
    border: 2px solid #3d3d3d;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s;
    color: #fff;
  }

  .mode-btn:hover:not(:disabled) {
    background: #3d3d3d;
    border-color: #667eea;
  }

  .mode-btn.active {
    background: #667eea;
    border-color: #667eea;
    color: white;
  }

  .mode-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .mode-icon {
    font-size: 2rem;
  }

  .mode-label {
    font-size: 0.875rem;
    font-weight: 500;
  }

  /* Warning Message */
  .warning-message {
    padding: 1rem;
    background: rgba(237, 137, 54, 0.1);
    border: 1px solid #ed8936;
    border-radius: 6px;
    color: #ed8936;
    font-size: 0.9rem;
  }

  /* Placeholder Icon */
  .placeholder-icon {
    font-size: 2.5rem;
  }

  /* Webcam Overlay Settings */
  .overlay-settings {
    background: #2d2d2d;
    border-radius: 6px;
    padding: 1rem;
    margin-top: 0.75rem;
  }

  .setting-group {
    margin-bottom: 1rem;
  }

  .setting-group:last-child {
    margin-bottom: 0;
  }

  .setting-label {
    display: block;
    color: #aaa;
    font-size: 0.85rem;
    margin-bottom: 0.5rem;
  }

  .position-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.5rem;
  }

  .position-btn {
    padding: 1rem;
    background: #1a1a1a;
    color: #fff;
    border: 2px solid #3d3d3d;
    border-radius: 4px;
    cursor: pointer;
    font-size: 1.5rem;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .position-btn:hover:not(:disabled) {
    background: #3d3d3d;
    border-color: #667eea;
  }

  .position-btn.selected {
    background: #667eea;
    border-color: #667eea;
  }

  .position-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .position-icon {
    font-size: 2rem;
  }

  .shape-buttons {
    display: flex;
    gap: 0.5rem;
  }

  .shape-btn {
    flex: 1;
    padding: 0.75rem;
    background: #1a1a1a;
    color: #fff;
    border: 2px solid #3d3d3d;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    font-size: 0.9rem;
  }

  .shape-btn:hover:not(:disabled) {
    background: #3d3d3d;
    border-color: #667eea;
  }

  .shape-btn.selected {
    background: #667eea;
    border-color: #667eea;
  }

  .shape-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .shape-icon {
    font-size: 1.2rem;
  }

  .size-slider {
    width: 100%;
    height: 6px;
    -webkit-appearance: none;
    appearance: none;
    background: #3d3d3d;
    border-radius: 3px;
    outline: none;
    cursor: pointer;
  }

  .size-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 18px;
    height: 18px;
    background: #667eea;
    border-radius: 50%;
    cursor: pointer;
    transition: all 0.2s;
  }

  .size-slider::-webkit-slider-thumb:hover {
    background: #5568d3;
    transform: scale(1.1);
  }

  .size-slider::-moz-range-thumb {
    width: 18px;
    height: 18px;
    background: #667eea;
    border-radius: 50%;
    border: none;
    cursor: pointer;
    transition: all 0.2s;
  }

  .size-slider::-moz-range-thumb:hover {
    background: #5568d3;
    transform: scale(1.1);
  }

  .size-slider:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .size-slider:disabled::-webkit-slider-thumb {
    cursor: not-allowed;
  }

  .size-slider:disabled::-moz-range-thumb {
    cursor: not-allowed;
  }
</style>
