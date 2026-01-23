<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import HelpTip from "./HelpTip.svelte";
  import type { ConfigStatus } from "../../types";

  let { creationType = $bindable(), isCreating }: { creationType: 'agent' | 'pipeline' | 'auto-pipeline', isCreating: boolean } = $props();

  let hasApiKey = $state(true); // Default to true to avoid flash of disabled state
  let configLoaded = $state(false);

  const agentTypeImage = new URL("../../assets/creation-type/agent.svg", import.meta.url).href;
  const pipelineTypeImage = new URL("../../assets/creation-type/pipeline.svg", import.meta.url).href;
  const autoPipelineTypeImage = new URL(
    "../../assets/creation-type/auto-pipeline.svg",
    import.meta.url,
  ).href;

  onMount(async () => {
    try {
      const config = await invoke<ConfigStatus>("get_config_status");
      hasApiKey = config.api_keys.some((key) => key.is_configured);
      configLoaded = true;
      // If no API key and current selection requires one, switch to single agent
      if (!hasApiKey && (creationType === 'auto-pipeline' || creationType === 'pipeline')) {
        creationType = 'agent';
      }
    } catch (err) {
      console.error("Failed to fetch config:", err);
      configLoaded = true;
    }
  });

  // Check if an option requires API key
  function requiresApiKey(type: string): boolean {
    return type === 'auto-pipeline' || type === 'pipeline';
  }

  // Check if an option should be disabled
  function isOptionDisabled(type: string): boolean {
    if (isCreating) return true;
    if (!configLoaded) return type !== 'agent'; // Disable AI options until config loads
    return requiresApiKey(type) && !hasApiKey;
  }
</script>

<div class="section-type">
  <div class="radio-group type-grid">
    <label class="radio-option" class:selected={creationType === 'agent'}>
      <input type="radio" bind:group={creationType} value="agent" disabled={isCreating} />
      <div class="corner-help">
        <HelpTip
          placement="top"
          text="Starts one agent focused on a single ongoing task in the selected working directory."
        />
      </div>
      <div class="type-image" aria-hidden="true">
        <img src={agentTypeImage} alt="" />
      </div>
      <div class="radio-content">
        <strong>Single Agent</strong>
        <span>Focused mission</span>
      </div>
    </label>
    <label class="radio-option" class:selected={creationType === 'auto-pipeline'} class:locked={isOptionDisabled('auto-pipeline') && !isCreating}>
      <input type="radio" bind:group={creationType} value="auto-pipeline" disabled={isOptionDisabled('auto-pipeline')} />
      <div class="corner-help">
        <HelpTip
          placement="top"
          text="Starts the automated pipeline (Ralph) that tries to help end-to-end with minimal configuration."
        />
      </div>
      <div class="type-image" aria-hidden="true">
        <img src={autoPipelineTypeImage} alt="" />
      </div>
      <div class="radio-content">
        <strong>Ralphline</strong>
        <span>I'm helping!</span>
      </div>
      {#if isOptionDisabled('auto-pipeline') && !isCreating}
        <div class="locked-overlay">
          <span class="lock-icon">🔒</span>
          <span class="lock-message">Enter API key to unlock</span>
        </div>
      {/if}
    </label>
    <label class="radio-option" class:selected={creationType === 'pipeline'} class:locked={isOptionDisabled('pipeline') && !isCreating}>
      <input type="radio" bind:group={creationType} value="pipeline" disabled={isOptionDisabled('pipeline')} />
      <div class="corner-help">
        <HelpTip
          placement="top"
          text="Runs a multi-phase workflow (planning/execution/verification) with configurable checkpoints."
        />
      </div>
      <div class="type-image" aria-hidden="true">
        <img src={pipelineTypeImage} alt="" />
      </div>
      <div class="radio-content">
        <strong>Custom</strong>
        <span>Multi-phase</span>
      </div>
      {#if isOptionDisabled('pipeline') && !isCreating}
        <div class="locked-overlay">
          <span class="lock-icon">🔒</span>
          <span class="lock-message">Enter API key to unlock</span>
        </div>
      {/if}
    </label>
  </div>
</div>

<style>
  .type-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-3);
    margin-bottom: var(--space-4);
  }

  @media (max-width: 500px) {
    .type-grid {
      grid-template-columns: 1fr;
    }
  }

  .radio-option {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: var(--space-2);
    padding: var(--space-3);
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    cursor: pointer;
    transition: all var(--transition-fast);
    height: 100%;
  }

  .radio-option:hover:not(:has(input:disabled)) {
    background: var(--bg-elevated);
    border-color: rgba(255, 255, 255, 0.15);
  }

  .radio-option.selected {
    background: rgba(232, 102, 77, 0.1);
    border-color: var(--accent-hex);
  }

  .radio-option input[type="radio"] {
    position: absolute;
    inset: 0;
    opacity: 0;
    margin: 0;
    pointer-events: none;
  }

  .radio-option:focus-within {
    border-color: var(--accent-hex);
    box-shadow: 0 0 0 3px rgba(232, 102, 77, 0.15);
  }

  .type-image {
    width: 48px;
    height: 48px;
    border-radius: var(--radius-md);
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    display: grid;
    place-items: center;
    flex-shrink: 0;
  }

  .type-image img {
    width: 28px;
    height: 28px;
    display: block;
  }

  .radio-option.selected .type-image {
    background: rgba(232, 102, 77, 0.15);
    border-color: var(--accent-hex);
  }

  .radio-content {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .corner-help {
    position: absolute;
    bottom: var(--space-2);
    right: var(--space-2);
    z-index: 10;
  }

  .radio-content strong {
    font-size: var(--text-sm);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
  }

  .radio-content span {
    font-size: var(--text-xs);
    color: var(--text-muted);
    line-height: 1.3;
  }

  .radio-option.selected .radio-content strong {
    color: var(--accent-hex);
  }

  /* Locked state styles */
  .radio-option.locked {
    opacity: 0.5;
    cursor: not-allowed;
    pointer-events: none;
    position: relative;
  }

  .radio-option.locked:hover {
    background: var(--bg-tertiary);
    border-color: var(--border);
  }

  .locked-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    background: rgba(15, 15, 15, 0.9);
    border-radius: calc(var(--radius-lg) - 1px);
    pointer-events: auto;
    cursor: not-allowed;
    gap: var(--space-1);
  }

  .lock-icon {
    font-size: 16px;
  }

  .lock-message {
    font-size: var(--text-xs);
    color: var(--text-muted);
    text-align: center;
    padding: 0 var(--space-2);
  }
</style>
