<script lang="ts">
  import type { AgentRun } from "$lib/types";
  import { formatTimeAbsolute, formatDurationVerbose, formatBytes, formatCost } from '$lib/utils/formatting';

  interface Props {
    run: AgentRun;
  }

  let { run }: Props = $props();
</script>

<div class="stats-summary">
  <div class="stat-card">
    <div class="stat-label">Started</div>
    <div class="stat-value">{formatTimeAbsolute(run.started_at)}</div>
  </div>
  <div class="stat-card">
    <div class="stat-label">Duration</div>
    <div class="stat-value">
      {formatDurationVerbose(run.started_at, run.ended_at)}
    </div>
  </div>
  <div class="stat-card">
    <div class="stat-label">Prompts</div>
    <div class="stat-value">{run.total_prompts}</div>
  </div>
  <div class="stat-card">
    <div class="stat-label">Tool Calls</div>
    <div class="stat-value">{run.total_tool_calls}</div>
  </div>
  <div class="stat-card">
    <div class="stat-label">Output Size</div>
    <div class="stat-value">{formatBytes(run.total_output_bytes)}</div>
  </div>
  {#if run.total_cost_usd}
    <div class="stat-card">
      <div class="stat-label">Cost</div>
      <div class="stat-value">{formatCost(run.total_cost_usd)}</div>
    </div>
  {/if}
</div>

<style>
  .stats-summary {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
    gap: var(--space-2);
    padding: var(--space-4);
    border-bottom: 1px solid var(--border-hex);
    flex-shrink: 0;
  }

  .stat-card {
    background-color: var(--bg-tertiary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-md);
    padding: var(--space-3);
  }

  .stat-label {
    font-size: var(--text-xs);
    color: var(--text-muted);
    font-weight: var(--font-medium);
    margin-bottom: 2px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .stat-value {
    font-size: var(--text-base);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
  }
</style>
