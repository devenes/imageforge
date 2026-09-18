<script lang="ts">
  import { appState } from '$lib/stores/appState.svelte';
  import type { CompressionPreset, OutputMode } from '$lib/types';

  let { onStartCompress }: { onStartCompress: () => void } = $props();

  const presets: { id: CompressionPreset; label: string; desc: string }[] = [
    {
      id: 'best_quality',
      label: 'Best Quality',
      desc: 'Minimum degradation',
    },
    {
      id: 'balanced',
      label: 'Balanced',
      desc: 'Great quality & size reduction (Recommended)',
    },
    {
      id: 'smallest',
      label: 'Smallest',
      desc: 'Maximum file savings',
    },
  ];

  function setPreset(p: CompressionPreset) {
    if (appState.isProcessing) return;
    appState.settings.preset = p;
  }

  function setOutputMode(m: OutputMode) {
    if (appState.isProcessing) return;
    appState.settings.outputMode = m;
  }
</script>

<div class="controls-panel">
  <!-- Presets -->
  <div class="control-group">
    <span class="group-label">Quality Preset</span>
    <div class="segmented-control" role="radiogroup" aria-label="Quality Preset">
      {#each presets as p}
        <button
          type="button"
          role="radio"
          aria-checked={appState.settings.preset === p.id}
          class="segment-btn"
          class:active={appState.settings.preset === p.id}
          disabled={appState.isProcessing}
          onclick={() => setPreset(p.id)}
          title={p.desc}
        >
          {p.label}
        </button>
      {/each}
    </div>
  </div>

  <!-- Output Format Selection -->
  <div class="control-group">
    <span class="group-label">Output Format</span>
    <div class="segmented-control" role="radiogroup" aria-label="Output Format">
      <button
        type="button"
        role="radio"
        aria-checked={appState.settings.outputMode === 'same_format'}
        class="segment-btn"
        class:active={appState.settings.outputMode === 'same_format'}
        disabled={appState.isProcessing}
        onclick={() => setOutputMode('same_format')}
        title="Keep original format (JPG, PNG, WebP, HEIC)"
      >
        Keep Original Format
      </button>

      <button
        type="button"
        role="radio"
        aria-checked={appState.settings.outputMode === 'convert_to_jpg'}
        class="segment-btn"
        class:active={appState.settings.outputMode === 'convert_to_jpg'}
        disabled={appState.isProcessing}
        onclick={() => setOutputMode('convert_to_jpg')}
        title="Convert all images to JPG (alpha composited over white)"
      >
        Convert to JPG
      </button>
    </div>
  </div>

  <!-- Primary Action -->
  <div class="action-group">
    <button
      class="btn primary-btn compress-btn"
      disabled={appState.isProcessing || appState.isQueueEmpty}
      onclick={onStartCompress}
      aria-label="Start compression"
    >
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M4 14.899A7 7 0 1 1 15.71 8h1.79a4.5 4.5 0 0 1 2.5 8.242"/>
        <path d="M12 12v9"/>
        <path d="m8 17 4 4 4-4"/>
      </svg>
      Compress
      <kbd class="shortcut">⌘↵</kbd>
    </button>
  </div>
</div>

<style>
  .controls-panel {
    display: flex;
    align-items: center;
    gap: 1.5rem;
    padding: 0.85rem 1.25rem;
    background: var(--surface-bg);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
  }

  .control-group {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .group-label {
    font-size: 0.72rem;
    font-weight: 600;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .segmented-control {
    display: flex;
    padding: 3px;
    background: var(--surface-bg-subtle);
    border: 1px solid var(--border-color-subtle);
    border-radius: var(--radius-md);
  }

  .segment-btn {
    border: none;
    background: transparent;
    padding: 0.35rem 0.75rem;
    font-size: 0.8rem;
    font-weight: 500;
    color: var(--text-secondary);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .segment-btn:hover:not(:disabled) {
    color: var(--text-primary);
  }

  .segment-btn.active {
    background: var(--surface-bg);
    color: var(--text-primary);
    font-weight: 600;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
  }

  .action-group {
    margin-left: auto;
  }

  .compress-btn {
    padding: 0.55rem 1.25rem;
    font-size: 0.9rem;
  }

  .shortcut {
    font-size: 0.75rem;
    padding: 0.15rem 0.35rem;
    border-radius: var(--radius-sm);
    background: rgba(255, 255, 255, 0.25);
    color: #fff;
    margin-left: 0.4rem;
    font-family: var(--font-mono);
  }
</style>
