<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let show = false;
  export let x = 0;
  export let y = 0;
  export let options: Array<{ label: string; action: string; disabled?: boolean }> = [];

  const dispatch = createEventDispatcher();

  function handleOptionClick(action: string) {
    dispatch('select', { action });
    show = false;
  }

  function handleClickOutside(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (show && !target.closest('.context-menu')) {
      show = false;
    }
  }
</script>

<svelte:window on:click={handleClickOutside} />

{#if show}
  <div
    class="context-menu"
    style="left: {x}px; top: {y}px;"
    role="menu"
    aria-label="Context menu"
  >
    {#each options as option}
      <button
        class="context-menu-item"
        class:disabled={option.disabled}
        disabled={option.disabled}
        on:click={() => handleOptionClick(option.action)}
        role="menuitem"
      >
        {option.label}
      </button>
    {/each}
  </div>
{/if}

<style>
  .context-menu {
    position: fixed;
    background: #2d2d2d;
    border: 1px solid #3d3d3d;
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
    padding: 4px;
    min-width: 150px;
    z-index: 10000;
  }

  .context-menu-item {
    display: block;
    width: 100%;
    padding: 8px 12px;
    background: transparent;
    color: #ffffff;
    border: none;
    border-radius: 4px;
    text-align: left;
    cursor: pointer;
    font-size: 0.875rem;
    transition: background 0.15s;
  }

  .context-menu-item:hover:not(.disabled) {
    background: #3d3d3d;
  }

  .context-menu-item:active:not(.disabled) {
    background: #4d4d4d;
  }

  .context-menu-item.disabled {
    color: #777;
    cursor: not-allowed;
    opacity: 0.5;
  }
</style>
