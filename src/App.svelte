<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { save, open } from "@tauri-apps/plugin-dialog";
  import Timeline from "./lib/components/Timeline.svelte";
  import VideoPreview from "./lib/components/VideoPreview.svelte";
  import MediaLibrary from "./lib/components/MediaLibrary.svelte";
  import ExportDialog from "./lib/components/ExportDialog.svelte";
  import RecordingPanel from "./lib/components/RecordingPanel.svelte";
  import { initializeTimeline, saveTimelineProject, loadTimelineProject } from "./lib/stores/timelineStore";

  let appVersion = "";
  let statusMessage = "";
  let exportDialog: ExportDialog;
  let recordingPanel: RecordingPanel;

  function handleExport() {
    if (exportDialog) {
      exportDialog.open();
    }
  }

  function handleRecord() {
    if (recordingPanel) {
      recordingPanel.open();
    }
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

<main class="container">
  <header>
    <h1>ClipForge</h1>
    <p class="subtitle">Desktop Video Editor - Complete Application</p>
    <p class="version">Version {appVersion}</p>
  </header>

  <section class="media-library-section">
    <div class="section-header">
      <div>
        <h2>Media Library</h2>
        <p class="description">
          Import and manage your video files. Click "Import Media" to add videos, or "Record Screen" to capture your screen.
        </p>
      </div>
      <button class="btn-record-screen" on:click={handleRecord}>
        <span class="record-icon">●</span>
        Record Screen
      </button>
    </div>
    <MediaLibrary />
  </section>

  <section class="preview-section">
    <h2>Video Preview</h2>
    <p class="description">
      Real-time preview window with frame-by-frame rendering. Scrub the timeline to see the preview update!
    </p>
    <VideoPreview />
  </section>

  <section class="timeline-section">
    <div class="timeline-header">
      <div>
        <h2>Timeline Editor</h2>
        <p class="description">
          Canvas-based timeline with Konva.js. Try dragging clips, resizing them, zooming with mouse wheel,
          and dragging the playhead!
        </p>
      </div>
      <div class="project-controls">
        <button class="btn-save" on:click={handleSaveProject}>Save Project</button>
        <button class="btn-load" on:click={handleLoadProject}>Load Project</button>
        <button class="btn-export" on:click={handleExport}>Export Video</button>
      </div>
    </div>
    {#if statusMessage}
      <div class="status-message">{statusMessage}</div>
    {/if}

    <Timeline width={1200} height={400} />
  </section>

  <section class="features">
    <h3>Features Implemented</h3>
    <div class="feature-grid">
      <div class="feature">
        <h4>✅ Media Library</h4>
        <p>Import, manage, and organize video files with thumbnails</p>
      </div>
      <div class="feature">
        <h4>✅ File Import</h4>
        <p>Multi-file import with deduplication and metadata extraction</p>
      </div>
      <div class="feature">
        <h4>✅ Canvas Rendering</h4>
        <p>Konva.js-based rendering for 60 FPS with 200+ clips</p>
      </div>
      <div class="feature">
        <h4>✅ Drag & Drop</h4>
        <p>Drag clips to reposition within or between tracks</p>
      </div>
      <div class="feature">
        <h4>✅ Clip Trimming</h4>
        <p>Resize handles on selected clips for trim adjustments</p>
      </div>
      <div class="feature">
        <h4>✅ Zoom & Scroll</h4>
        <p>Mouse wheel zoom, Shift+wheel scroll</p>
      </div>
      <div class="feature">
        <h4>✅ Playhead Control</h4>
        <p>Draggable playhead for time scrubbing</p>
      </div>
      <div class="feature">
        <h4>✅ Multi-track Support</h4>
        <p>Video and Audio tracks with visual separation</p>
      </div>
      <div class="feature">
        <h4>✅ State Management</h4>
        <p>Svelte stores for reactive timeline state</p>
      </div>
      <div class="feature">
        <h4>✅ Selection System</h4>
        <p>Click to select clips, visual feedback</p>
      </div>
      <div class="feature">
        <h4>✅ Search & Filter</h4>
        <p>Search files by name and sort by various criteria</p>
      </div>
      <div class="feature">
        <h4>✅ File Metadata</h4>
        <p>Display resolution, codec, duration, and file size</p>
      </div>
      <div class="feature">
        <h4>✅ Video Export</h4>
        <p>Export timeline to MP4 with presets (YouTube, Instagram, Twitter)</p>
      </div>
      <div class="feature">
        <h4>✅ Export Progress</h4>
        <p>Real-time progress tracking with percentage, FPS, and ETA</p>
      </div>
    </div>
  </section>

  <section class="instructions">
    <h3>How to Use</h3>
    <ul>
      <li><strong>Import videos:</strong> Click "Import Media" to add video files to your library</li>
      <li><strong>Browse library:</strong> Search and sort your imported files</li>
      <li><strong>Select files:</strong> Click on a file to select it, double-click to add to timeline (coming soon)</li>
      <li><strong>Remove files:</strong> Hover over a file and click the trash icon to remove it</li>
      <li><strong>Drag clips:</strong> Click and drag any clip on the timeline to move it</li>
      <li><strong>Trim clips:</strong> Select a clip, then drag the white handles on the edges</li>
      <li><strong>Zoom:</strong> Use mouse wheel to zoom in/out on the timeline</li>
      <li><strong>Scroll:</strong> Hold Shift + mouse wheel to scroll horizontally</li>
      <li><strong>Move playhead:</strong> Drag the red circle or click on the timeline</li>
      <li><strong>Export video:</strong> Click "Export Video" to render timeline to MP4 with real-time progress</li>
    </ul>
  </section>

  <footer>
    <p>Built with Tauri 2.0 + Svelte 4 + Konva.js + Rust</p>
    <p class="architecture">
      Modules: Application Shell (1) • File System (2) • Timeline Engine (5) • Timeline UI (7) • Video Preview (8) • Export (6)
    </p>
  </footer>
</main>

<ExportDialog bind:this={exportDialog} />
<RecordingPanel bind:this={recordingPanel} />

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

  header {
    text-align: center;
    margin-bottom: 3rem;
  }

  h1 {
    font-size: 3rem;
    margin-bottom: 0.5rem;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .subtitle {
    font-size: 1.2rem;
    color: #aaa;
    margin-bottom: 0.5rem;
  }

  .version {
    color: #666;
    font-size: 0.9rem;
  }

  .media-library-section {
    margin-bottom: 3rem;
  }

  .preview-section {
    margin-bottom: 3rem;
  }

  .timeline-section {
    margin-bottom: 3rem;
  }

  h2 {
    font-size: 1.8rem;
    margin-bottom: 0.5rem;
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

  .instructions {
    background: #1a1a1a;
    padding: 2rem;
    border-radius: 0.5rem;
    border: 1px solid #2d2d2d;
    margin-bottom: 3rem;
  }

  .instructions ul {
    list-style: none;
    padding: 0;
    margin-top: 1rem;
  }

  .instructions li {
    padding: 0.75rem 0;
    border-bottom: 1px solid #2d2d2d;
    color: #ccc;
  }

  .instructions li:last-child {
    border-bottom: none;
  }

  .instructions strong {
    color: #667eea;
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
  .timeline-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 1rem;
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

  /* Media Library section header with record button */
  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 1rem;
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

  .btn-record-screen .record-icon {
    font-size: 1.2rem;
    animation: pulse-subtle 2s ease-in-out infinite;
  }

  @keyframes pulse-subtle {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.7;
    }
  }
</style>
