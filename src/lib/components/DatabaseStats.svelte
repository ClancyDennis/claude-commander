<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import HelpTip from "./new-agent/HelpTip.svelte";
  import { formatCost } from '$lib/utils/formatting';
  import { useAsyncData } from '$lib/hooks/useAsyncData.svelte';

  interface DatabaseStats {
    db_size_bytes: number;
    db_size_formatted: string;
    total_runs: number;
    total_prompts: number;
    runs_by_status: [string, number][];
    runs_by_source: [string, number][];
    total_cost_usd: number;
  }

  interface CostMetrics {
    today: number;
    last24Hours: number;
    thisWeek: number;
    thisMonth: number;
    allTime: number;
  }

  let autoRefresh = $state(true);
  let refreshInterval: number;

  const asyncStats = useAsyncData(() => invoke<DatabaseStats>("get_database_stats"));

  const asyncCostMetrics = useAsyncData(async () => {
    const now = new Date();
    const twentyFourHoursAgo = new Date(now.getTime() - 24 * 60 * 60 * 1000);
    const weekAgo = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000);

    const [todayCost, monthCost, summary, last24HoursData, weekData] = await Promise.all([
      invoke<number>('get_today_cost'),
      invoke<number>('get_current_month_cost'),
      invoke<{ total_cost_usd: number }>('get_cost_summary'),
      invoke<{ total_cost_usd: number }>('get_cost_by_date_range', {
        startDate: twentyFourHoursAgo.toISOString(),
        endDate: null
      }),
      invoke<{ total_cost_usd: number }>('get_cost_by_date_range', {
        startDate: weekAgo.toISOString(),
        endDate: null
      })
    ]);

    return {
      today: todayCost,
      last24Hours: last24HoursData.total_cost_usd,
      thisWeek: weekData.total_cost_usd,
      thisMonth: monthCost,
      allTime: summary.total_cost_usd
    };
  });

  // Derived for combined loading state
  const loading = $derived(asyncStats.loading && !asyncStats.data);

  onMount(() => {
    refreshAll();
    refreshInterval = setInterval(() => {
      if (autoRefresh) refreshAll();
    }, 5000);
  });

  onDestroy(() => {
    clearInterval(refreshInterval);
  });

  function formatNumber(num: number): string {
    return new Intl.NumberFormat().format(num);
  }

  function formatStatus(status: string): string {
    if (status === 'crashed') return 'ended';
    if (status === 'waiting_input') return 'waiting';
    return status;
  }

  async function refreshAll() {
    await Promise.all([asyncStats.fetch(), asyncCostMetrics.fetch()]);
  }
</script>

<div class="database-stats">
  <header class="stats-header">
    <h2>Database Statistics</h2>
    <button class="refresh-btn" onclick={refreshAll} disabled={loading} aria-label="Refresh statistics">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M1 4v6h6M23 20v-6h-6"/>
        <path d="M20.49 9A9 9 0 005.64 5.64L1 10m22 4l-4.64 4.36A9 9 0 014.51 15"/>
      </svg>
    </button>
  </header>

  {#if loading && !asyncStats.data}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Loading statistics...</p>
    </div>
  {:else if asyncStats.data}
    <!-- Storage Section -->
    <section class="section">
      <h3 class="section-title">Storage</h3>
      <div class="list-group">
        <div class="list-row">
          <span class="list-label">Database Size <HelpTip text="Total size of the SQLite database." placement="right" /></span>
          <span class="list-value">{asyncStats.data.db_size_formatted}</span>
        </div>
        <div class="list-row">
          <span class="list-label">Size in Bytes</span>
          <span class="list-value secondary">{formatNumber(asyncStats.data.db_size_bytes)}</span>
        </div>
      </div>
    </section>

    <!-- Usage Section -->
    <section class="section">
      <h3 class="section-title">Usage</h3>
      <div class="list-group">
        <div class="list-row">
          <span class="list-label">Total Runs <HelpTip text="Number of agent sessions stored." placement="right" /></span>
          <span class="list-value">{formatNumber(asyncStats.data.total_runs)}</span>
        </div>
        <div class="list-row">
          <span class="list-label">Total Prompts</span>
          <span class="list-value">{formatNumber(asyncStats.data.total_prompts)}</span>
        </div>
      </div>
    </section>

    <!-- Cost Section -->
    {#if asyncCostMetrics.data}
      <section class="section">
        <h3 class="section-title">Cost</h3>
        <div class="list-group">
          <div class="list-row featured">
            <span class="list-label">Total Spend <HelpTip text="Total cost across all agent sessions." placement="right" /></span>
            <span class="list-value featured-value">{formatCost(asyncCostMetrics.data.allTime)}</span>
          </div>
        </div>
        <div class="list-group">
          <div class="list-row">
            <span class="list-label">Last 24 Hours</span>
            <span class="list-value">{formatCost(asyncCostMetrics.data.last24Hours)}</span>
          </div>
          <div class="list-row">
            <span class="list-label">Today</span>
            <span class="list-value">{formatCost(asyncCostMetrics.data.today)}</span>
          </div>
          <div class="list-row">
            <span class="list-label">This Week</span>
            <span class="list-value">{formatCost(asyncCostMetrics.data.thisWeek)}</span>
          </div>
          <div class="list-row">
            <span class="list-label">This Month</span>
            <span class="list-value">{formatCost(asyncCostMetrics.data.thisMonth)}</span>
          </div>
        </div>
      </section>
    {/if}

    <!-- Runs by Status -->
    {#if asyncStats.data.runs_by_status.length > 0}
      <section class="section">
        <h3 class="section-title">Runs by Status</h3>
        <div class="list-group">
          {#each asyncStats.data.runs_by_status as [status, count]}
            <div class="list-row">
              <span class="status-indicator {status}">{formatStatus(status)}</span>
              <span class="list-value">{formatNumber(count)}</span>
            </div>
          {/each}
        </div>
      </section>
    {/if}

    <!-- Runs by Source -->
    {#if asyncStats.data.runs_by_source.length > 0}
      <section class="section">
        <h3 class="section-title">Runs by Source</h3>
        <div class="list-group">
          {#each asyncStats.data.runs_by_source as [source, count]}
            <div class="list-row">
              <span class="list-label">{source}</span>
              <span class="list-value">{formatNumber(count)}</span>
            </div>
          {/each}
        </div>
      </section>
    {/if}
  {/if}
</div>

<style>
  .database-stats {
    padding: var(--space-4);
    background: var(--bg-secondary);
    border-radius: var(--radius-lg);
    border: 1px solid var(--border-hex);
    max-width: 100%;
    overflow: hidden;
  }

  .stats-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-5);
    padding-bottom: var(--space-3);
    border-bottom: 1px solid var(--border-hex);
  }

  .stats-header h2 {
    margin: 0;
    font-size: var(--text-lg);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
  }

  .refresh-btn {
    width: 28px;
    height: 28px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-hex);
    background: var(--bg-tertiary);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all var(--transition-fast);
    color: var(--text-muted);
  }

  .refresh-btn:hover:not(:disabled) {
    background: var(--accent-glow);
    border-color: var(--accent-hex);
    color: var(--accent-hex);
  }

  .refresh-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .refresh-btn svg {
    width: 14px;
    height: 14px;
  }

  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-6);
    gap: var(--space-3);
  }

  .loading-state p {
    font-size: var(--text-sm);
    color: var(--text-muted);
    margin: 0;
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 2px solid var(--border-hex);
    border-top-color: var(--accent-hex);
    border-radius: var(--radius-full);
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  /* Sections */
  .section {
    margin-bottom: var(--space-4);
  }

  .section:last-child {
    margin-bottom: 0;
  }

  .section-title {
    font-size: var(--text-xs);
    font-weight: var(--font-semibold);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin: 0 0 var(--space-2) var(--space-1);
  }

  /* List Groups */
  .list-group {
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-hex);
    overflow: hidden;
    margin-bottom: var(--space-2);
  }

  .list-group:last-child {
    margin-bottom: 0;
  }

  .list-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-2) var(--space-3);
    border-bottom: 1px solid var(--border-hex);
    min-height: 36px;
  }

  .list-row:last-child {
    border-bottom: none;
  }

  .list-row.featured {
    background: linear-gradient(135deg, rgba(74, 158, 255, 0.06) 0%, rgba(155, 89, 182, 0.06) 100%);
  }

  .list-label {
    font-size: var(--text-sm);
    color: var(--text-primary);
    display: flex;
    align-items: center;
    gap: var(--space-1);
    flex-shrink: 0;
  }

  .list-value {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--text-primary);
    text-align: right;
    font-variant-numeric: tabular-nums;
    flex-shrink: 0;
  }

  .list-value.secondary {
    color: var(--text-muted);
    font-weight: normal;
  }

  .list-value.featured-value {
    font-weight: var(--font-semibold);
    color: var(--accent-hex);
  }

  /* Status indicators */
  .status-indicator {
    font-size: var(--text-xs);
    font-weight: var(--font-medium);
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    text-transform: capitalize;
  }

  .status-indicator.running {
    background: rgba(0, 122, 255, 0.15);
    color: hsl(var(--system-blue));
  }

  .status-indicator.completed {
    background: var(--success-glow);
    color: var(--success-hex);
  }

  .status-indicator.stopped {
    background: rgba(128, 128, 128, 0.15);
    color: var(--text-muted);
  }

  .status-indicator.crashed,
  .status-indicator.ended {
    background: rgba(128, 128, 128, 0.15);
    color: var(--text-muted);
  }

  .status-indicator.waiting_input {
    background: var(--warning-glow);
    color: var(--warning-hex);
  }
</style>
