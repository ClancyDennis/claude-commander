<script lang="ts">
  import type { AgentPipelineSettings } from "../../stores/pipelineSettings";
  import HelpTip from "./HelpTip.svelte";

  // We only need a subset of AgentPipelineSettings here, but using the full type is fine as long as we pass matching object
  let { settings = $bindable(), isCreating }: { settings: Partial<AgentPipelineSettings>, isCreating: boolean } = $props();
  
  let showPipelineSettings = $state(true);
</script>

<div class="settings-section">
  <div class="label-row">
    <span class="label-text" style="margin-bottom: 0;">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="3" />
        <path
          d="M12 1v6m0 6v6M5.64 5.64l4.24 4.24m5.66 5.66l4.24 4.24M1 12h6m6 0h6M5.64 18.36l4.24-4.24m5.66-5.66l4.24-4.24"
        />
      </svg>
      Pipeline Settings
      <HelpTip text="Optional tuning for the multi-phase pipeline (agent pool, orchestration, verification, and checkpoints)." />
    </span>

    <button
      type="button"
      class="collapse-btn"
      onclick={() => showPipelineSettings = !showPipelineSettings}
      aria-expanded={showPipelineSettings}
      title={showPipelineSettings ? "Hide settings" : "Show settings"}
    >
      <svg class="chevron" class:rotated={!showPipelineSettings} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="6 9 12 15 18 9" />
      </svg>
    </button>
  </div>

  {#if showPipelineSettings}
    <div class="settings-form animate-slide-up">
      <div class="warning-banner">
        ⚠️ Pipeline system is experimental. Use with caution.
      </div>

      <!-- P-Thread: Agent Pool -->
      <div class="setting-group">
        <div class="group-header">
          <span>P-Thread: Agent Pool</span>
          <label class="toggle-small">
            <input type="checkbox" bind:checked={settings.useAgentPool} disabled={isCreating} />
            <span class="slider-small"></span>
          </label>
        </div>
        {#if settings.useAgentPool}
          <label class="setting-row">
            <span>Pool Priority <HelpTip text="Higher priority agents get resources first when the pool is constrained." placement="right" /></span>
            <select bind:value={settings.poolPriority} class="tiny-select" disabled={isCreating}>
              <option value="low">Low</option>
              <option value="normal">Normal</option>
              <option value="high">High</option>
            </select>
          </label>
        {/if}
      </div>

      <!-- B-Thread: Orchestration -->
      <div class="setting-group">
        <div class="group-header">
          <span>B-Thread: Orchestration</span>
          <label class="toggle-small">
            <input type="checkbox" bind:checked={settings.enableOrchestration} disabled={isCreating} />
            <span class="slider-small"></span>
          </label>
        </div>
        {#if settings.enableOrchestration}
          <label class="checkbox-item">
            <input type="checkbox" bind:checked={settings.autoDecompose} disabled={isCreating} />
            <span>Auto-decompose tasks <HelpTip text="Automatically break complex requests into smaller subtasks for parallel execution." placement="right" /></span>
          </label>
          <label class="setting-row">
            <span>Max Parallel Tasks <HelpTip text="Maximum subtasks that can run simultaneously. Higher = more resources used." placement="right" /></span>
            <input type="number" bind:value={settings.maxParallelTasks} min="1" max="10" class="tiny-input" disabled={isCreating} />
          </label>
        {/if}
      </div>

      <!-- F-Thread: Verification -->
      <div class="setting-group">
        <div class="group-header">
          <span>F-Thread: Verification</span>
          <label class="toggle-small">
            <input type="checkbox" bind:checked={settings.enableVerification} disabled={isCreating} />
            <span class="slider-small"></span>
          </label>
        </div>
        {#if settings.enableVerification}
          <div class="nested-settings">
            <label class="setting-row">
              <span>Strategy <HelpTip text="Majority: simple vote. Weighted: confidence-based. Meta: AI picks best. First: fastest wins." placement="right" /></span>
              <select bind:value={settings.verificationStrategy} class="tiny-select" disabled={isCreating}>
                <option value="majority">Majority</option>
                <option value="weighted">Weighted</option>
                <option value="meta">Meta</option>
                <option value="first">First</option>
              </select>
            </label>
            <label class="setting-row">
              <span>Agents (N) <HelpTip text="Number of parallel agents for verification. More agents = higher confidence but more cost." placement="right" /></span>
              <input type="number" bind:value={settings.verificationN} min="2" max="5" class="tiny-input" disabled={isCreating} />
            </label>
            <label class="setting-row">
              <span>Confidence <HelpTip text="Minimum confidence required to auto-approve. Higher = more agent agreement needed." placement="right" /></span>
              <div class="range-wrapper">
                <input type="range" bind:value={settings.confidenceThreshold} min="0.5" max="1.0" step="0.05" disabled={isCreating} />
                <span class="range-val">{((settings.confidenceThreshold || 0.8) * 100).toFixed(0)}%</span>
              </div>
            </label>
          </div>
        {/if}
      </div>

      <!-- C-Thread: Checkpoints -->
      <div class="setting-group">
        <div class="group-header">
          <span>C-Thread: Checkpoints</span>
        </div>
        <label class="checkbox-item">
          <input type="checkbox" bind:checked={settings.requirePlanReview} disabled={isCreating} />
          <span>Require plan review <HelpTip text="Pause after planning phase for human review before execution begins." placement="right" /></span>
        </label>
        <label class="checkbox-item">
          <input type="checkbox" bind:checked={settings.requireFinalReview} disabled={isCreating} />
          <span>Require final review <HelpTip text="Pause before completing to review final results." placement="right" /></span>
        </label>
        <label class="setting-row">
          <span>Auto-validate <HelpTip text="Command to run after each phase (e.g., 'cargo check'). Leave empty to skip." placement="right" /></span>
          <input type="text" bind:value={settings.autoValidationCommand} class="tiny-text-input" placeholder="cargo check" disabled={isCreating} />
        </label>
        {#if settings.enableVerification}
          <label class="checkbox-item">
            <input type="checkbox" bind:checked={settings.autoApproveOnVerification} disabled={isCreating} />
            <span>Auto-approve if verified <HelpTip text="Skip human review when verification passes with high confidence." placement="right" /></span>
          </label>
        {/if}
      </div>

    </div>
  {/if}
</div>

<style>
  .settings-section {
    margin-top: auto;
  }

  .label-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-sm);
  }

  .label-text {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .label-text svg {
    width: 18px;
    height: 18px;
    color: var(--accent);
  }

  .collapse-btn {
    width: 36px;
    height: 36px;
    padding: 0;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 10px;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .collapse-btn:hover {
    background: var(--bg-tertiary);
    border-color: var(--accent);
    color: var(--text-primary);
  }

  .chevron {
    width: 16px;
    height: 16px;
    transition: transform 0.2s ease;
  }

  .chevron.rotated {
    transform: rotate(180deg);
  }

  .settings-form {
    margin-top: var(--space-sm);
    padding: var(--space-md);
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 10px;
  }

  .setting-group {
    margin-bottom: 12px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border);
  }

  .setting-group:last-child {
    margin-bottom: 0;
    padding-bottom: 0;
    border-bottom: none;
  }

  .group-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
    font-size: 13px;
    font-weight: 600;
    color: var(--accent);
  }

  .checkbox-item {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: 4px 0;
    cursor: pointer;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .checkbox-item:hover {
    color: var(--text-primary);
  }

  .checkbox-item input[type="checkbox"] {
    margin: 0;
    width: 16px;
    height: 16px;
    cursor: pointer;
    accent-color: var(--accent);
  }

  .nested-settings {
    margin-left: 0;
    margin-top: 4px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-md);
    font-size: 13px;
    color: var(--text-secondary);
    padding: 4px 0;
  }

  .tiny-input {
    width: 60px;
    padding: 4px 8px;
    font-size: 13px;
    height: 28px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-primary);
  }

  .tiny-text-input {
    width: 120px;
    padding: 4px 8px;
    font-size: 13px;
    height: 28px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-primary);
  }

  .tiny-select {
    width: 100px;
    padding: 4px 8px;
    background-color: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-primary);
    font-size: 13px;
    height: 28px;
  }

  .warning-banner {
    padding: 8px 12px;
    background: rgba(245, 158, 11, 0.15);
    border: 1px solid var(--warning);
    border-radius: 8px;
    color: var(--warning);
    font-size: 12px;
    font-weight: 600;
    margin-bottom: 12px;
  }

  .toggle-small {
    position: relative;
    display: inline-block;
    width: 36px;
    height: 20px;
  }

  .toggle-small input {
    opacity: 0;
    width: 0;
    height: 0;
  }

  .slider-small {
    position: absolute;
    cursor: pointer;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: var(--border);
    transition: 0.3s;
    border-radius: 20px;
  }

  .slider-small:before {
    position: absolute;
    content: "";
    height: 14px;
    width: 14px;
    left: 3px;
    bottom: 3px;
    background-color: white;
    transition: 0.3s;
    border-radius: 50%;
  }

  input:checked + .slider-small {
    background-color: var(--accent);
  }

  input:checked + .slider-small:before {
    transform: translateX(16px);
  }

  .range-wrapper {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 120px;
  }

  .range-wrapper input {
    flex: 1;
    padding: 0;
    height: 4px;
  }

  .range-val {
    font-size: 11px;
    color: var(--text-muted);
    min-width: 32px;
    text-align: right;
  }
</style>
