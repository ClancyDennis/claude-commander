<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import HelpTip from "./new-agent/HelpTip.svelte";

  interface DatabaseStats {
    db_size_bytes: number;
    db_size_formatted: string;
    total_runs: number;
    total_prompts: number;
    runs_by_status: [string, number][];
    runs_by_source: [string, number][];
    total_cost_usd: number;
  }

  let stats = $state<DatabaseStats | null>(null);
  let loading = $state(true);
  let autoRefresh = $state(true);
  let refreshInterval: number;

  async function fetchDatabaseStats() {
    try {
      loading = true;
      stats = await invoke<DatabaseStats>("get_database_stats");
    } catch (err) {
      console.error("Failed to fetch database stats:", err);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    fetchDatabaseStats();
    refreshInterval = setInterval(() => {
      if (autoRefresh) fetchDatabaseStats();
    }, 5000); // Refresh every 5 seconds
  });

  onDestroy(() => {
    clearInterval(refreshInterval);
  });

  function formatNumber(num: number): string {
    return new Intl.NumberFormat().format(num);
  }

  function formatCost(cost: number): string {
    return `$${cost.toFixed(4)}`;
  }
</script>

<div class="database-stats">
  <header class="stats-header">
    <h2>Database Statistics</h2>
    <button class="refresh-btn" onclick={fetchDatabaseStats} disabled={loading} aria-label="Refresh statistics">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M1 4v6h6M23 20v-6h-6"/>
        <path d="M20.49 9A9 9 0 005.64 5.64L1 10m22 4l-4.64 4.36A9 9 0 014.51 15"/>
      </svg>
    </button>
  </header>

  {#if loading && !stats}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Loading database statistics...</p>
    </div>
  {:else if stats}
    <div class="stats-grid">
      <div class="stat-card primary">
        <div class="stat-icon" style="background: var(--info);">
          <svg viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2">
            <path d="M21 16V8a2 2 0 00-1-1.73l-7-4a2 2 0 00-2 0l-7 4A2 2 0 003 8v8a2 2 0 001 1.73l7 4a2 2 0 002 0l7-4A2 2 0 0021 16z"/>
            <polyline points="3.27 6.96 12 12.01 20.73 6.96"/>
            <line x1="12" y1="22.08" x2="12" y2="12"/>
          </svg>
        </div>
        <div class="stat-content">
          <div class="stat-value">{stats.db_size_formatted}</div>
          <div class="stat-label">Database Size <HelpTip text="Total size of the SQLite database storing agent runs and outputs." placement="right" /></div>
          <div class="stat-meta">{formatNumber(stats.db_size_bytes)} bytes</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon" style="background: var(--accent);">
          <svg viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2">
            <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/>
            <polyline points="7 10 12 15 17 10"/>
            <line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
        </div>
        <div class="stat-content">
          <div class="stat-value">{formatNumber(stats.total_runs)}</div>
          <div class="stat-label">Total Runs <HelpTip text="Number of agent sessions stored, including completed and failed runs." placement="right" /></div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon" style="background: var(--success);">
          <svg viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2">
            <path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z"/>
          </svg>
        </div>
        <div class="stat-content">
          <div class="stat-value">{formatNumber(stats.total_prompts)}</div>
          <div class="stat-label">Total Prompts</div>
        </div>
      </div>
    </div>

    <div class="breakdown-section">
      {#if stats.runs_by_status.length > 0}
        <div class="breakdown-column">
          <h3>Runs by Status</h3>
          <div class="breakdown-list">
            {#each stats.runs_by_status as [status, count]}
              <div class="breakdown-item">
                <span class="status-badge {status}">{status}</span>
                <span class="breakdown-value">{formatNumber(count)}</span>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      {#if stats.runs_by_source.length > 0}
        <div class="breakdown-column">
          <h3>Runs by Source</h3>
          <div class="breakdown-list">
            {#each stats.runs_by_source as [source, count]}
              <div class="breakdown-item">
                <span class="breakdown-label">{source.toUpperCase()}</span>
                <span class="breakdown-value">{formatNumber(count)}</span>
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .database-stats {
    padding: var(--space-lg);
    background: var(--bg-secondary);
    border-radius: 12px;
    border: 1px solid var(--border);
  }

  .stats-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-lg);
  }

  .stats-header h2 {
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

  .refresh-btn:hover:not(:disabled) {
    background: var(--accent-glow);
    border-color: var(--accent);
  }

  .refresh-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .refresh-btn svg {
    width: 18px;
    height: 18px;
  }

  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-xl);
    gap: var(--space-md);
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
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

  .stat-card.primary {
    grid-column: 1 / -1;
    background: linear-gradient(135deg, rgba(74, 158, 255, 0.1) 0%, rgba(155, 89, 182, 0.1) 100%);
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
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .stat-meta {
    font-size: 11px;
    color: var(--text-muted);
    margin-top: 2px;
  }

  .breakdown-section {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: var(--space-lg);
  }

  .breakdown-column h3 {
    margin: 0 0 var(--space-md) 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .breakdown-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .breakdown-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-sm) var(--space-md);
    background: var(--bg-tertiary);
    border-radius: 8px;
    border: 1px solid var(--border);
  }

  .breakdown-label {
    color: var(--text-secondary);
    font-size: 13px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .breakdown-value {
    color: var(--text-primary);
    font-weight: 600;
    font-size: 14px;
  }

  .status-badge {
    padding: 4px 8px;
    border-radius: 6px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .status-badge.running {
    background: rgba(74, 158, 255, 0.2);
    color: var(--info);
  }

  .status-badge.completed {
    background: rgba(46, 213, 115, 0.2);
    color: var(--success);
  }

  .status-badge.stopped {
    background: rgba(149, 165, 166, 0.2);
    color: var(--text-muted);
  }

  .status-badge.crashed {
    background: rgba(231, 76, 60, 0.2);
    color: var(--error);
  }

  .status-badge.waiting_input {
    background: rgba(255, 193, 7, 0.2);
    color: var(--warning);
  }
</style>
