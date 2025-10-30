import "./styles.css";
import App from "./App.svelte";

// Detect if running in Tauri desktop app context
const isTauriApp = typeof window !== 'undefined' && '__TAURI__' in window;

if (!isTauriApp) {
  // Show error message if running in browser instead of desktop app
  const appContainer = document.getElementById("app");
  if (appContainer) {
    appContainer.innerHTML = `
      <div style="
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        min-height: 100vh;
        background: #0f0f0f;
        color: #fff;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
        padding: 2rem;
        text-align: center;
      ">
        <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="#ef4444" style="margin-bottom: 1.5rem;">
          <circle cx="12" cy="12" r="10" stroke-width="2"/>
          <line x1="12" y1="8" x2="12" y2="12" stroke-width="2"/>
          <line x1="12" y1="16" x2="12.01" y2="16" stroke-width="2"/>
        </svg>
        <h1 style="font-size: 2rem; margin-bottom: 1rem; color: #ef4444;">
          Wrong Window
        </h1>
        <p style="font-size: 1.1rem; color: #aaa; max-width: 500px; margin-bottom: 1.5rem;">
          ClipForge must run in the <strong>desktop app window</strong>, not in a web browser.
        </p>
        <div style="background: #1a1a1a; padding: 1.5rem; border-radius: 0.5rem; border: 1px solid #2d2d2d; max-width: 600px;">
          <h2 style="font-size: 1.2rem; margin-bottom: 1rem; color: #667eea;">How to Run ClipForge:</h2>
          <ol style="text-align: left; color: #ccc; line-height: 1.8;">
            <li>Close this browser window</li>
            <li>Run <code style="background: #0f0f0f; padding: 0.25rem 0.5rem; border-radius: 0.25rem; color: #667eea;">npm run tauri:dev</code> in your terminal</li>
            <li>Wait for the <strong>desktop application window</strong> to open automatically</li>
            <li>Use the desktop window (not localhost:5173 in your browser)</li>
          </ol>
        </div>
        <p style="margin-top: 1.5rem; font-size: 0.9rem; color: #666;">
          The browser preview at localhost:5173 won't work because Tauri APIs are only available in the desktop app.
        </p>
      </div>
    `;
  }
} else {
  // Initialize app normally in Tauri context
  new App({
    target: document.getElementById("app")!,
  });
}
