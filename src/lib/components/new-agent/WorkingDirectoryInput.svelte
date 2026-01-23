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
    gap: var(--space-sm);
    margin-bottom: var(--space-sm);
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .label-text svg {
    width: 18px;
    height: 18px;
    color: var(--accent);
  }

  .input-group {
    display: flex;
    gap: var(--space-sm);
  }

  .input-group input {
    flex: 1;
    padding: var(--space-md);
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 10px;
    color: var(--text-primary);
    font-size: 14px;
    transition: all 0.2s ease;
  }

  .input-group input:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-glow);
  }

  .browse-btn {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: 0 var(--space-md);
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 10px;
    color: var(--text-secondary);
    font-size: 14px;
    font-weight: 500;
    white-space: nowrap;
    transition: all 0.2s ease;
  }

  .browse-btn:hover:not(:disabled) {
    background: var(--bg-tertiary);
    border-color: var(--accent);
    color: var(--text-primary);
  }

  .browse-btn svg {
    width: 16px;
    height: 16px;
  }
</style>
