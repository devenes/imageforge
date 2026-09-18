<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';

  import DropZone from '$lib/components/DropZone.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import FileQueue from '$lib/components/FileQueue.svelte';
  import CompressionControls from '$lib/components/CompressionControls.svelte';
  import ProgressView from '$lib/components/ProgressView.svelte';
  import ResultsSummary from '$lib/components/ResultsSummary.svelte';
  import PreferencesModal from '$lib/components/PreferencesModal.svelte';
  import ErrorBanner from '$lib/components/ErrorBanner.svelte';

  import { appState } from '$lib/stores/appState.svelte';
  import { APP_CONFIG } from '$lib/config/appConfig';
  import type { CompressionResult, ProgressUpdate } from '$lib/types';

  let dropZoneRef: any;

  async function handleStartCompress() {
    if (appState.isProcessing || appState.isQueueEmpty) return;

    appState.isProcessing = true;
    appState.hasCompletedBatch = false;
    appState.results = [];

    // Mark all items as processing / queued
    for (const item of appState.queue) {
      item.status = 'QUEUED';
      item.result = undefined;
    }

    try {
      const items = appState.queue.map((q) => q.info);
      await invoke('start_batch', {
        items,
        settings: {
          preset: appState.settings.preset,
          outputMode: appState.settings.outputMode,
          preserveMetadata: appState.settings.preserveMetadata,
          preserveColorProfile: appState.settings.preserveColorProfile,
          customOutputDir: appState.settings.customOutputDir,
          filenameSuffix: appState.settings.filenameSuffix,
        },
      });
    } catch (err) {
      appState.isProcessing = false;
      appState.setError(`Failed to start compression: ${err}`);
    }
  }

  function handleChooseFiles() {
    dropZoneRef?.chooseFiles();
  }

  function handleCompressMore() {
    appState.resetForNewBatch();
  }

  onMount(() => {
    // Keyboard shortcuts
    const handleKeydown = (e: KeyboardEvent) => {
      const isMeta = e.metaKey || e.ctrlKey;
      if (isMeta && e.key === 'o') {
        e.preventDefault();
        handleChooseFiles();
      } else if (isMeta && e.key === 'k') {
        e.preventDefault();
        appState.clearQueue();
      } else if (isMeta && e.key === 'Enter') {
        e.preventDefault();
        handleStartCompress();
      } else if (isMeta && e.key === ',') {
        e.preventDefault();
        appState.showPreferences = !appState.showPreferences;
      }
    };

    window.addEventListener('keydown', handleKeydown);

    // Tauri event listeners
    const unlistenProgress = listen<ProgressUpdate>('file_progress', (event) => {
      appState.progress = event.payload;
    });

    const unlistenCompleted = listen<CompressionResult>('file_completed', (event) => {
      appState.updateItemStatus(event.payload.id, event.payload.status, event.payload);
    });

    const unlistenBatchCompleted = listen<CompressionResult[]>('batch_completed', (event) => {
      appState.finishBatch(event.payload);
    });

    const unlistenBatchCancelled = listen('batch_cancelled', () => {
      appState.cancelBatch();
    });

    return () => {
      window.removeEventListener('keydown', handleKeydown);
      unlistenProgress.then((u) => u());
      unlistenCompleted.then((u) => u());
      unlistenBatchCompleted.then((u) => u());
      unlistenBatchCancelled.then((u) => u());
    };
  });
</script>

<div class="app-layout">
  <!-- Titlebar area with native drag region -->
  <header class="app-header" data-tauri-drag-region>
    <div class="window-controls-spacer"></div>
    <div class="header-center">
      <span class="app-title">{APP_CONFIG.name}</span>
    </div>
    <div class="header-right">
      <button
        class="icon-btn"
        onclick={() => (appState.showPreferences = true)}
        title="Preferences (⌘,)"
        aria-label="Settings"
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/>
          <circle cx="12" cy="12" r="3"/>
        </svg>
      </button>
    </div>
  </header>

  <!-- Main Container -->
  <main class="main-content">
    <ErrorBanner />

    <DropZone bind:this={dropZoneRef}>
      {#if appState.isQueueEmpty}
        <EmptyState onChooseFiles={handleChooseFiles} />
      {:else}
        <div class="workspace-flow">
          {#if appState.hasCompletedBatch}
            <ResultsSummary onCompressMore={handleCompressMore} />
          {/if}

          <ProgressView />

          <FileQueue onAddMore={handleChooseFiles} />

          <CompressionControls onStartCompress={handleStartCompress} />
        </div>
      {/if}
    </DropZone>
  </main>

  <PreferencesModal />
</div>

<style>
  .app-layout {
    display: flex;
    flex-direction: column;
    width: 100vw;
    height: 100vh;
    background: var(--bg-app);
  }

  .app-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 38px;
    padding: 0 1rem;
    border-bottom: 1px solid var(--border-color-subtle);
    background: var(--surface-bg);
    user-select: none;
    flex-shrink: 0;
  }

  .window-controls-spacer {
    width: 68px; /* space for macOS traffic lights */
  }

  .header-center {
    display: flex;
    align-items: center;
  }

  .app-title {
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--text-secondary);
    letter-spacing: -0.01em;
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border-radius: var(--radius-sm);
    border: none;
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
  }

  .icon-btn:hover {
    background: var(--surface-bg-subtle);
    color: var(--text-primary);
  }

  .main-content {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    padding: 1.25rem 1.5rem;
    overflow: hidden;
  }

  .workspace-flow {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    flex: 1;
    min-height: 0;
  }
</style>
