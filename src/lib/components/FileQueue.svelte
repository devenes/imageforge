<script lang="ts">
  import FileRow from './FileRow.svelte';
  import { appState } from '$lib/stores/appState.svelte';
  import { formatBytes } from '$lib/utils/formatters';

  let { onAddMore }: { onAddMore: () => void } = $props();
</script>

<div class="queue-container">
  <div class="queue-header">
    <div class="header-left">
      <h2 class="queue-title">
        {appState.queue.length} {appState.queue.length === 1 ? 'image' : 'images'}
      </h2>
      <span class="header-size">({formatBytes(appState.totalOriginalBytes)})</span>
    </div>

    <div class="header-actions">
      {#if !appState.isProcessing}
        <button class="btn secondary-btn small-btn" onclick={onAddMore} aria-label="Add more images">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 5v14M5 12h14"/>
          </svg>
          Add More
        </button>

        <button class="btn ghost-btn small-btn" onclick={() => appState.clearQueue()} aria-label="Clear queue">
          Clear
          <kbd class="shortcut">⌘K</kbd>
        </button>
      {/if}
    </div>
  </div>

  <div class="queue-list" role="list">
    {#each appState.queue as item (item.info.id)}
      <FileRow {item} />
    {/each}
  </div>
</div>

<style>
  .queue-container {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    background: var(--surface-bg);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    overflow: hidden;
  }

  .queue-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--border-color-subtle);
    background: var(--surface-bg-subtle);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .queue-title {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .header-size {
    font-size: 0.8rem;
    color: var(--text-secondary);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .small-btn {
    padding: 0.25rem 0.6rem;
    font-size: 0.75rem;
  }

  .shortcut {
    font-size: 0.65rem;
    padding: 0.1rem 0.3rem;
    border-radius: var(--radius-sm);
    background: var(--surface-bg-subtle);
    border: 1px solid var(--border-color-subtle);
    color: var(--text-tertiary);
    font-family: var(--font-mono);
  }

  .queue-list {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    padding: 0.75rem;
    overflow-y: auto;
    flex: 1;
    min-height: 160px;
  }
</style>
