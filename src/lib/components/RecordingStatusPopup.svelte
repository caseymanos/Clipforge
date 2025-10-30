<script lang="ts">
  import { recordingState, stopRecording, recordingError } from "../stores/recordingStore";
  import { importMediaFile } from "../stores/mediaLibraryStore";

  export let show = false;

  $: duration = $recordingState.duration;
  $: state = $recordingState.state;
  $: isRecording = state === 'Recording';
  $: shouldShow = show && isRecording;

  function formatDuration(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  }

  async function handleStop() {
    const filePath = await stopRecording();
    if (filePath) {
      // Auto-import the recorded file to media library
      const mediaFile = await importMediaFile(filePath);
      if (mediaFile) {
        console.log('Recording saved and imported to media library!');
      }
    } else if ($recordingError) {
      console.error('Failed to stop recording:', $recordingError);
    }
  }
</script>

{#if shouldShow}
  <div
    class="recording-popup"
    on:mouseenter
    on:mouseleave
  >
    <div class="popup-header">
      <div class="recording-indicator">
        <span class="pulse-dot"></span>
        <span class="recording-text">Recording</span>
      </div>
    </div>

    <div class="popup-content">
      <div class="duration-display">
        {formatDuration(duration)}
      </div>

      <button class="btn-stop" on:click={handleStop}>
        <span class="stop-icon">■</span>
        Stop Recording
      </button>
    </div>
  </div>
{/if}

<style>
  .recording-popup {
    position: absolute;
    top: calc(100% + 8px);
    right: 0;
    width: 250px;
    background: #1a1a1a;
    border: 1px solid #e53e3e;
    border-radius: 8px;
    box-shadow: 0 8px 24px rgba(229, 62, 62, 0.3);
    z-index: 1000;
    animation: slideDown 0.2s ease-out;
  }

  @keyframes slideDown {
    from {
      opacity: 0;
      transform: translateY(-8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .popup-header {
    padding: 12px 16px;
    border-bottom: 1px solid #2d2d2d;
  }

  .recording-indicator {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .pulse-dot {
    width: 10px;
    height: 10px;
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
      transform: scale(0.9);
    }
  }

  .recording-text {
    font-size: 0.9rem;
    font-weight: 600;
    color: #e53e3e;
  }

  .popup-content {
    padding: 16px;
  }

  .duration-display {
    font-size: 2rem;
    font-weight: bold;
    color: #fff;
    text-align: center;
    margin-bottom: 16px;
    font-family: 'Courier New', monospace;
    letter-spacing: 2px;
  }

  .btn-stop {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 12px 16px;
    background: #e53e3e;
    color: white;
    border: none;
    border-radius: 6px;
    font-size: 0.95rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .btn-stop:hover {
    background: #c53030;
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(229, 62, 62, 0.4);
  }

  .btn-stop:active {
    transform: translateY(0);
  }

  .stop-icon {
    font-size: 1rem;
  }
</style>
