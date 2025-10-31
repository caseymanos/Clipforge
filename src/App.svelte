<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { save, open } from "@tauri-apps/plugin-dialog";
  import Timeline from "./lib/components/Timeline.svelte";
  import VideoPreview from "./lib/components/VideoPreview.svelte";
  import MediaLibrary from "./lib/components/MediaLibrary.svelte";
  import ExportDialog from "./lib/components/ExportDialog.svelte";
  import RecordingPanel from "./lib/components/RecordingPanel.svelte";
  import RecordingStatusPopup from "./lib/components/RecordingStatusPopup.svelte";
  import SubtitleEditor from "./lib/components/SubtitleEditor.svelte";
  import HelpDialog from "./lib/components/HelpDialog.svelte";
  import { initializeTimeline, saveTimelineProject, loadTimelineProject } from "./lib/stores/timelineStore";
  import { isRecording } from "./lib/stores/recordingStore";

  let appVersion = "";
  let statusMessage = "";
  let exportDialog: ExportDialog;
  let recordingPanel: RecordingPanel;
  let helpDialog: HelpDialog;
  let showRecordingPopup = false;
  let hidePopupTimeout: number | null = null;

  // FPS tracking for main app
  let appFps = 0;
  let appFrameCount = 0;
  let appLastFpsUpdate = performance.now();

  function trackAppFps() {
    appFrameCount++;
    const now = performance.now();
    if (now - appLastFpsUpdate >= 1000) {
      appFps = Math.round(appFrameCount / ((now - appLastFpsUpdate) / 1000));
      appFrameCount = 0;
      appLastFpsUpdate = now;
    }
    requestAnimationFrame(trackAppFps);
  }

  function handleExport() {
    if (exportDialog) {
      exportDialog.open();
    }
  }

  function handleRecord() {
    // Don't open panel if already recording
    if (!$isRecording && recordingPanel) {
      recordingPanel.open();
    }
  }

  function handleHelp() {
    if (helpDialog) {
      helpDialog.open();
    }
  }

  function handlePopupMouseEnter() {
    // Clear any pending hide timeout when mouse enters popup area
    if (hidePopupTimeout) {
      clearTimeout(hidePopupTimeout);
      hidePopupTimeout = null;
    }
    showRecordingPopup = true;
  }

  function handlePopupMouseLeave() {
    // Delay hiding to allow smooth transition
    hidePopupTimeout = setTimeout(() => {
      showRecordingPopup = false;
      hidePopupTimeout = null;
    }, 200) as unknown as number;
  }

  onMount(async () => {
    appVersion = await invoke("get_app_version");

    // Initialize timeline from backend
    try {
      await initializeTimeline();
      console.log("Timeline initialized from backend");
    } catch (error) {
      console.error("Failed to initialize timeline:", error);
    }

    // Start FPS tracking
    trackAppFps();
  });

  async function handleSaveProject() {
    try {
      const filePath = await save({
        defaultPath: 'untitled-project.cfp',
        filters: [{
          name: 'ClipForge Project',
          extensions: ['cfp']
        }]
      });

      if (filePath) {
        await saveTimelineProject(filePath);
        statusMessage = `Project saved to ${filePath}`;
        setTimeout(() => statusMessage = "", 3000);
      }
    } catch (error) {
      console.error("Failed to save project:", error);
      statusMessage = "Failed to save project";
      setTimeout(() => statusMessage = "", 3000);
    }
  }

  async function handleLoadProject() {
    try {
      const filePath = await open({
        multiple: false,
        filters: [{
          name: 'ClipForge Project',
          extensions: ['cfp']
        }]
      });

      if (filePath && typeof filePath === 'string') {
        await loadTimelineProject(filePath);
        statusMessage = `Project loaded from ${filePath}`;
        setTimeout(() => statusMessage = "", 3000);
      }
    } catch (error) {
      console.error("Failed to load project:", error);
      statusMessage = "Failed to load project";
      setTimeout(() => statusMessage = "", 3000);
    }
  }
</script>

<main class="container wide">
  <!-- App FPS Overlay -->
  <div class="app-fps-overlay">
    {appFps} FPS
  </div>

  <header>
    <div class="header-content">
      <div class="title-section">
        <h1>ClipForge</h1>
        <div class="subtitle-group">
          <p class="subtitle">Desktop Video Editor</p>
          <p class="version">MVP</p>
        </div>
      </div>
      <div class="header-actions">
        <button class="btn-save" on:click={handleSaveProject}>Save Project</button>
        <button class="btn-load" on:click={handleLoadProject}>Load Project</button>
        <button class="btn-help" on:click={handleHelp}>
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"></circle>
            <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path>
            <line x1="12" y1="17" x2="12.01" y2="17"></line>
          </svg>
          Help
        </button>
      </div>
    </div>
  </header>

  <section class="media-library-section">
    <MediaLibrary
      on:recordClick={handleRecord}
      isRecording={$isRecording}
      showRecordingPopup={showRecordingPopup}
      on:popupMouseEnter={handlePopupMouseEnter}
      on:popupMouseLeave={handlePopupMouseLeave}
    />
  </section>

  <section class="preview-section">
    <!-- OPTION 3: Badge-Style Header with Indicators -->
    <div class="preview-header-with-indicators">
      <div class="preview-header-option3">
        <div class="preview-title-badge">
          <span class="live-badge">Video Preview</span>
        </div>
      </div>
      <div class="preview-indicators-header" id="preview-indicators-target">
        <!-- Indicators will be rendered here from VideoPreview component -->
      </div>
    </div>

    <VideoPreview />
  </section>

  <section class="timeline-section">
    <div class="timeline-header-with-controls">
      <div class="timeline-title-badge">
        <span class="timeline-badge">Timeline Editor</span>
      </div>
      <div class="project-controls">
        <button class="btn-export" on:click={handleExport}>Export Video</button>
      </div>
    </div>
    {#if statusMessage}
      <div class="status-message">{statusMessage}</div>
    {/if}

    <Timeline width={1200} height={400} />
  </section>

  <section class="subtitle-section">
    <h2>Subtitles & Transcription</h2>
    <p class="description">
      AI-powered subtitle generation using OpenAI Whisper. Transcribe your timeline audio, edit subtitles, and export to SRT format.
    </p>
    <SubtitleEditor />
  </section>


  <footer>
    <p>Built with Tauri + Svelte + Konva.js + Rust</p>
    <p class="architecture">
      Casey Manos, GauntletAI Fall 2025)
    </p>
  </footer>
</main>

<ExportDialog bind:this={exportDialog} />
<RecordingPanel bind:this={recordingPanel} />
<HelpDialog bind:this={helpDialog} />

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen,
      Ubuntu, Cantarell, sans-serif;
    background: #0f0f0f;
    color: #fff;
  }

  .container {
    max-width: 1400px;
    margin: 0 auto;
    padding: 2rem;
  }

  /* Wide layout - use full screen width */
  .container.wide {
    max-width: 100%;
    padding: 2rem 3rem;
  }

  header {
    margin-bottom: 1.5rem;
    padding: 0;
  }

  .header-content {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .title-section {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  h1 {
    font-size: 3.5rem;
    margin: 0;
    line-height: 1.2;
    padding-bottom: 0.1rem;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .subtitle-group {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .subtitle {
    font-size: 1rem;
    color: #aaa;
    margin: 0;
  }

  .version {
    color: #666;
    font-size: 0.9rem;
    margin: 0;
  }

  .media-library-section {
    margin-bottom: 3rem;
  }

  .preview-section {
    margin-bottom: 3rem;
  }

  .subtitle-section {
    margin-bottom: 3rem;
  }

  .timeline-section {
    margin-bottom: 3rem;
  }

  h2 {
    font-size: 1.8rem;
    margin-bottom: 0.5rem;
  }

  /* OPTION 1: Inline Compact Header */
  .preview-header-option1 {
    display: flex;
    align-items: baseline;
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .preview-title-compact {
    font-size: 1.25rem;
    margin: 0;
    font-weight: 600;
  }

  .preview-meta {
    color: #666;
    font-size: 0.85rem;
    font-weight: 400;
  }

  /* OPTION 2: Minimal Header with Icon Toggle */
  .preview-header-option2 {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1rem;
    padding: 0.5rem 0;
  }

  .preview-title-with-icon {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .preview-title-with-icon svg {
    color: #667eea;
    flex-shrink: 0;
  }

  .preview-title-minimal {
    font-size: 1.1rem;
    font-weight: 600;
    color: #fff;
  }

  .preview-status {
    color: #48bb78;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  /* OPTION 3: Badge-Style Header */
  .preview-header-with-indicators {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1rem;
  }

  .preview-header-option3 {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .preview-title-badge {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .preview-title-badge .title-text {
    font-size: 1.25rem;
    font-weight: 600;
    color: #fff;
  }

  .live-badge {
    display: inline-block;
    padding: 0.375rem 0.75rem;
    background: rgba(72, 187, 120, 0.15);
    color: #48bb78;
    font-size: 0.75rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    border-radius: 0.375rem;
    border: 1px solid rgba(72, 187, 120, 0.3);
  }

  .preview-subtitle-micro {
    color: #666;
    font-size: 0.8rem;
    margin-left: 0.125rem;
  }

  .preview-indicators-header {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  /* Preview indicators styles (global since they're rendered via innerHTML) */
  :global(.preview-indicators-header .indicator) {
    padding: 0.375rem 0.75rem;
    background: rgba(0, 0, 0, 0.7);
    color: #fff;
    font-size: 0.75rem;
    font-family: monospace;
    border-radius: 0.375rem;
    backdrop-filter: blur(10px);
    white-space: nowrap;
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  :global(.preview-indicators-header .mode-indicator) {
    background: rgba(102, 126, 234, 0.8);
    font-weight: 600;
  }

  :global(.preview-indicators-header .mode-indicator.composite) {
    background: rgba(234, 102, 126, 0.8);
  }

  :global(.preview-indicators-header .codec-indicator) {
    background: rgba(52, 211, 153, 0.8);
    font-weight: 600;
    text-transform: uppercase;
  }

  :global(.preview-indicators-header .file-indicator) {
    font-size: 0.7rem;
    background: rgba(0, 0, 0, 0.6);
  }

  h3 {
    font-size: 1.5rem;
    margin-bottom: 1rem;
  }

  h4 {
    font-size: 1.1rem;
    margin-bottom: 0.5rem;
  }

  .description {
    color: #aaa;
    margin-bottom: 1.5rem;
    line-height: 1.6;
  }

  .features {
    margin-bottom: 3rem;
  }

  .feature-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 1.5rem;
  }

  .feature {
    background: #1a1a1a;
    padding: 1.5rem;
    border-radius: 0.5rem;
    border: 1px solid #2d2d2d;
  }

  .feature h4 {
    color: #667eea;
    margin-bottom: 0.5rem;
  }

  .feature p {
    color: #aaa;
    font-size: 0.9rem;
    line-height: 1.5;
  }


  footer {
    text-align: center;
    padding-top: 2rem;
    border-top: 1px solid #2d2d2d;
    color: #666;
  }

  footer p {
    margin: 0.5rem 0;
  }

  .architecture {
    font-size: 0.85rem;
    color: #555;
  }

  /* Timeline header with project controls */
  .timeline-header-with-controls {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1rem;
  }

  .timeline-title-badge {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .timeline-badge {
    display: inline-block;
    padding: 0.375rem 0.75rem;
    background: rgba(72, 187, 120, 0.15);
    color: #48bb78;
    font-size: 0.75rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    border-radius: 0.375rem;
    border: 1px solid rgba(72, 187, 120, 0.3);
  }

  .project-controls {
    display: flex;
    gap: 0.75rem;
  }

  .project-controls button {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 0.375rem;
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .btn-save {
    background: #667eea;
    color: white;
  }

  .btn-save:hover {
    background: #5568d3;
  }

  .btn-load {
    background: #48bb78;
    color: white;
  }

  .btn-load:hover {
    background: #38a169;
  }

  .btn-export {
    background: #ed8936;
    color: white;
  }

  .btn-export:hover {
    background: #dd6b20;
  }

  .status-message {
    background: #1a1a1a;
    border: 1px solid #667eea;
    border-radius: 0.375rem;
    padding: 0.75rem 1rem;
    margin-bottom: 1rem;
    color: #aaa;
    font-size: 0.9rem;
  }

  /* Header actions */
  .header-actions {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .btn-help {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1.25rem;
    background: #2d2d2d;
    color: #fff;
    border: 1px solid #3d3d3d;
    border-radius: 0.375rem;
    font-size: 0.95rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .btn-help:hover {
    background: #3d3d3d;
    border-color: #667eea;
    transform: translateY(-1px);
  }

  .btn-help svg {
    flex-shrink: 0;
  }

  /* Media Library section header with record button */
  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 1rem;
  }

  .record-button-container {
    position: relative;
  }

  .btn-record-screen {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1.5rem;
    background: #e53e3e;
    color: white;
    border: none;
    border-radius: 0.375rem;
    font-size: 0.95rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .btn-record-screen:hover {
    background: #c53030;
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(229, 62, 62, 0.3);
  }

  .btn-record-screen.recording {
    animation: flash-recording 1.5s ease-in-out infinite;
  }

  .btn-record-screen.recording .record-icon {
    animation: pulse-fast 0.75s ease-in-out infinite;
  }

  .btn-record-screen .record-icon {
    font-size: 1.2rem;
  }

  @keyframes flash-recording {
    0%, 100% {
      background: #e53e3e;
      box-shadow: 0 4px 12px rgba(229, 62, 62, 0.3);
    }
    50% {
      background: #c53030;
      box-shadow: 0 4px 20px rgba(229, 62, 62, 0.6);
    }
  }

  @keyframes pulse-fast {
    0%, 100% {
      opacity: 1;
      transform: scale(1);
    }
    50% {
      opacity: 0.6;
      transform: scale(0.85);
    }
  }
</style>
