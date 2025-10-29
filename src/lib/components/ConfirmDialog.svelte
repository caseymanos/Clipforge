<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let message: string;
  export let x: number = 0;
  export let y: number = 0;
  export let show: boolean = false;

  const dispatch = createEventDispatcher();

  function handleConfirm() {
    dispatch('confirm', { confirmed: true });
    show = false;
  }

  function handleCancel() {
    dispatch('confirm', { confirmed: false });
    show = false;
  }
</script>

{#if show}
  <div class="dialog-backdrop" on:click={handleCancel}>
    <div
      class="dialog-box"
      style="left: {x}px; top: {y}px;"
      on:click|stopPropagation
    >
      <div class="dialog-message">
        {message}
      </div>
      <div class="dialog-buttons">
        <button class="btn btn-danger" on:click={handleConfirm}>
          Delete
        </button>
        <button class="btn btn-secondary" on:click={handleCancel}>
          Cancel
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .dialog-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: rgba(0, 0, 0, 0.5);
    z-index: 10000;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .dialog-box {
    position: absolute;
    background: #2d2d2d;
    border: 1px solid #444;
    border-radius: 8px;
    padding: 1.5rem;
    min-width: 300px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
    transform: translate(-50%, -50%);
  }

  .dialog-message {
    color: #fff;
    font-size: 0.95rem;
    margin-bottom: 1.25rem;
    line-height: 1.5;
  }

  .dialog-buttons {
    display: flex;
    gap: 0.75rem;
    justify-content: flex-end;
  }

  .btn {
    padding: 0.5rem 1.25rem;
    border: none;
    border-radius: 4px;
    font-size: 0.875rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-danger {
    background: #ef4444;
    color: #fff;
  }

  .btn-danger:hover {
    background: #dc2626;
  }

  .btn-danger:active {
    background: #b91c1c;
  }

  .btn-secondary {
    background: #555;
    color: #fff;
  }

  .btn-secondary:hover {
    background: #666;
  }

  .btn-secondary:active {
    background: #444;
  }
</style>
