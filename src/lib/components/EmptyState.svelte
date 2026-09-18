<script lang="ts">
  import { APP_CONFIG } from '$lib/config/appConfig';

  let { onChooseFiles }: { onChooseFiles: () => void } = $props();
</script>

<div class="empty-state-container" role="region" aria-label="Dropzone">
  <div class="icon-wrap">
    <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
      <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
      <polyline points="17 8 12 3 7 8" />
      <line x1="12" y1="3" x2="12" y2="15" />
    </svg>
  </div>

  <h1 class="title">Compress Images</h1>
  <p class="tagline">{APP_CONFIG.tagline}</p>

  <p class="drop-hint">Drop JPG, PNG, WebP, or HEIC files here</p>

  <div class="format-badges" aria-label="Supported formats">
    <span class="badge">JPG</span>
    <span class="dot">·</span>
    <span class="badge">PNG</span>
    <span class="dot">·</span>
    <span class="badge">WebP</span>
    <span class="dot">·</span>
    <span class="badge">HEIC</span>
  </div>

  <button class="btn primary-btn" onclick={onChooseFiles} aria-label="Choose files to compress">
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
      <path d="M12 5v14M5 12h14"/>
    </svg>
    Choose Files
    <kbd class="shortcut">⌘O</kbd>
  </button>
</div>

<style>
  .empty-state-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 440px;
    padding: 3rem 2rem;
    text-align: center;
    background: var(--surface-bg);
    border: 2px dashed var(--border-color);
    border-radius: var(--radius-xl);
    transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .icon-wrap {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 72px;
    height: 72px;
    margin-bottom: 1.25rem;
    border-radius: 50%;
    background: var(--accent-light);
    color: var(--accent-color);
  }

  .title {
    font-size: 1.75rem;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0 0 0.5rem;
    letter-spacing: -0.02em;
  }

  .tagline {
    font-size: 0.95rem;
    color: var(--text-secondary);
    margin: 0 0 1.5rem;
    max-width: 380px;
    line-height: 1.45;
  }

  .drop-hint {
    font-size: 0.9rem;
    font-weight: 500;
    color: var(--text-primary);
    margin: 0 0 0.5rem;
  }

  .format-badges {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    margin-bottom: 2rem;
    font-size: 0.8rem;
    color: var(--text-tertiary);
  }

  .badge {
    font-weight: 500;
    color: var(--text-secondary);
  }

  .dot {
    color: var(--text-tertiary);
  }

  .shortcut {
    font-size: 0.75rem;
    padding: 0.15rem 0.4rem;
    border-radius: var(--radius-sm);
    background: rgba(255, 255, 255, 0.2);
    color: #fff;
    margin-left: 0.35rem;
    font-family: var(--font-mono);
  }
</style>
