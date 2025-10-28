<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  let appVersion = "";
  let greetMsg = "";

  onMount(async () => {
    appVersion = await invoke("get_app_version");
  });

  async function greet() {
    greetMsg = `Welcome to ClipForge!`;
  }
</script>

<main class="container">
  <h1>ClipForge</h1>

  <div class="row">
    <p>Desktop Video Editor</p>
  </div>

  <div class="row">
    <div>
      <button type="button" on:click={greet}>
        Get Started
      </button>
    </div>
  </div>

  {#if greetMsg}
    <p class="message">{greetMsg}</p>
  {/if}

  <footer>
    <p>Version {appVersion}</p>
    <p>Built with Tauri + Svelte + Rust</p>
  </footer>
</main>

<style>
  .container {
    margin: 0;
    padding-top: 10vh;
    display: flex;
    flex-direction: column;
    justify-content: center;
    text-align: center;
  }

  h1 {
    font-size: 3.2em;
    line-height: 1.1;
    font-weight: 700;
    margin-bottom: 0.5em;
    color: #667eea;
  }

  .row {
    display: flex;
    justify-content: center;
    margin: 1em 0;
  }

  button {
    border-radius: 8px;
    border: 1px solid transparent;
    padding: 0.6em 1.2em;
    font-size: 1em;
    font-weight: 500;
    font-family: inherit;
    background-color: #667eea;
    color: white;
    cursor: pointer;
    transition: border-color 0.25s;
  }

  button:hover {
    border-color: #646cff;
    background-color: #5568d3;
  }

  button:focus,
  button:focus-visible {
    outline: 4px auto -webkit-focus-ring-color;
  }

  .message {
    color: #667eea;
    font-size: 1.2em;
    margin-top: 1em;
  }

  footer {
    margin-top: 3em;
    color: #888;
    font-size: 0.9em;
  }

  footer p {
    margin: 0.25em 0;
  }
</style>
