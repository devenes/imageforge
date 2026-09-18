<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import Thumbnail from './Thumbnail.svelte';
  import { appState } from '$lib/stores/appState.svelte';
  import type { QueueItem } from '$lib/types';
  import { formatBytes, formatDimensions, formatPercent } from '$lib/utils/formatters';

  let { item }: { item: QueueItem } = $props();

  async function handleReveal() {
    const target = item.result?.outputPath || item.info.path;
    try {
      await invoke('reveal_file', { path: target });
    } catch (e) {
      console.error('Failed to reveal file:', e);
    }
  }

  function handleRemove() {
    appState.removeItem(item.info.id);
  }
</script>

<div class="file-row" class:completed={item.status === 'COMPLETED'} class:failed={item.status === 'FAILED'}>
  <Thumbnail path={item.info.path} format={item.info.format} alt={item.info.filename} />

  <div class="info-col">
    <div class="name-row">
      <span class="filename" title={item.info.filename}>{item.info.filename}</span>
      {#if item.info.colorProfile}
        <span class="badge-tag" title="Color Profile: {item.info.colorProfile}">P3/ICC</span>
      {/if}
      {#if item.info.hasAlpha}
        <span class="badge-tag" title="Contains Alpha Channel">α</span>
      {/if}
    </div>

    <div class="meta-row">
      <span class="format-label">{item.info.format.toUpperCase()}</span>
      <span class="separator">·</span>
      <span class="dimensions">{formatDimensions(item.info.width, item.info.height)}</span>
      <span class="separator">·</span>
      <span class="size-orig">{formatBytes(item.info.bytes)}</span>

      {#if item.status === 'COMPLETED' && item.result}
        <span class="arrow">→</span>
        <span class="size-result">{formatBytes(item.result.outputBytes)}</span>
        <span class="savings-tag">-{formatPercent(item.result.savedPercent)}</span>
        <span class="dim-check" title="Dimensions preserved">✓ {formatDimensions(item.result.outputWidth, item.result.outputHeight)}</span>
      {:else if item.status === 'NO_SAVINGS'}
        <span class="no-savings-tag">Already optimized (original kept)</span>
      {:else if item.status === 'FAILED' && item.result?.error}
        <span class="error-tag" title={item.result.error}>{item.result.error}</span>
      {/if}
    </div>
  </div>

  <div class="status-col">
    {#if item.status === 'QUEUED'}
      <span class="status-badge queued">Ready</span>
      {#if !appState.isProcessing}
        <button class="icon-btn remove-btn" onclick={handleRemove} title="Remove from queue" aria-label="Remove">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      {/if}
    {:else if item.status === 'PROCESSING'}
      <span class="status-badge processing">
        <span class="spinner"></span>
        Processing
      </span>
    {:else if item.status === 'COMPLETED'}
      <button class="reveal-btn" onclick={handleReveal} title="Reveal in Finder">
        Show
      </button>
    {:else if item.status === 'NO_SAVINGS'}
      <span class="status-badge no-savings">No Savings</span>
    {:else if item.status === 'FAILED'}
      <span class="status-badge failed">Failed</span>
    {:else if item.status === 'SKIPPED'}
      <span class="status-badge skipped">Skipped</span>
    {/if}
  </div>
</div>

<style>
  .file-row {
    display: flex;
    align-items: center;
    gap: 0.85rem;
    padding: 0.65rem 0.85rem;
    background: var(--surface-bg);
    border: 1px solid var(--border-color-subtle);
    border-radius: var(--radius-md);
    transition: background 0.15s ease, border-color 0.15s ease;
  }

  .file-row:hover {
    background: var(--surface-bg-hover);
    border-color: var(--border-color);
  }

  .info-col {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    flex: 1;
    min-width: 0;
  }

  .name-row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .filename {
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 320px;
  }

  .badge-tag {
    font-size: 0.65rem;
    font-weight: 600;
    padding: 0.05rem 0.3rem;
    border-radius: var(--radius-sm);
    background: var(--surface-bg-subtle);
    color: var(--text-tertiary);
  }

  .meta-row {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.75rem;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
  }

  .separator {
    color: var(--text-tertiary);
  }

  .arrow {
    color: var(--text-tertiary);
    font-size: 0.7rem;
  }

  .size-result {
    font-weight: 600;
    color: var(--success-color);
  }

  .savings-tag {
    font-size: 0.7rem;
    font-weight: 600;
    padding: 0.05rem 0.35rem;
    border-radius: var(--radius-sm);
    background: rgba(16, 185, 129, 0.12);
    color: var(--success-color);
  }

  .dim-check {
    font-size: 0.7rem;
    color: var(--text-tertiary);
  }

  .no-savings-tag {
    font-size: 0.7rem;
    color: var(--warning-color);
    font-weight: 500;
  }

  .error-tag {
    font-size: 0.7rem;
    color: var(--error-color);
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .status-col {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-shrink: 0;
  }

  .status-badge {
    font-size: 0.7rem;
    font-weight: 500;
    padding: 0.15rem 0.5rem;
    border-radius: 9999px;
  }

  .status-badge.queued {
    background: var(--surface-bg-subtle);
    color: var(--text-tertiary);
  }

  .status-badge.processing {
    background: var(--accent-light);
    color: var(--accent-color);
    display: flex;
    align-items: center;
    gap: 0.3rem;
  }

  .status-badge.no-savings {
    background: rgba(245, 158, 11, 0.12);
    color: var(--warning-color);
  }

  .status-badge.failed {
    background: rgba(239, 68, 68, 0.12);
    color: var(--error-color);
  }

  .status-badge.skipped {
    background: var(--surface-bg-subtle);
    color: var(--text-tertiary);
  }

  .reveal-btn {
    font-size: 0.7rem;
    font-weight: 500;
    padding: 0.2rem 0.55rem;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-color);
    background: var(--surface-bg);
    color: var(--text-primary);
    cursor: pointer;
  }

  .reveal-btn:hover {
    background: var(--surface-bg-hover);
  }

  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border-radius: var(--radius-sm);
    border: none;
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
  }

  .icon-btn:hover {
    color: var(--error-color);
    background: rgba(239, 68, 68, 0.1);
  }

  .spinner {
    width: 10px;
    height: 10px;
    border: 2px solid currentColor;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
