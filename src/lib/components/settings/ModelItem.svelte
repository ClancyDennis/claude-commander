<script lang="ts">
  import type { ModelConfig } from "$lib/types";
  import HelpTip from "../new-agent/HelpTip.svelte";

  let {
    model,
    isEditing,
    editedValue,
    availableModels,
    description,
    onModelChange,
  }: {
    model: ModelConfig;
    isEditing: boolean;
    editedValue: string;
    availableModels: string[];
    description: string;
    onModelChange: (value: string) => void;
  } = $props();
</script>

<div class="model-item">
  <div class="model-header">
    <span class="model-name">{model.name}</span>
    {#if !isEditing && model.is_default}
      <span class="default-badge">Default</span>
    {/if}
    <HelpTip text={description} placement="top" />
  </div>
  {#if isEditing}
    <select
      value={editedValue}
      onchange={(e) => onModelChange(e.currentTarget.value)}
      class="model-select"
    >
      <option value="">Use default</option>
      {#each availableModels as modelOption}
        <option value={modelOption}>{modelOption}</option>
      {/each}
    </select>
  {:else}
    <div class="model-value">
      <code>{model.value ?? model.default_value ?? "—"}</code>
      {#if model.is_default && model.default_value}
        <span class="default-indicator">(default)</span>
      {/if}
    </div>
  {/if}
</div>

<style>
  .model-item {
    padding: var(--space-4);
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-hex);
  }

  .model-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-2);
  }

  .model-name {
    font-weight: var(--font-semibold);
    color: var(--text-primary);
    font-family: monospace;
  }

  .default-badge {
    font-size: var(--text-xs);
    padding: 2px var(--space-2);
    background: rgba(232, 102, 77, 0.2);
    color: var(--accent-hex);
    border-radius: var(--radius-sm);
    font-weight: var(--font-medium);
  }

  .model-value code {
    font-family: monospace;
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }

  .model-select {
    width: 100%;
    padding: var(--space-2) var(--space-3);
    background: var(--bg-primary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: var(--text-sm);
    cursor: pointer;
  }

  .model-select:focus {
    outline: none;
    border-color: var(--accent-hex);
    box-shadow: 0 0 0 3px var(--accent-glow);
  }

  .default-indicator {
    font-size: var(--text-xs);
    color: var(--text-muted);
    margin-left: var(--space-2);
    font-style: italic;
  }
</style>
