<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import HelpTip from "./HelpTip.svelte";

  let {
    workingDir = $bindable(),
    isCreating
  }: {
    workingDir: string;
    isCreating: boolean;
  } = $props();

  async function selectDirectory() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        defaultPath: workingDir,
      });

      if (selected) {
        workingDir = selected as string;
      }
    } catch (e) {
      console.error("Failed to open directory picker:", e);
    }
  }
</script>

<label>
  <span class="label-text">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
    </svg>
    Working Directory
    <HelpTip
      text="Where the agent/pipeline will read and write files. Pick the folder you want it to work inside."
    />
  </span>
  <div class="input-group">
    <input
      type="text"
      bind:value={workingDir}
      placeholder="/path/to/your/project"
      disabled={isCreating}
    />
    <button
      type="button"
      class="browse-btn"
      onclick={selectDirectory}
      disabled={isCreating}
      title="Browse for directory"
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
      </svg>
    </button>
  </div>
</label>

<style>
  label {
    display: block;
  }

  .label-text {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-2);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--text-primary);
  }

  .label-text svg {
    width: 16px;
    height: 16px;
    color: var(--accent-hex);
    flex-shrink: 0;
  }

  .input-group {
    display: flex;
    gap: var(--space-2);
  }

  .input-group input {
    flex: 1;
    min-width: 0;
    padding: var(--space-2) var(--space-3);
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text-primary);
    font-size: var(--text-sm);
    transition: all var(--transition-fast);
  }

  .input-group input:focus {
    outline: none;
    border-color: var(--accent-hex);
    box-shadow: 0 0 0 3px rgba(232, 102, 77, 0.15);
  }

  .browse-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    padding: 0;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    flex-shrink: 0;
    transition: all var(--transition-fast);
  }

  .browse-btn:hover:not(:disabled) {
    background: var(--bg-elevated);
    border-color: var(--accent-hex);
    color: var(--text-primary);
  }

  .browse-btn svg {
    width: 16px;
    height: 16px;
  }
</style>
