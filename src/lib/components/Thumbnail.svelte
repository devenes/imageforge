<script lang="ts">
  import { convertFileSrc } from '@tauri-apps/api/core';
  import type { ImageFormat } from '$lib/types';

  let { path, format, alt = '' }: { path: string; format: ImageFormat; alt?: string } = $props();

  let hasError = $state(false);
  let src = $derived(path ? convertFileSrc(path) : '');

  function handleError() {
    hasError = true;
  }
</script>

<div class="thumb-container" class:fallback={hasError || format === 'heic'}>
  {#if (format === 'jpg' || format === 'png' || format === 'webp') && !hasError}
    <img {src} {alt} onerror={handleError} class="thumb-img" loading="lazy" />
  {:else}
    <div class="format-tag">
      <span class="ext">{format.toUpperCase()}</span>
    </div>
  {/if}
</div>

<style>
  .thumb-container {
    width: 44px;
    height: 44px;
    border-radius: var(--radius-md);
    overflow: hidden;
    background: var(--surface-bg-alt);
    border: 1px solid var(--border-color);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .thumb-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .format-tag {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    background: var(--surface-bg-subtle);
  }

  .ext {
    font-size: 0.7rem;
    font-weight: 700;
    color: var(--text-secondary);
    letter-spacing: 0.04em;
  }
</style>
