<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { appState } from '$lib/stores/appState.svelte';
  import { formatBytes, formatPercent } from '$lib/utils/formatters';

  let { onCompressMore }: { onCompressMore: () => void } = $props();

  async function handleOpenFolder() {
    // Find output folder from first completed item with output_path or customOutputDir
    const firstOutput = appState.results.find((r) => r.outputPath)?.outputPath;
    const targetDir = appState.settings.customOutputDir || firstOutput;
    if (targetDir) {
      try {
        await invoke('open_output_folder', { path: targetDir });
      } catch (e) {
        console.error('Failed to open output folder:', e);
      }
    }
  }
</script>

<div class="results-banner">
  <div class="summary-card">
    <div class="stats-group">
      <div class="stat-item">
        <span class="stat-label">Before</span>
        <span class="stat-val">{formatBytes(appState.totalOriginalBytes)}</span>
      </div>

      <div class="stat-divider">→</div>

      <div class="stat-item">
        <span class="stat-label">After</span>
        <span class="stat-val result-after">{formatBytes(appState.totalOutputBytes)}</span>
      </div>

      <div class="stat-divider">·</div>

      <div class="stat-item">
        <span class="stat-label">Saved</span>
        <span class="stat-val highlight">{formatBytes(appState.totalSavedBytes)}</span>
      </div>

      <div class="reduction-badge">
        {formatPercent(appState.overallReductionPercent)} smaller
      </div>
    </div>

    <div class="actions-group">
      <button class="btn secondary-btn" onclick={handleOpenFolder}>
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
        </svg>
        Open Folder
      </button>

      <button class="btn primary-btn" onclick={onCompressMore}>
        Compress More
      </button>
    </div>
  </div>
</div>

<style>
  .results-banner {
    padding: 0.25rem 0;
  }

  .summary-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1.5rem;
    padding: 1rem 1.25rem;
    background: var(--surface-bg);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
  }

  .stats-group {
    display: flex;
    align-items: center;
    gap: 1.25rem;
  }

  .stat-item {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .stat-label {
    font-size: 0.72rem;
    font-weight: 600;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .stat-val {
    font-size: 1.05rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .result-after {
    color: var(--success-color);
  }

  .highlight {
    color: var(--success-color);
  }

  .stat-divider {
    color: var(--text-tertiary);
    font-size: 0.9rem;
  }

  .reduction-badge {
    font-size: 0.85rem;
    font-weight: 700;
    padding: 0.3rem 0.65rem;
    border-radius: var(--radius-md);
    background: rgba(16, 185, 129, 0.15);
    color: var(--success-color);
  }

  .actions-group {
    display: flex;
    align-items: center;
    gap: 0.65rem;
  }
</style>
