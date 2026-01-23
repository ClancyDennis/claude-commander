<script lang="ts">
  import type { AgentPipelineSettings } from "../../stores/pipelineSettings";
  import HelpTip from "./HelpTip.svelte";
  import SettingGroup from "./settings/SettingGroup.svelte";
  import SettingRow from "./settings/SettingRow.svelte";
  import CheckboxItem from "./settings/CheckboxItem.svelte";

  let { settings = $bindable(), isCreating }: { settings: Partial<AgentPipelineSettings>, isCreating: boolean } = $props();

  let showPipelineSettings = $state(true);
</script>

<div class="settings-section">
  <div class="label-row">
    <span class="label-text" style="margin-bottom: 0;">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="3" />
        <path d="M12 1v6m0 6v6M5.64 5.64l4.24 4.24m5.66 5.66l4.24 4.24M1 12h6m6 0h6M5.64 18.36l4.24-4.24m5.66-5.66l4.24-4.24" />
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
        Pipeline system is experimental. Use with caution.
      </div>

      <!-- P-Thread: Agent Pool -->
      <SettingGroup title="P-Thread: Agent Pool" hasToggle bind:enabled={settings.useAgentPool} {isCreating}>
        <SettingRow label="Pool Priority" helpText="Higher priority agents get resources first when the pool is constrained.">
          <select bind:value={settings.poolPriority} class="tiny-select" disabled={isCreating}>
            <option value="low">Low</option>
            <option value="normal">Normal</option>
            <option value="high">High</option>
          </select>
        </SettingRow>
      </SettingGroup>

      <!-- B-Thread: Orchestration -->
      <SettingGroup title="B-Thread: Orchestration" hasToggle bind:enabled={settings.enableOrchestration} {isCreating}>
        <CheckboxItem
          label="Auto-decompose tasks"
          bind:checked={settings.autoDecompose}
          disabled={isCreating}
          helpText="Automatically break complex requests into smaller subtasks for parallel execution."
        />
        <SettingRow label="Max Parallel Tasks" helpText="Maximum subtasks that can run simultaneously. Higher = more resources used.">
          <input type="number" bind:value={settings.maxParallelTasks} min="1" max="10" class="tiny-input" disabled={isCreating} />
        </SettingRow>
      </SettingGroup>

      <!-- F-Thread: Verification -->
      <SettingGroup title="F-Thread: Verification" hasToggle bind:enabled={settings.enableVerification} {isCreating}>
        <div class="nested-settings">
          <SettingRow label="Strategy" helpText="Majority: simple vote. Weighted: confidence-based. Meta: AI picks best. First: fastest wins.">
            <select bind:value={settings.verificationStrategy} class="tiny-select" disabled={isCreating}>
              <option value="majority">Majority</option>
              <option value="weighted">Weighted</option>
              <option value="meta">Meta</option>
              <option value="first">First</option>
            </select>
          </SettingRow>
          <SettingRow label="Agents (N)" helpText="Number of parallel agents for verification. More agents = higher confidence but more cost.">
            <input type="number" bind:value={settings.verificationN} min="2" max="5" class="tiny-input" disabled={isCreating} />
          </SettingRow>
          <SettingRow label="Confidence" helpText="Minimum confidence required to auto-approve. Higher = more agent agreement needed.">
            <div class="range-wrapper">
              <input type="range" bind:value={settings.confidenceThreshold} min="0.5" max="1.0" step="0.05" disabled={isCreating} />
              <span class="range-val">{((settings.confidenceThreshold || 0.8) * 100).toFixed(0)}%</span>
            </div>
          </SettingRow>
        </div>
      </SettingGroup>

      <!-- C-Thread: Checkpoints -->
      <SettingGroup title="C-Thread: Checkpoints">
        <CheckboxItem
          label="Require plan review"
          bind:checked={settings.requirePlanReview}
          disabled={isCreating}
          helpText="Pause after planning phase for human review before execution begins."
        />
        <CheckboxItem
          label="Require final review"
          bind:checked={settings.requireFinalReview}
          disabled={isCreating}
          helpText="Pause before completing to review final results."
        />
        <SettingRow label="Auto-validate" helpText="Command to run after each phase (e.g., 'cargo check'). Leave empty to skip.">
          <input type="text" bind:value={settings.autoValidationCommand} class="tiny-text-input" placeholder="cargo check" disabled={isCreating} />
        </SettingRow>
        {#if settings.enableVerification}
          <CheckboxItem
            label="Auto-approve if verified"
            bind:checked={settings.autoApproveOnVerification}
            disabled={isCreating}
            helpText="Skip human review when verification passes with high confidence."
          />
        {/if}
      </SettingGroup>
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

  .nested-settings {
    margin-left: 0;
    margin-top: 4px;
    display: flex;
    flex-direction: column;
    gap: 8px;
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
