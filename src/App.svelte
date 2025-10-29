<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { save, open } from "@tauri-apps/plugin-dialog";
  import Timeline from "./lib/components/Timeline.svelte";
  import VideoPreview from "./lib/components/VideoPreview.svelte";
  import MediaLibrary from "./lib/components/MediaLibrary.svelte";
  import { initializeTimeline, saveTimelineProject, loadTimelineProject } from "./lib/stores/timelineStore";

  let appVersion = "";
  let statusMessage = "";

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
    <h2>Media Library</h2>
    <p class="description">
      Import and manage your video files. Click "Import Media" to add videos, then double-click to add them to the timeline.
    </p>
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
      </div>
    </div>
    {#if statusMessage}
      <div class="status-message">{statusMessage}</div>
    {/if}

    <Timeline width={1200} height={400} />
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
    </ul>
  </section>

  <footer>
    <p>Casey Manos | 2025 | GauntletAI</p>
    <p class="architecture">
      Built with Tauri + Svelte + Konva.js + Rust
    </p>
  </footer>
</main>

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

  .status-message {
    background: #1a1a1a;
    border: 1px solid #667eea;
    border-radius: 0.375rem;
    padding: 0.75rem 1rem;
    margin-bottom: 1rem;
    color: #aaa;
    font-size: 0.9rem;
  }
</style>
