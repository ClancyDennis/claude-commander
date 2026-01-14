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
    <h2>Pipeline Settings for Agent</h2>
    <span class="agent-id">{agentId.substring(0, 8)}</span>
  </header>

  {#if settings}
    <div class="settings-content">
      <!-- Master Toggle -->
      <section class="settings-section">
        <div class="section-header">
          <h3>Pipeline System</h3>
          <label class="toggle-large">
            <input type="checkbox" bind:checked={settings.enablePipeline} />
            <span class="slider-large"></span>
          </label>
        </div>
        <p class="section-description">
          Enable the complete pipeline workflow with phases and checkpoints
        </p>
        {#if settings.enablePipeline}
          <div class="warning-banner">
            ⚠️ Pipeline system is experimental. Use with caution.
          </div>
        {/if}
      </section>

      {#if settings.enablePipeline}
        <!-- Phase A: Agent Pool -->
        <section class="settings-section">
          <div class="section-header">
            <h3>P-Thread: Agent Pool</h3>
            <label class="toggle">
              <input type="checkbox" bind:checked={settings.useAgentPool} />
              <span class="slider"></span>
            </label>
          </div>
          <p class="section-description">Use shared agent pool for task execution</p>

          {#if settings.useAgentPool}
            <div class="setting-item">
              <label>
                Pool Priority
                <select bind:value={settings.poolPriority}>
                  <option value="low">Low</option>
                  <option value="normal">Normal</option>
                  <option value="high">High</option>
                </select>
              </label>
            </div>
          {/if}
        </section>

        <!-- Phase B: Orchestration -->
        <section class="settings-section">
          <div class="section-header">
            <h3>B-Thread: Task Orchestration</h3>
            <label class="toggle">
              <input type="checkbox" bind:checked={settings.enableOrchestration} />
              <span class="slider"></span>
            </label>
          </div>
          <p class="section-description">
            Automatically decompose complex tasks into workflows
          </p>

          {#if settings.enableOrchestration}
            <div class="setting-item">
              <label class="checkbox-label">
                <input type="checkbox" bind:checked={settings.autoDecompose} />
                Auto-decompose tasks
              </label>
            </div>

            <div class="setting-item">
              <label>
                Max Parallel Tasks
                <input
                  type="number"
                  bind:value={settings.maxParallelTasks}
                  min="1"
                  max="20"
                />
              </label>
            </div>
          {/if}
        </section>

        <!-- Phase C: Verification -->
        <section class="settings-section">
          <div class="section-header">
            <h3>F-Thread: Verification</h3>
            <label class="toggle">
              <input type="checkbox" bind:checked={settings.enableVerification} />
              <span class="slider"></span>
            </label>
          </div>
          <p class="section-description">
            Best-of-N verification for high-confidence results
          </p>

          {#if settings.enableVerification}
            <div class="setting-item">
              <label>
                Verification Strategy
                <select bind:value={settings.verificationStrategy}>
                  <option value="majority">Majority Vote</option>
                  <option value="weighted">Weighted Consensus</option>
                  <option value="meta">Meta-Agent Review</option>
                  <option value="first">First Correct</option>
                </select>
              </label>
            </div>

            <div class="setting-item">
              <label>
                Number of Agents (N)
                <input
                  type="number"
                  bind:value={settings.verificationN}
                  min="2"
                  max="10"
                />
              </label>
            </div>

            <div class="setting-item">
              <label>
                Confidence Threshold
                <input
                  type="range"
                  bind:value={settings.confidenceThreshold}
                  min="0.5"
                  max="1.0"
                  step="0.05"
                />
                <span class="range-value"
                  >{(settings.confidenceThreshold * 100).toFixed(0)}%</span
                >
              </label>
            </div>
          {/if}
        </section>

        <!-- Phase D: Checkpoints -->
        <section class="settings-section">
          <div class="section-header">
            <h3>C-Thread: Checkpoints</h3>
            <label class="toggle">
              <input type="checkbox" bind:checked={settings.enableCheckpoints} />
              <span class="slider"></span>
            </label>
          </div>
          <p class="section-description">Phase gates with approval and validation</p>

          {#if settings.enableCheckpoints}
            <div class="setting-item">
              <label class="checkbox-label">
                <input type="checkbox" bind:checked={settings.requirePlanReview} />
                Require plan review
              </label>
            </div>

            <div class="setting-item">
              <label class="checkbox-label">
                <input type="checkbox" bind:checked={settings.requireFinalReview} />
                Require final review
              </label>
            </div>

            <div class="setting-item">
              <label>
                Auto-validation Command
                <input
                  type="text"
                  bind:value={settings.autoValidationCommand}
                  placeholder="e.g., cargo check"
                />
              </label>
            </div>

            <div class="setting-item">
              <label class="checkbox-label">
                <input
                  type="checkbox"
                  bind:checked={settings.autoApproveOnVerification}
                />
                Auto-approve if verification passes
              </label>
            </div>
          {/if}
        </section>
      {/if}
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
