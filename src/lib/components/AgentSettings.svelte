<script lang="ts">
  import { onMount } from "svelte";
  import {
    agentPipelineSettings,
    updateAgentSettings,
    saveSettings,
    type AgentPipelineSettings,
  } from "../stores/pipelineSettings";

  let { agentId, onClose }: { agentId: string; onClose?: () => void } = $props();

  let settings = $state<AgentPipelineSettings | null>(null);

  onMount(() => {
    // Load settings for this agent
    const unsubscribe = agentPipelineSettings.subscribe((map) => {
      const agentSettings = map.get(agentId);
      if (agentSettings) {
        settings = { ...agentSettings };
      } else {
        // Use defaults
        settings = {
          agentId,
          enablePipeline: true,
          useAgentPool: true,
          poolPriority: "normal",
          enableOrchestration: true,
          autoDecompose: true,
          maxParallelTasks: 5,
          enableVerification: true,
          verificationStrategy: "weighted",
          verificationN: 3,
          confidenceThreshold: 0.8,
          enableCheckpoints: true,
          requirePlanReview: true,
          requireFinalReview: true,
          autoValidationCommand: "cargo check",
          autoApproveOnVerification: false,
          instructionFiles: [],
        };
      }
    });

    return () => unsubscribe();
  });

  function handleSave() {
    if (settings) {
      updateAgentSettings(agentId, settings);
      saveSettings();
      if (onClose) onClose();
    }
  }

  function handleCancel() {
    if (onClose) onClose();
  }
</script>

<div class="agent-settings">
  <header class="settings-header">
    <h2>Agent Settings</h2>
    <span class="agent-id">{agentId.substring(0, 8)}</span>
  </header>

  {#if settings}
    <div class="settings-content">
      <section class="settings-section">
        <div class="section-header">
          <h3>Display Settings</h3>
        </div>
        <p class="section-description">
          Configure how the agent output is displayed
        </p>
        <div class="setting-item">
          <!-- Placeholder for future display settings -->
          <p style="color: var(--text-muted); font-style: italic;">No display settings available yet.</p>
        </div>
      </section>
    </div>

    <footer class="settings-footer">
      <button class="btn btn-secondary" onclick={handleCancel}>Cancel</button>
      <button class="btn btn-primary" onclick={handleSave}>Save Settings</button>
    </footer>
  {:else}
    <div class="loading">Loading settings...</div>
  {/if}
</div>

<style>
  .agent-settings {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-secondary);
    border-radius: 12px;
    overflow: hidden;
  }

  .settings-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-lg);
    background: var(--bg-tertiary);
    border-bottom: 1px solid var(--border);
  }

  .settings-header h2 {
    margin: 0;
    font-size: 20px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .agent-id {
    padding: 4px 10px;
    background: var(--accent-glow);
    color: var(--accent);
    font-size: 12px;
    font-weight: 700;
    font-family: monospace;
    border-radius: 6px;
  }

  .settings-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-lg);
  }

  .settings-section {
    margin-bottom: var(--space-xl);
    padding: var(--space-lg);
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 10px;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-sm);
  }

  .section-header h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .section-description {
    margin: 0 0 var(--space-md) 0;
    font-size: 13px;
    color: var(--text-muted);
  }

  .setting-item {
    margin-bottom: var(--space-md);
  }

  .setting-item:last-child {
    margin-bottom: 0;
  }

  .setting-item label {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .setting-item input[type="text"],
  .setting-item input[type="number"],
  .setting-item select {
    padding: 8px 12px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-primary);
    font-size: 14px;
  }

  .setting-item input[type="range"] {
    width: 100%;
  }

  .range-value {
    font-size: 12px;
    color: var(--text-muted);
    margin-left: var(--space-sm);
  }

  .checkbox-label {
    flex-direction: row !important;
    align-items: center;
    gap: var(--space-sm) !important;
    cursor: pointer;
  }

  .checkbox-label input[type="checkbox"] {
    width: 18px;
    height: 18px;
    cursor: pointer;
  }

  /* Large toggle for master switch */
  .toggle-large {
    position: relative;
    display: inline-block;
    width: 60px;
    height: 32px;
  }

  .toggle-large input {
    opacity: 0;
    width: 0;
    height: 0;
  }

  .slider-large {
    position: absolute;
    cursor: pointer;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: var(--border);
    transition: 0.3s;
    border-radius: 32px;
  }

  .slider-large:before {
    position: absolute;
    content: "";
    height: 24px;
    width: 24px;
    left: 4px;
    bottom: 4px;
    background-color: white;
    transition: 0.3s;
    border-radius: 50%;
  }

  input:checked + .slider-large {
    background-color: var(--accent);
  }

  input:checked + .slider-large:before {
    transform: translateX(28px);
  }

  /* Regular toggle */
  .toggle {
    position: relative;
    display: inline-block;
    width: 48px;
    height: 24px;
  }

  .toggle input {
    opacity: 0;
    width: 0;
    height: 0;
  }

  .slider {
    position: absolute;
    cursor: pointer;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: var(--border);
    transition: 0.3s;
    border-radius: 24px;
  }

  .slider:before {
    position: absolute;
    content: "";
    height: 18px;
    width: 18px;
    left: 3px;
    bottom: 3px;
    background-color: white;
    transition: 0.3s;
    border-radius: 50%;
  }

  input:checked + .slider {
    background-color: var(--accent);
  }

  input:checked + .slider:before {
    transform: translateX(24px);
  }

  .settings-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: var(--space-md);
    padding: var(--space-lg);
    background: var(--bg-tertiary);
    border-top: 1px solid var(--border);
  }

  .btn {
    padding: 10px 20px;
    border: none;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .btn-secondary {
    background: var(--bg-secondary);
    color: var(--text-primary);
    border: 1px solid var(--border);
  }

  .btn-secondary:hover {
    background: var(--bg-tertiary);
  }

  .btn-primary {
    background: var(--accent);
    color: white;
  }

  .btn-primary:hover {
    background: var(--accent-hover, var(--accent));
    transform: scale(1.02);
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-xl);
    color: var(--text-muted);
  }

  .warning-banner {
    padding: var(--space-md);
    background: rgba(245, 158, 11, 0.15);
    border: 1px solid var(--warning);
    border-radius: 8px;
    color: var(--warning);
    font-size: 13px;
    font-weight: 600;
    margin-top: var(--space-md);
  }
</style>
