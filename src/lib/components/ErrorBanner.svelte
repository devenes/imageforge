<script lang="ts">
  import { appState } from '$lib/stores/appState.svelte';

  function dismiss() {
    appState.setError(null);
  }
</script>

{#if appState.errorBanner}
  <div class="error-banner" role="alert">
    <div class="error-icon">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="10"/>
        <line x1="12" y1="8" x2="12" y2="12"/>
        <line x1="12" y1="16" x2="12.01" y2="16"/>
      </svg>
    </div>
    <span class="error-text">{appState.errorBanner}</span>
    <button class="dismiss-btn" onclick={dismiss} aria-label="Dismiss error">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <line x1="18" y1="6" x2="6" y2="18"/>
        <line x1="6" y1="6" x2="18" y2="18"/>
      </svg>
    </button>
  </div>
{/if}

<style>
  .error-banner {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    padding: 0.6rem 0.85rem;
    background: rgba(239, 68, 68, 0.09);
    border: 1px solid rgba(239, 68, 68, 0.25);
    border-radius: var(--radius-md);
    margin-bottom: 0.75rem;
    animation: fadeIn 0.15s ease-out;
  }

  .error-icon {
    display: flex;
    align-items: center;
    color: var(--error-color);
  }

  .error-text {
    font-size: 0.82rem;
    font-weight: 500;
    color: var(--error-color);
    flex: 1;
  }

  .dismiss-btn {
    border: none;
    background: transparent;
    color: var(--error-color);
    cursor: pointer;
    padding: 0.2rem;
    display: flex;
    align-items: center;
    opacity: 0.75;
  }

  .dismiss-btn:hover {
    opacity: 1;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(-4px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
