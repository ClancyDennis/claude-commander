<script lang="ts">
  import type { ToolCallStatistics } from "../../types";
  import { formatTimeDuration } from "$lib/utils/formatting";

  let { stats }: { stats: ToolCallStatistics } = $props();

  const successRate = $derived(
    stats.totalCalls > 0
      ? ((stats.successfulCalls / stats.totalCalls) * 100).toFixed(0)
      : "0"
  );
</script>

<section class="stats-section">
  <div class="stats-row">
    <div class="stat-info">
      <svg class="stat-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <polyline points="20 6 9 17 4 12"/>
      </svg>
      <span class="stat-label">Success rate</span>
    </div>
    <span class="stat-value success">{successRate}%</span>
  </div>
  <div class="stats-row">
    <div class="stat-info">
      <svg class="stat-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <circle cx="12" cy="12" r="10"/>
        <polyline points="12 6 12 12 16 14"/>
      </svg>
      <span class="stat-label">Avg time</span>
    </div>
    <span class="stat-value">{formatTimeDuration(stats.averageExecutionTimeMs)}</span>
  </div>
  <div class="stats-row">
    <div class="stat-info">
      <svg class="stat-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <circle cx="12" cy="12" r="10"/>
        <line x1="15" y1="9" x2="9" y2="15"/>
        <line x1="9" y1="9" x2="15" y2="15"/>
      </svg>
      <span class="stat-label">Failed</span>
    </div>
    <span class="stat-value {stats.failedCalls > 0 ? 'error' : ''}">{stats.failedCalls}</span>
  </div>
</section>

<style>
  .stats-section {
    padding: var(--space-3) var(--space-5);
    border-bottom: 1px solid var(--border-hex);
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .stats-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-2) var(--space-3);
    margin: 0 calc(var(--space-3) * -1);
    border-radius: var(--radius-sm);
    transition: background var(--transition-fast);
  }

  .stats-row:hover {
    background: var(--bg-tertiary);
  }

  .stat-info {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .stat-icon {
    width: 16px;
    height: 16px;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .stat-label {
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }

  .stat-value {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--text-primary);
    font-variant-numeric: tabular-nums;
  }

  .stat-value.success {
    color: var(--success-hex);
  }

  .stat-value.error {
    color: var(--error);
  }
</style>
