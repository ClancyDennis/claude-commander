<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";

  interface PoolStats {
    total_agents: number;
    idle_agents: number;
    busy_agents: number;
    utilization: number;
    tasks_completed: number;
    average_task_time: number;
    by_source: Record<string, number>;  // NEW
    pooled_count: number;  // NEW
  }

  let stats = $state<PoolStats>({
    total_agents: 0,
    idle_agents: 0,
    busy_agents: 0,
    utilization: 0,
    tasks_completed: 0,
    average_task_time: 0,
    by_source: {},
    pooled_count: 0,
  });

  // Computed property for source breakdown
  let sourceBreakdown = $derived(stats.by_source || {});

  let autoRefresh = $state(true);
  let refreshInterval: number;

  async function fetchPoolStats() {
    try {
      stats = await invoke<PoolStats>("get_pool_stats");
    } catch (err) {
      console.error("Failed to fetch pool stats:", err);
    }
  }

  onMount(() => {
    fetchPoolStats();
    refreshInterval = setInterval(() => {
      if (autoRefresh) fetchPoolStats();
    }, 2000);
  });

  onDestroy(() => {
    clearInterval(refreshInterval);
  });

  function getUtilizationColor(utilization: number): string {
    if (utilization < 0.3) return "var(--success)";
    if (utilization < 0.7) return "var(--warning)";
    return "var(--error)";
  }
</script>

<div class="pool-dashboard">
  <header class="dashboard-header">
    <h2>Agent Pool</h2>
    <button class="refresh-btn" onclick={fetchPoolStats}>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M1 4v6h6M23 20v-6h-6"/>
        <path d="M20.49 9A9 9 0 005.64 5.64L1 10m22 4l-4.64 4.36A9 9 0 014.51 15"/>
      </svg>
    </button>
  </header>

  <div class="stats-grid">
    <div class="stat-card">
      <div class="stat-icon" style="background: var(--accent);">
        <svg viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2">
          <path d="M17 21v-2a4 4 0 00-4-4H5a4 4 0 00-4 4v2"/>
          <circle cx="9" cy="7" r="4"/>
          <path d="M23 21v-2a4 4 0 00-3-3.87M16 3.13a4 4 0 010 7.75"/>
        </svg>
      </div>
      <div class="stat-content">
        <div class="stat-value">{stats.total_agents}</div>
        <div class="stat-label">Total Agents</div>
      </div>
    </div>

    <div class="stat-card">
      <div class="stat-icon" style="background: var(--success);">
        <svg viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2">
          <circle cx="12" cy="12" r="10"/>
          <path d="M12 6v6l4 2"/>
        </svg>
      </div>
      <div class="stat-content">
        <div class="stat-value">{stats.idle_agents}</div>
        <div class="stat-label">Idle</div>
      </div>
    </div>

    <div class="stat-card">
      <div class="stat-icon" style="background: var(--warning);">
        <svg viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2">
          <circle cx="12" cy="12" r="10"/>
          <polyline points="12 6 12 12 16 14"/>
        </svg>
      </div>
      <div class="stat-content">
        <div class="stat-value">{stats.busy_agents}</div>
        <div class="stat-label">Busy</div>
      </div>
    </div>

    <div class="stat-card">
      <div class="stat-icon" style="background: {getUtilizationColor(stats.utilization)};">
        <svg viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2">
          <path d="M22 12h-4l-3 9L9 3l-3 9H2"/>
        </svg>
      </div>
      <div class="stat-content">
        <div class="stat-value">{(stats.utilization * 100).toFixed(0)}%</div>
        <div class="stat-label">Utilization</div>
      </div>
    </div>
  </div>

  <div class="utilization-bar">
    <div class="bar-fill"
         style="width: {stats.utilization * 100}%; background: {getUtilizationColor(stats.utilization)};">
    </div>
  </div>

  <div class="pool-metrics">
    <div class="metric">
      <span class="metric-label">Tasks Completed:</span>
      <span class="metric-value">{stats.tasks_completed}</span>
    </div>
    <div class="metric">
      <span class="metric-label">Avg Task Time:</span>
      <span class="metric-value">{stats.average_task_time.toFixed(2)}s</span>
    </div>
    <div class="metric">
      <span class="metric-label">Tracked by Pool:</span>
      <span class="metric-value">{stats.pooled_count} / {stats.total_agents}</span>
    </div>
  </div>

  {#if Object.keys(sourceBreakdown).length > 0}
    <div class="source-breakdown">
      <h3>Agents by Source</h3>
      <div class="source-grid">
        {#each Object.entries(sourceBreakdown) as [source, count]}
          <div class="source-card">
            <div class="source-icon">
              {#if source === 'ui'}
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2"/>
                  <circle cx="12" cy="7" r="4"/>
                </svg>
              {:else if source === 'meta'}
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M12 2L2 7l10 5 10-5-10-5z"/>
                  <path d="M2 17l10 5 10-5"/>
                  <path d="M2 12l10 5 10-5"/>
                </svg>
              {:else if source === 'pipeline'}
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/>
                  <path d="M22 6l-10 7L2 6"/>
                </svg>
              {:else if source === 'pool'}
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="12" r="3"/>
                  <path d="M12 1v6m0 6v6M4.93 4.93l4.24 4.24m5.66 5.66l4.24 4.24M1 12h6m6 0h6M4.93 19.07l4.24-4.24m5.66-5.66l4.24-4.24"/>
                </svg>
              {:else}
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="12" r="10"/>
                  <path d="M12 8v8m-4-4h8"/>
                </svg>
              {/if}
            </div>
            <div class="source-info">
              <div class="source-name">{source.toUpperCase()}</div>
              <div class="source-count">{count}</div>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .pool-dashboard {
    padding: var(--space-lg);
    background: var(--bg-secondary);
    border-radius: 12px;
    border: 1px solid var(--border);
  }

  .dashboard-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-lg);
  }

  .dashboard-header h2 {
    margin: 0;
    font-size: 24px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .refresh-btn {
    width: 36px;
    height: 36px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--bg-tertiary);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .refresh-btn:hover {
    background: var(--accent-glow);
    border-color: var(--accent);
  }

  .refresh-btn svg {
    width: 18px;
    height: 18px;
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: var(--space-md);
    margin-bottom: var(--space-lg);
  }

  .stat-card {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-md);
    background: var(--bg-tertiary);
    border-radius: 10px;
    border: 1px solid var(--border);
  }

  .stat-icon {
    width: 48px;
    height: 48px;
    border-radius: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .stat-icon svg {
    width: 24px;
    height: 24px;
  }

  .stat-content {
    flex: 1;
  }

  .stat-value {
    font-size: 28px;
    font-weight: 700;
    color: var(--text-primary);
    line-height: 1;
  }

  .stat-label {
    font-size: 13px;
    color: var(--text-muted);
    margin-top: 4px;
  }

  .utilization-bar {
    height: 8px;
    background: var(--bg-tertiary);
    border-radius: 4px;
    overflow: hidden;
    margin-bottom: var(--space-lg);
  }

  .bar-fill {
    height: 100%;
    transition: width 0.3s ease, background 0.3s ease;
  }

  .pool-metrics {
    display: flex;
    gap: var(--space-xl);
  }

  .metric {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .metric-label {
    font-size: 14px;
    color: var(--text-muted);
  }

  .metric-value {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .source-breakdown {
    margin-top: var(--space-lg);
    padding-top: var(--space-lg);
    border-top: 1px solid var(--border);
  }

  .source-breakdown h3 {
    margin: 0 0 var(--space-md) 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .source-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: var(--space-sm);
  }

  .source-card {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-sm) var(--space-md);
    background: var(--bg-tertiary);
    border-radius: 8px;
    border: 1px solid var(--border);
  }

  .source-icon {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    color: var(--accent);
  }

  .source-icon svg {
    width: 20px;
    height: 20px;
  }

  .source-info {
    flex: 1;
    min-width: 0;
  }

  .source-name {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .source-count {
    font-size: 18px;
    font-weight: 700;
    color: var(--text-primary);
  }
</style>
