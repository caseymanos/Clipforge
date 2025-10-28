<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import Timeline from "./lib/components/Timeline.svelte";
  import { timelineStore, addClipToTrack, type Clip } from "./lib/stores/timelineStore";

  let appVersion = "";

  onMount(async () => {
    appVersion = await invoke("get_app_version");

    // Add example clips for demonstration
    addExampleClips();
  });

  function addExampleClips() {
    // Add some demo clips to the timeline
    const demoClips: Clip[] = [
      {
        id: 'clip-1',
        media_file_id: 'media-1',
        track_position: 0,
        duration: 5.0,
        trim_start: 0,
        trim_end: 5.0,
        effects: [],
        volume: 1.0,
        speed: 1.0,
      },
      {
        id: 'clip-2',
        media_file_id: 'media-2',
        track_position: 6.0,
        duration: 4.0,
        trim_start: 0,
        trim_end: 4.0,
        effects: [],
        volume: 1.0,
        speed: 1.0,
      },
      {
        id: 'clip-3',
        media_file_id: 'media-3',
        track_position: 11.0,
        duration: 3.5,
        trim_start: 0,
        trim_end: 3.5,
        effects: [],
        volume: 1.0,
        speed: 1.0,
      },
      {
        id: 'clip-4',
        media_file_id: 'media-4',
        track_position: 1.0,
        duration: 6.0,
        trim_start: 0,
        trim_end: 6.0,
        effects: [],
        volume: 1.0,
        speed: 1.0,
      },
    ];

    // Add clips to video track
    addClipToTrack('video-track-1', demoClips[0]);
    addClipToTrack('video-track-1', demoClips[1]);
    addClipToTrack('video-track-1', demoClips[2]);

    // Add clip to audio track
    addClipToTrack('audio-track-1', demoClips[3]);
  }
</script>

<main class="container">
  <header>
    <h1>ClipForge</h1>
    <p class="subtitle">Desktop Video Editor - Module 7: Timeline UI</p>
    <p class="version">Version {appVersion}</p>
  </header>

  <section class="timeline-section">
    <h2>Timeline Editor</h2>
    <p class="description">
      Canvas-based timeline with Konva.js. Try dragging clips, resizing them, zooming with mouse wheel,
      and dragging the playhead!
    </p>

    <Timeline width={1200} height={400} />
  </section>

  <section class="features">
    <h3>Module 7 Features Implemented</h3>
    <div class="feature-grid">
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
    </div>
  </section>

  <section class="instructions">
    <h3>How to Use</h3>
    <ul>
      <li><strong>Drag clips:</strong> Click and drag any clip to move it</li>
      <li><strong>Trim clips:</strong> Select a clip, then drag the white handles on the edges</li>
      <li><strong>Zoom:</strong> Use mouse wheel to zoom in/out</li>
      <li><strong>Scroll:</strong> Hold Shift + mouse wheel to scroll horizontally</li>
      <li><strong>Move playhead:</strong> Drag the red circle or click on the timeline</li>
      <li><strong>Select clip:</strong> Click on any clip to select it</li>
    </ul>
  </section>

  <footer>
    <p>Built with Tauri 2.0 + Svelte 4 + Konva.js + Rust</p>
    <p class="architecture">
      Modules: Application Shell (1) • File System (2) • Timeline Engine (5) • Timeline UI (7)
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
</style>
