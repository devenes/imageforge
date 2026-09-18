<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import type { Snippet } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { invoke } from '@tauri-apps/api/core';
  import { getCurrentWebview } from '@tauri-apps/api/webview';
  import { appState } from '$lib/stores/appState.svelte';
  import type { ImageInfo } from '$lib/types';

  let { children }: { children?: Snippet } = $props();
  let isDragOver = $state(false);

  // Tauri-native drag-drop listener (unlistens on component destroy)
  let unlisten: (() => void) | null = null;

  onMount(async () => {
    try {
      unlisten = await getCurrentWebview().onDragDropEvent((event) => {
        const { type } = event.payload;

        if (type === 'enter') {
          isDragOver = true;
        } else if (type === 'over') {
          isDragOver = true;
        } else if (type === 'drop') {
          isDragOver = false;
          const paths = (event.payload as { type: 'drop'; paths: string[] }).paths;
          if (paths && paths.length > 0) {
            processPaths(paths);
          }
        } else if (type === 'leave') {
          isDragOver = false;
        }
      });
    } catch (e) {
      console.error('Failed to register drag-drop listener:', e);
    }
  });

  onDestroy(() => {
    unlisten?.();
  });

  export async function chooseFiles() {
    try {
      const selected = await open({
        multiple: true,
        filters: [
          {
            name: 'Images',
            extensions: ['jpg', 'jpeg', 'png', 'webp', 'heic', 'heif'],
          },
        ],
      });

      if (selected && Array.isArray(selected) && selected.length > 0) {
        await processPaths(selected);
      } else if (typeof selected === 'string') {
        await processPaths([selected]);
      }
    } catch (e) {
      appState.setError(`Failed to open file picker: ${e}`);
    }
  }

  async function processPaths(paths: string[]) {
    try {
      appState.setError(null);
      const inspected = await invoke<ImageInfo[]>('inspect_files', { paths });
      if (inspected.length === 0 && paths.length > 0) {
        appState.setError(
          'Unsupported file type. Please provide JPG, PNG, WebP, or HEIC images.'
        );
      } else if (inspected.length < paths.length) {
        const skipped = paths.length - inspected.length;
        appState.setError(
          `Added ${inspected.length} images (${skipped} unsupported file(s) ignored).`
        );
        appState.addFiles(inspected);
      } else {
        appState.addFiles(inspected);
      }
    } catch (err) {
      appState.setError(`Error inspecting images: ${err}`);
    }
  }
</script>

<div
  class="drop-wrapper"
  class:drag-over={isDragOver}
  role="region"
  aria-label="Drag and Drop Area"
>
  {#if isDragOver}
    <div class="drag-indicator">
      <div class="drag-content">
        <svg
          width="36"
          height="36"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
          <polyline points="17 8 12 3 7 8" />
          <line x1="12" y1="3" x2="12" y2="15" />
        </svg>
        <span>Drop images to add to queue</span>
      </div>
    </div>
  {/if}

  {@render children?.()}
</div>

<style>
  .drop-wrapper {
    position: relative;
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .drop-wrapper.drag-over {
    opacity: 0.95;
  }

  .drag-indicator {
    position: absolute;
    inset: 0;
    z-index: 50;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(37, 99, 235, 0.08);
    backdrop-filter: blur(4px);
    border: 2px dashed var(--accent-color);
    border-radius: var(--radius-lg);
    pointer-events: none;
    animation: fadeIn 0.15s ease-out;
  }

  .drag-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.75rem;
    color: var(--accent-color);
    font-weight: 600;
    font-size: 1.1rem;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: scale(0.98);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }
</style>
