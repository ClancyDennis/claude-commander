<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import type { InstructionFileInfo } from "../../types";
  import HelpTip from "./HelpTip.svelte";

  let {
    workingDir,
    selectedInstructions = $bindable(new Set()),
    isCreating
  }: {
    workingDir: string,
    selectedInstructions: Set<string>,
    isCreating: boolean
  } = $props();

  let instructionFiles = $state<Array<InstructionFileInfo>>([]);
  let isLoadingInstructions = $state(false);
  let showInstructionSelector = $state(false);

  onMount(() => {
    loadInstructionFiles();
  });

  async function loadInstructionFiles() {
    try {
      isLoadingInstructions = true;
      const files = await invoke<Array<InstructionFileInfo>>("list_instruction_files", {
        workingDir: "",
      });
      instructionFiles = files;
    } catch (e) {
      console.error("Failed to load instruction files:", e);
      instructionFiles = [];
    } finally {
      isLoadingInstructions = false;
    }
  }

  function toggleInstructionSelection(fileId: string) {
    if (selectedInstructions.has(fileId)) {
      selectedInstructions.delete(fileId);
    } else {
      selectedInstructions.add(fileId);
    }
    selectedInstructions = new Set(selectedInstructions);
  }
</script>

<label>
  <div class="label-row">
    <span class="label-text">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
        <polyline points="14 2 14 8 20 8"/>
      </svg>
      Instructions <span class="optional-label">(Opt)</span>
      <HelpTip
        text="Select instruction files to include as context for the agent. Manage instructions from the Instructions panel in the sidebar."
      />
    </span>
  </div>

  <div class="instructions-section">
    <button
      type="button"
      class="instructions-toggle"
      onclick={() => showInstructionSelector = !showInstructionSelector}
      disabled={isCreating || isLoadingInstructions}
    >
      <span class="toggle-content">
        {#if selectedInstructions.size > 0}
          <span class="badge">{selectedInstructions.size}</span>
        {/if}
        {selectedInstructions.size > 0
          ? `${selectedInstructions.size} selected`
          : 'Select instructions'}
      </span>
      <svg class="chevron" class:rotated={showInstructionSelector} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="6 9 12 15 18 9"/>
      </svg>
    </button>

    {#if showInstructionSelector}
      <div class="instructions-dropdown animate-slide-up">
        {#if isLoadingInstructions}
          <div class="loading-state">
            <span class="spinner"></span>
            Loading...
          </div>
        {:else if instructionFiles.length === 0}
          <div class="empty-state">
            <p>No instruction files found</p>
            <p class="hint">Create instructions from the Instructions panel</p>
          </div>
        {:else}
          <div class="instructions-list">
            {#each instructionFiles as file (file.id)}
              <label class="instruction-item" class:selected={selectedInstructions.has(file.id)}>
                <input
                  type="checkbox"
                  checked={selectedInstructions.has(file.id)}
                  onchange={() => toggleInstructionSelection(file.id)}
                  disabled={isCreating}
                />
                <span class="instruction-name">{file.name}</span>
                <span class="file-type">.{file.fileType}</span>
              </label>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
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

  .optional-label {
    color: var(--text-muted);
    font-weight: 400;
  }

  .label-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-2);
    gap: var(--space-2);
  }

  .instructions-section {
    position: relative;
  }

  .instructions-toggle {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-2) var(--space-3);
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .instructions-toggle:hover:not(:disabled) {
    background: var(--bg-elevated);
    border-color: rgba(255, 255, 255, 0.15);
    color: var(--text-primary);
  }

  .instructions-toggle:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .toggle-content {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-width: 0;
  }

  .badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 18px;
    height: 18px;
    padding: 0 5px;
    background: var(--accent-hex);
    color: white;
    font-size: 11px;
    font-weight: var(--font-semibold);
    border-radius: var(--radius-full);
    flex-shrink: 0;
  }

  .instructions-toggle .chevron {
    width: 14px;
    height: 14px;
    transition: transform 0.2s ease;
    flex-shrink: 0;
  }

  .instructions-toggle .chevron.rotated {
    transform: rotate(180deg);
  }

  .instructions-dropdown {
    margin-top: var(--space-2);
    padding: var(--space-1);
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    max-height: 180px;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .loading-state,
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-3);
    gap: var(--space-1);
    text-align: center;
    color: var(--text-muted);
    font-size: var(--text-xs);
  }

  .empty-state .hint {
    font-size: 10px;
    opacity: 0.7;
  }

  .instructions-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .instruction-item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2);
    background: var(--bg-secondary);
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .instruction-item:hover {
    background: var(--bg-elevated);
  }

  .instruction-item.selected {
    background: rgba(var(--accent-rgb), 0.1);
    border-color: rgba(var(--accent-rgb), 0.3);
  }

  .instruction-item input[type="checkbox"] {
    margin: 0;
    width: 14px;
    height: 14px;
    flex-shrink: 0;
  }

  .instruction-name {
    flex: 1;
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .file-type {
    font-size: 10px;
    padding: 2px 5px;
    background: var(--bg-tertiary);
    border-radius: 3px;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .spinner {
    width: 16px;
    height: 16px;
    border: 2px solid rgba(255, 255, 255, 0.2);
    border-top-color: var(--text-secondary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
