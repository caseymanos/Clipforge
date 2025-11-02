<script lang="ts">
  export let show = false;

  export function open() {
    show = true;
  }

  export function close() {
    show = false;
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      close();
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      close();
    }
  }
</script>

{#if show}
  <div
    class="dialog-backdrop"
    on:click={handleBackdropClick}
    on:keydown={handleKeyDown}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <div class="dialog-content">
      <div class="dialog-header">
        <h2>How to Use ClipForge</h2>
        <button class="close-btn" on:click={close} aria-label="Close help dialog">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>

      <div class="dialog-body">
        <section class="help-section">
          <h3>Media Library</h3>
          <ul>
            <li><strong>Import videos:</strong> Click "Import Media" to add video files to your library</li>
            <li><strong>Browse library:</strong> Search and sort your imported files</li>
            <li><strong>Select files:</strong> Click on a file to select it, double-click to add to timeline</li>
            <li><strong>Remove files:</strong> Hover over a file and click the trash icon to remove it</li>
          </ul>
        </section>

        <section class="help-section">
          <h3>Timeline Editor</h3>
          <ul>
            <li><strong>Drag clips:</strong> Click and drag any clip on the timeline to move it</li>
            <li><strong>Trim clips:</strong> Select a clip, then drag the white handles on the edges</li>
            <li><strong>Zoom:</strong> Use mouse wheel to zoom in/out on the timeline</li>
            <li><strong>Scroll:</strong> Hold Shift + mouse wheel to scroll horizontally</li>
            <li><strong>Move playhead:</strong> Drag the red circle or click on the timeline</li>
            <li><strong>Split clips:</strong> Position playhead over a clip and press Cmd/Ctrl+B</li>
            <li><strong>Delete clips:</strong> Select a clip and press Delete or Backspace</li>
          </ul>
        </section>

        <section class="help-section">
          <h3>Video Preview</h3>
          <ul>
            <li><strong>Real-time preview:</strong> See the composite result of all timeline clips</li>
            <li><strong>Scrub timeline:</strong> Move the playhead to update the preview</li>
          </ul>
        </section>

        <section class="help-section">
          <h3>Recording</h3>
          <ul>
            <li><strong>Record screen:</strong> Click the "Record Screen" button in the header</li>
            <li><strong>Select source:</strong> Choose screen, window, or camera</li>
            <li><strong>Start/Stop:</strong> Use the recording panel controls</li>
            <li><strong>Auto-import:</strong> Recorded videos are automatically added to your media library</li>
          </ul>
        </section>

        <section class="help-section">
          <h3>Export & Projects</h3>
          <ul>
            <li><strong>Export video:</strong> Click "Export Video" to render timeline to MP4 with real-time progress</li>
            <li><strong>Save project:</strong> Click "Save Project" to save your timeline as a .cfp file</li>
            <li><strong>Load project:</strong> Click "Load Project" to open a saved .cfp project</li>
          </ul>
        </section>

        <section class="help-section">
          <h3>Keyboard Shortcuts</h3>
          <ul>
            <li><strong>Cmd/Ctrl + B:</strong> Split clip at playhead</li>
            <li><strong>Delete/Backspace:</strong> Delete selected clip</li>
            <li><strong>Mouse Wheel:</strong> Zoom timeline</li>
            <li><strong>Shift + Mouse Wheel:</strong> Scroll timeline horizontally</li>
          </ul>
        </section>
      </div>
    </div>
  </div>
{/if}

<style>
  .dialog-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.85);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 2rem;
  }

  .dialog-content {
    background: #1a1a1a;
    border: 1px solid #2d2d2d;
    border-radius: 0.75rem;
    max-width: 800px;
    width: 100%;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
  }

  .dialog-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem 2rem;
    border-bottom: 1px solid #2d2d2d;
  }

  .dialog-header h2 {
    margin: 0;
    font-size: 1.5rem;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .close-btn {
    background: none;
    border: none;
    color: #aaa;
    cursor: pointer;
    padding: 0.5rem;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: color 0.2s;
  }

  .close-btn:hover {
    color: #fff;
  }

  .dialog-body {
    padding: 2rem;
    overflow-y: auto;
  }

  .help-section {
    margin-bottom: 2rem;
  }

  .help-section:last-child {
    margin-bottom: 0;
  }

  .help-section h3 {
    font-size: 1.25rem;
    margin-bottom: 1rem;
    color: #667eea;
  }

  .help-section ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .help-section li {
    padding: 0.75rem 0;
    border-bottom: 1px solid #2d2d2d;
    color: #ccc;
    line-height: 1.6;
  }

  .help-section li:last-child {
    border-bottom: none;
  }

  .help-section strong {
    color: #667eea;
  }

  /* Scrollbar styling */
  .dialog-body::-webkit-scrollbar {
    width: 8px;
  }

  .dialog-body::-webkit-scrollbar-track {
    background: #0f0f0f;
  }

  .dialog-body::-webkit-scrollbar-thumb {
    background: #2d2d2d;
    border-radius: 4px;
  }

  .dialog-body::-webkit-scrollbar-thumb:hover {
    background: #3d3d3d;
  }
</style>
