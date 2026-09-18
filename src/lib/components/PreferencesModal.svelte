<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { appState } from '$lib/stores/appState.svelte';

  function close() {
    appState.showPreferences = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      close();
    }
  }

  async function chooseOutputDir() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      });
      if (selected && typeof selected === 'string') {
        appState.settings.customOutputDir = selected;
      }
    } catch (e) {
      console.error('Failed to select directory:', e);
    }
  }

  function resetOutputDir() {
    appState.settings.customOutputDir = null;
  }
</script>

{#if appState.showPreferences}
  <div
    class="modal-backdrop"
    onclick={close}
    onkeydown={handleKeydown}
    role="button"
    tabindex="0"
    aria-label="Close preferences backdrop"
  >
    <div
      class="modal-card"
      role="dialog"
      aria-modal="true"
      aria-labelledby="pref-title"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <div class="modal-header">
        <h2 id="pref-title" class="modal-title">Settings</h2>
        <button class="close-btn" onclick={close} aria-label="Close">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <div class="modal-body">
        <!-- Output Destination -->
        <section class="pref-section">
          <h3 class="section-title">Output Location</h3>
          <div class="pref-row">
            <div class="pref-label-desc">
              <span class="label">Destination Folder</span>
              <span class="desc">Where compressed images will be saved</span>
            </div>
            <div class="folder-control">
              {#if appState.settings.customOutputDir}
                <span class="folder-path" title={appState.settings.customOutputDir}>
                  {appState.settings.customOutputDir}
                </span>
                <button class="btn secondary-btn small-btn" onclick={resetOutputDir}>
                  Use Same Folder
                </button>
              {:else}
                <span class="folder-path same-folder">Same folder as original</span>
                <button class="btn secondary-btn small-btn" onclick={chooseOutputDir}>
                  Choose Folder...
                </button>
              {/if}
            </div>
          </div>

          <div class="pref-row">
            <div class="pref-label-desc">
              <span class="label">Filename Suffix</span>
              <span class="desc">Appended before the extension (e.g. photo-compressed.jpg)</span>
            </div>
            <input
              type="text"
              class="text-input"
              bind:value={appState.settings.filenameSuffix}
              placeholder="-compressed"
            />
          </div>
        </section>

        <!-- Metadata & Color Profiles -->
        <section class="pref-section">
          <h3 class="section-title">Metadata & Privacy</h3>

          <label class="toggle-row">
            <div class="pref-label-desc">
              <span class="label">Preserve Metadata (EXIF / XMP)</span>
              <span class="desc">Preserve camera details, date, and orientation tags</span>
            </div>
            <input
              type="checkbox"
              class="checkbox"
              bind:checked={appState.settings.preserveMetadata}
            />
          </label>

          <label class="toggle-row">
            <div class="pref-label-desc">
              <span class="label">Preserve Color Profiles (ICC / Display P3)</span>
              <span class="desc">Maintains wide-gamut and accurate screen colors</span>
            </div>
            <input
              type="checkbox"
              class="checkbox"
              bind:checked={appState.settings.preserveColorProfile}
            />
          </label>
        </section>
      </div>

      <div class="modal-footer">
        <button class="btn primary-btn" onclick={close}>Done</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 100;
    background: rgba(0, 0, 0, 0.4);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    animation: fadeIn 0.15s ease-out;
  }

  .modal-card {
    width: 480px;
    max-width: 90vw;
    background: var(--surface-bg);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-xl);
    box-shadow: 0 16px 36px rgba(0, 0, 0, 0.2);
    overflow: hidden;
    animation: scaleIn 0.15s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--border-color-subtle);
  }

  .modal-title {
    font-size: 1.05rem;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .close-btn {
    border: none;
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
    display: flex;
    align-items: center;
    padding: 0.25rem;
    border-radius: var(--radius-sm);
  }

  .close-btn:hover {
    color: var(--text-primary);
    background: var(--surface-bg-subtle);
  }

  .modal-body {
    padding: 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .pref-section {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
  }

  .section-title {
    font-size: 0.72rem;
    font-weight: 700;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin: 0;
  }

  .pref-row, .toggle-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }

  .toggle-row {
    cursor: pointer;
  }

  .pref-label-desc {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .label {
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--text-primary);
  }

  .desc {
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .folder-control {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .folder-path {
    font-size: 0.75rem;
    max-width: 140px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--text-secondary);
  }

  .folder-path.same-folder {
    font-style: italic;
    color: var(--text-tertiary);
  }

  .text-input {
    width: 120px;
    padding: 0.35rem 0.5rem;
    font-size: 0.82rem;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    background: var(--surface-bg-subtle);
    color: var(--text-primary);
  }

  .checkbox {
    width: 18px;
    height: 18px;
    accent-color: var(--accent-color);
    cursor: pointer;
  }

  .small-btn {
    padding: 0.25rem 0.55rem;
    font-size: 0.75rem;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    padding: 0.85rem 1.25rem;
    border-top: 1px solid var(--border-color-subtle);
    background: var(--surface-bg-subtle);
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  @keyframes scaleIn {
    from { opacity: 0; transform: scale(0.96); }
    to { opacity: 1; transform: scale(1); }
  }
</style>
