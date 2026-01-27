<script lang="ts">
  let {
    isEditing,
    isSaving,
    saveError,
    configPath,
    onSave,
    onCancel,
    onOpenConfigDirectory,
  }: {
    isEditing: boolean;
    isSaving: boolean;
    saveError: string | null;
    configPath: string;
    onSave: () => void;
    onCancel: () => void;
    onOpenConfigDirectory: () => void;
  } = $props();
</script>

{#if isEditing}
  {#if saveError}
    <div class="save-error">
      {saveError}
    </div>
  {/if}
  <div class="edit-actions">
    <button class="btn-primary" onclick={onSave} disabled={isSaving}>
      {isSaving ? "Saving..." : "Save Changes"}
    </button>
    <button class="btn-secondary" onclick={onCancel} disabled={isSaving}>
      Cancel
    </button>
  </div>
{:else}
  <!-- Configuration Actions -->
  <section class="config-section">
    <h3>Configuration</h3>
    <p class="config-path-info">
      Settings are stored at: <code>{configPath}</code>
    </p>
    <div class="config-actions">
      <button class="btn-secondary" onclick={onOpenConfigDirectory}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/>
        </svg>
        Open Config Folder
      </button>
    </div>
  </section>
{/if}

<style>
  .config-section {
    background: var(--bg-secondary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-lg);
    padding: var(--space-6);
    margin-bottom: var(--space-6);
  }

  .config-section h3 {
    margin: 0 0 var(--space-4) 0;
    font-size: var(--text-lg);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
  }

  .config-path-info {
    color: var(--text-secondary);
    margin: 0 0 var(--space-4) 0;
    font-size: var(--text-sm);
  }

  .config-path-info code {
    background: var(--bg-tertiary);
    padding: 2px var(--space-2);
    border-radius: var(--radius-sm);
    font-size: var(--text-xs);
  }

  .config-actions {
    display: flex;
    gap: var(--space-4);
  }

  .edit-actions {
    display: flex;
    gap: var(--space-4);
    padding: var(--space-6);
    background: var(--bg-secondary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-lg);
    margin-bottom: var(--space-6);
  }

  .save-error {
    padding: var(--space-4);
    background: var(--error-glow);
    border: 1px solid var(--error);
    border-radius: var(--radius-md);
    color: var(--error);
    font-size: var(--text-sm);
    margin-bottom: var(--space-4);
  }

  .btn-primary {
    padding: var(--space-3) var(--space-5);
    background: var(--accent-hex);
    color: white;
    border: none;
    border-radius: var(--radius-md);
    cursor: pointer;
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    transition: all var(--transition-fast);
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .btn-primary:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .btn-secondary {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-5);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    cursor: pointer;
    font-size: var(--text-sm);
    transition: all var(--transition-fast);
  }

  .btn-secondary:hover:not(:disabled) {
    background: var(--accent-glow);
    border-color: var(--accent-hex);
    color: var(--text-primary);
  }

  .btn-secondary:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .btn-secondary svg {
    width: 16px;
    height: 16px;
  }
</style>
