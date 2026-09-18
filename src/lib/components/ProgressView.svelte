<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { appState } from '$lib/stores/appState.svelte';

  async function handleCancel() {
    try {
      await invoke('cancel_batch');
      appState.cancelBatch();
    } catch (e) {
      console.error('Failed to cancel batch:', e);
    }
  }

  let percent = $derived.by(() => {
    if (!appState.progress || appState.progress.totalCount === 0) return 0;
    return Math.round((appState.progress.currentIndex / appState.progress.totalCount) * 100);
  });
</script>

{#if appState.isProcessing}
  <div class="progress-panel" role="status" aria-live="polite">
    <div class="progress-info">
      <div class="header-line">
        <span class="count-badge">
          {appState.progress?.currentIndex || 0} / {appState.progress?.totalCount || 0} files
        </span>
        <span class="stage-badge">{appState.progress?.stage || 'Processing'}</span>
      </div>

      <div class="file-name" title={appState.progress?.currentFilename || ''}>
        Processing: <strong>{appState.progress?.currentFilename || '...'}</strong>
      </div>

      <div class="bar-wrap">
        <div class="bar-fill" style="width: {percent}%"></div>
      </div>
    </div>

    <button class="btn secondary-btn cancel-btn" onclick={handleCancel}>
      Cancel
    </button>
  </div>
{/if}

<style>
  .progress-panel {
    display: flex;
    align-items: center;
    gap: 1.5rem;
    padding: 1rem 1.25rem;
    background: var(--surface-bg);
    border: 1px solid var(--accent-light);
    box-shadow: 0 4px 12px rgba(37, 99, 235, 0.08);
    border-radius: var(--radius-lg);
    animation: slideDown 0.2s ease-out;
  }

  .progress-info {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    flex: 1;
    min-width: 0;
  }

  .header-line {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .count-badge {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--accent-color);
  }

  .stage-badge {
    font-size: 0.7rem;
    font-weight: 500;
    padding: 0.1rem 0.4rem;
    border-radius: var(--radius-sm);
    background: var(--surface-bg-subtle);
    color: var(--text-secondary);
  }

  .file-name {
    font-size: 0.82rem;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .file-name strong {
    color: var(--text-primary);
  }

  .bar-wrap {
    height: 6px;
    background: var(--surface-bg-subtle);
    border-radius: 9999px;
    overflow: hidden;
  }

  .bar-fill {
    height: 100%;
    background: var(--accent-color);
    border-radius: 9999px;
    transition: width 0.2s ease-out;
  }

  .cancel-btn {
    padding: 0.4rem 0.9rem;
    font-size: 0.8rem;
    color: var(--error-color);
  }

  .cancel-btn:hover {
    background: rgba(239, 68, 68, 0.08);
    border-color: rgba(239, 68, 68, 0.3);
  }

  @keyframes slideDown {
    from { opacity: 0; transform: translateY(-8px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
