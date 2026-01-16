<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import {
    costTrackingStore,
    refreshCostSummary,
    clearCostHistory,
    formatCost,
    formatNumber,
    getCostByModelArray,
    getCostByWorkingDirArray,
    getCostsByDateRange,
    startAutoRefresh,
    stopAutoRefresh,
  } from '../stores/costTracking';
  import type { DateRangeCostSummary } from '../types';

  let activeTab: 'overview' | 'sessions' | 'charts' = 'overview';
  let dateRange: DateRangeCostSummary | null = null;
  let dateRangeType: 'all' | 'today' | 'week' | 'month' | 'custom' = 'all';
  let customStartDate = '';
  let customEndDate = '';
  let showClearConfirm = false;

  $: summary = $costTrackingStore.summary;
  $: loading = $costTrackingStore.loading;
  $: todayCost = $costTrackingStore.todayCost;
  $: monthCost = $costTrackingStore.monthCost;
  $: modelCosts = getCostByModelArray(summary);
  $: dirCosts = getCostByWorkingDirArray(summary);

  onMount(() => {
    refreshCostSummary();
    startAutoRefresh(30000); // Refresh every 30 seconds
  });

  onDestroy(() => {
    stopAutoRefresh();
  });

  async function handleDateRangeChange() {
    let startDate: string | undefined;
    let endDate: string | undefined;

    const now = new Date();

    switch (dateRangeType) {
      case 'today':
        startDate = new Date(now.getFullYear(), now.getMonth(), now.getDate()).toISOString();
        endDate = now.toISOString();
        break;
      case 'week':
        const weekAgo = new Date(now);
        weekAgo.setDate(weekAgo.getDate() - 7);
        startDate = weekAgo.toISOString();
        endDate = now.toISOString();
        break;
      case 'month':
        const monthAgo = new Date(now);
        monthAgo.setMonth(monthAgo.getMonth() - 1);
        startDate = monthAgo.toISOString();
        endDate = now.toISOString();
        break;
      case 'custom':
        if (customStartDate) {
          startDate = new Date(customStartDate).toISOString();
        }
        if (customEndDate) {
          endDate = new Date(customEndDate).toISOString();
        }
        break;
      case 'all':
      default:
        // No date range
        break;
    }

    dateRange = await getCostsByDateRange(startDate, endDate);
  }

  async function handleClearHistory() {
    const success = await clearCostHistory();
    if (success) {
      showClearConfirm = false;
      dateRange = null;
    }
  }

  function formatDate(dateStr: string): string {
    try {
      return new Date(dateStr).toLocaleDateString();
    } catch (e) {
      console.error('Error formatting date:', e);
      return dateStr;
    }
  }

  function formatDateTime(dateStr: string): string {
    try {
      const date = new Date(dateStr);
      if (isNaN(date.getTime())) {
        return dateStr;
      }
      return date.toLocaleString();
    } catch (e) {
      console.error('Error formatting datetime:', e);
      return dateStr;
    }
  }

  $: if (dateRangeType !== 'custom') {
    handleDateRangeChange();
  }
</script>

<div class="cost-tracker">
  <div class="cost-header">
    <h2>💰 Cost Tracking</h2>
    <div class="header-actions">
      <button class="btn-refresh" on:click={refreshCostSummary} disabled={loading}>
        {loading ? '🔄 Loading...' : '🔄 Refresh'}
      </button>
      <button class="btn-clear" on:click={() => (showClearConfirm = true)}>
        🗑️ Clear History
      </button>
    </div>
  </div>

  {#if showClearConfirm}
    <div class="confirm-dialog">
      <div class="confirm-content">
        <h3>⚠️ Clear Cost History?</h3>
        <p>This will permanently delete all cost tracking data. This action cannot be undone.</p>
        <div class="confirm-actions">
          <button class="btn-cancel" on:click={() => (showClearConfirm = false)}>Cancel</button>
          <button class="btn-confirm-delete" on:click={handleClearHistory}>Clear All Data</button>
        </div>
      </div>
    </div>
  {/if}

  <div class="tabs">
    <button class="tab" class:active={activeTab === 'overview'} on:click={() => (activeTab = 'overview')}>
      Overview
    </button>
    <button class="tab" class:active={activeTab === 'sessions'} on:click={() => (activeTab = 'sessions')}>
      Sessions ({summary?.totalSessions ?? 0})
    </button>
    <button class="tab" class:active={activeTab === 'charts'} on:click={() => (activeTab = 'charts')}>
      Analytics
    </button>
  </div>

  <div class="tab-content">
    {#if activeTab === 'overview'}
      <div class="overview">
        <div class="cost-cards">
          <div class="cost-card total">
            <div class="card-icon">💵</div>
            <div class="card-content">
              <div class="card-label">Total Cost (All Time)</div>
              <div class="card-value">{formatCost(summary?.totalCostUsd ?? 0)}</div>
              <div class="card-meta">{summary?.totalSessions ?? 0} sessions</div>
            </div>
          </div>

          <div class="cost-card today">
            <div class="card-icon">📅</div>
            <div class="card-content">
              <div class="card-label">Today</div>
              <div class="card-value">{formatCost(todayCost)}</div>
            </div>
          </div>

          <div class="cost-card month">
            <div class="card-icon">📆</div>
            <div class="card-content">
              <div class="card-label">This Month</div>
              <div class="card-value">{formatCost(monthCost)}</div>
            </div>
          </div>

          <div class="cost-card tokens">
            <div class="card-icon">🔤</div>
            <div class="card-content">
              <div class="card-label">Total Tokens</div>
              <div class="card-value">{formatNumber(summary?.totalTokens ?? 0)}</div>
            </div>
          </div>
        </div>

        <div class="breakdown-section">
          <div class="breakdown-column">
            <h3>Cost by Model</h3>
            {#if modelCosts.length > 0}
              <div class="breakdown-list">
                {#each modelCosts as { model, cost }}
                  <div class="breakdown-item">
                    <span class="breakdown-label">{model}</span>
                    <span class="breakdown-value">{formatCost(cost)}</span>
                  </div>
                {/each}
              </div>
            {:else}
              <div class="empty-state">No model data available</div>
            {/if}
          </div>

          <div class="breakdown-column">
            <h3>Cost by Project</h3>
            {#if dirCosts.length > 0}
              <div class="breakdown-list">
                {#each dirCosts.slice(0, 5) as { dir, cost, fullPath }}
                  <div class="breakdown-item" title={fullPath}>
                    <span class="breakdown-label">{dir}</span>
                    <span class="breakdown-value">{formatCost(cost)}</span>
                  </div>
                {/each}
              </div>
            {:else}
              <div class="empty-state">No project data available</div>
            {/if}
          </div>
        </div>
      </div>
    {:else if activeTab === 'sessions'}
      <div class="sessions">
        <div class="date-range-filter">
          <label>
            <span>Filter:</span>
            <select bind:value={dateRangeType} on:change={handleDateRangeChange}>
              <option value="all">All Time</option>
              <option value="today">Today</option>
              <option value="week">Last 7 Days</option>
              <option value="month">Last 30 Days</option>
              <option value="custom">Custom Range</option>
            </select>
          </label>

          {#if dateRangeType === 'custom'}
            <div class="custom-range">
              <input type="date" bind:value={customStartDate} />
              <span>to</span>
              <input type="date" bind:value={customEndDate} />
              <button class="btn-apply" on:click={handleDateRangeChange}>Apply</button>
            </div>
          {/if}

          {#if dateRange}
            <div class="range-summary">
              <strong>{formatCost(dateRange.totalCostUsd)}</strong> across {dateRange.sessionCount} sessions
            </div>
          {/if}
        </div>

        {#if summary?.sessionRecords && Array.isArray(summary.sessionRecords) && summary.sessionRecords.length > 0}
          <div class="session-list">
            {#each summary.sessionRecords.slice().reverse() as session}
              {#if session && session.startedAt && session.workingDir}
                <div class="session-card">
                  <div class="session-header">
                    <div class="session-time">{formatDateTime(session.startedAt)}</div>
                    <div class="session-cost">{formatCost(session.totalCostUsd ?? 0)}</div>
                  </div>
                  <div class="session-details">
                    <div class="session-detail">
                      <span class="detail-label">Project:</span>
                      <span class="detail-value" title={session.workingDir}>
                        {session.workingDir.split('/').pop() || session.workingDir}
                      </span>
                    </div>
                    <div class="session-stats">
                      <span>🔤 {formatNumber(session.totalTokens ?? 0)} tokens</span>
                      <span>💬 {session.totalPrompts ?? 0} prompts</span>
                      <span>🔧 {session.totalToolCalls ?? 0} tool calls</span>
                    </div>
                  </div>
                  {#if session.modelUsage && typeof session.modelUsage === 'object'}
                    <div class="session-models">
                      {#each Object.entries(session.modelUsage) as [model, usage]}
                        {#if usage && typeof usage.costUsd === 'number'}
                          <div class="model-usage">
                            <span class="model-name">{model}</span>
                            <span class="model-cost">{formatCost(usage.costUsd)}</span>
                          </div>
                        {/if}
                      {/each}
                    </div>
                  {/if}
                </div>
              {/if}
            {/each}
          </div>
        {:else}
          <div class="empty-state-large">
            <div class="empty-icon">📊</div>
            <h3>No Sessions Yet</h3>
            <p>Cost data will appear here after agents complete their work.</p>
          </div>
        {/if}
      </div>
    {:else if activeTab === 'charts'}
      <div class="charts">
        {#if dateRange && dateRange.dailyCosts.length > 0}
          <div class="chart-section">
            <h3>Daily Costs</h3>
            <div class="daily-chart">
              {#each dateRange.dailyCosts as day}
                {@const maxCost = Math.max(...dateRange.dailyCosts.map((d) => d.costUsd))}
                {@const height = maxCost > 0 ? (day.costUsd / maxCost) * 100 : 0}
                <div class="chart-bar-wrapper">
                  <div class="chart-bar" style="height: {height}%" title="{formatDate(day.date)}: {formatCost(day.costUsd)}">
                    <div class="bar-label">{formatCost(day.costUsd)}</div>
                  </div>
                  <div class="chart-label">{formatDate(day.date).split('/').slice(0, 2).join('/')}</div>
                </div>
              {/each}
            </div>
          </div>
        {:else}
          <div class="empty-state-large">
            <div class="empty-icon">📈</div>
            <h3>No Chart Data</h3>
            <p>Select a date range to see cost trends over time.</p>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .cost-tracker {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-secondary, #1e1e1e);
    border-radius: 8px;
    overflow: hidden;
  }

  .cost-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 1.5rem;
    background: var(--bg-primary, #252525);
    border-bottom: 1px solid var(--border-color, #333);
  }

  .cost-header h2 {
    margin: 0;
    font-size: 1.25rem;
    color: var(--text-primary, #fff);
  }

  .header-actions {
    display: flex;
    gap: 0.5rem;
  }

  .btn-refresh,
  .btn-clear {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.875rem;
    transition: all 0.2s;
  }

  .btn-refresh {
    background: var(--accent-color, #4a9eff);
    color: white;
  }

  .btn-refresh:hover:not(:disabled) {
    background: var(--accent-hover, #3a8eef);
  }

  .btn-refresh:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .btn-clear {
    background: var(--danger-color, #e74c3c);
    color: white;
  }

  .btn-clear:hover {
    background: var(--danger-hover, #c0392b);
  }

  .confirm-dialog {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .confirm-content {
    background: var(--bg-primary, #252525);
    padding: 2rem;
    border-radius: 8px;
    max-width: 400px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
  }

  .confirm-content h3 {
    margin: 0 0 1rem;
    color: var(--text-primary, #fff);
  }

  .confirm-content p {
    margin: 0 0 1.5rem;
    color: var(--text-secondary, #aaa);
  }

  .confirm-actions {
    display: flex;
    gap: 0.5rem;
    justify-content: flex-end;
  }

  .btn-cancel,
  .btn-confirm-delete {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.875rem;
  }

  .btn-cancel {
    background: var(--bg-tertiary, #333);
    color: var(--text-primary, #fff);
  }

  .btn-confirm-delete {
    background: var(--danger-color, #e74c3c);
    color: white;
  }

  .tabs {
    display: flex;
    border-bottom: 1px solid var(--border-color, #333);
    background: var(--bg-primary, #252525);
  }

  .tab {
    padding: 0.75rem 1.5rem;
    border: none;
    background: transparent;
    color: var(--text-secondary, #aaa);
    cursor: pointer;
    font-size: 0.875rem;
    transition: all 0.2s;
    border-bottom: 2px solid transparent;
  }

  .tab:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .tab.active {
    color: var(--accent-color, #4a9eff);
    border-bottom-color: var(--accent-color, #4a9eff);
  }

  .tab-content {
    flex: 1;
    overflow-y: auto;
    padding: 1.5rem;
  }

  .cost-cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
    margin-bottom: 2rem;
  }

  .cost-card {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1.25rem;
    background: var(--bg-tertiary, #2a2a2a);
    border-radius: 8px;
    border: 1px solid var(--border-color, #333);
  }

  .cost-card.total {
    grid-column: 1 / -1;
    background: linear-gradient(135deg, #4a9eff22 0%, #9b59b622 100%);
    border-color: var(--accent-color, #4a9eff);
  }

  .card-icon {
    font-size: 2rem;
  }

  .card-content {
    flex: 1;
  }

  .card-label {
    font-size: 0.75rem;
    color: var(--text-secondary, #aaa);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 0.25rem;
  }

  .card-value {
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--text-primary, #fff);
  }

  .card-meta {
    font-size: 0.75rem;
    color: var(--text-secondary, #aaa);
    margin-top: 0.25rem;
  }

  .breakdown-section {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 1.5rem;
  }

  .breakdown-column h3 {
    margin: 0 0 1rem;
    font-size: 1rem;
    color: var(--text-primary, #fff);
  }

  .breakdown-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .breakdown-item {
    display: flex;
    justify-content: space-between;
    padding: 0.75rem;
    background: var(--bg-tertiary, #2a2a2a);
    border-radius: 6px;
    border: 1px solid var(--border-color, #333);
  }

  .breakdown-label {
    color: var(--text-secondary, #aaa);
    font-size: 0.875rem;
  }

  .breakdown-value {
    color: var(--accent-color, #4a9eff);
    font-weight: 600;
    font-size: 0.875rem;
  }

  .date-range-filter {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
    align-items: center;
    margin-bottom: 1.5rem;
    padding: 1rem;
    background: var(--bg-tertiary, #2a2a2a);
    border-radius: 8px;
  }

  .date-range-filter label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: var(--text-secondary, #aaa);
    font-size: 0.875rem;
  }

  .date-range-filter select,
  .date-range-filter input {
    padding: 0.5rem;
    background: var(--bg-primary, #252525);
    border: 1px solid var(--border-color, #333);
    border-radius: 4px;
    color: var(--text-primary, #fff);
  }

  .custom-range {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .btn-apply {
    padding: 0.5rem 1rem;
    background: var(--accent-color, #4a9eff);
    border: none;
    border-radius: 4px;
    color: white;
    cursor: pointer;
  }

  .range-summary {
    color: var(--text-secondary, #aaa);
    font-size: 0.875rem;
  }

  .range-summary strong {
    color: var(--accent-color, #4a9eff);
  }

  .session-list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .session-card {
    background: var(--bg-tertiary, #2a2a2a);
    border: 1px solid var(--border-color, #333);
    border-radius: 8px;
    padding: 1rem;
  }

  .session-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
  }

  .session-time {
    color: var(--text-secondary, #aaa);
    font-size: 0.875rem;
  }

  .session-cost {
    color: var(--accent-color, #4a9eff);
    font-weight: 600;
    font-size: 1.125rem;
  }

  .session-details {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .session-detail {
    display: flex;
    gap: 0.5rem;
    font-size: 0.875rem;
  }

  .detail-label {
    color: var(--text-secondary, #aaa);
  }

  .detail-value {
    color: var(--text-primary, #fff);
  }

  .session-stats {
    display: flex;
    gap: 1rem;
    font-size: 0.75rem;
    color: var(--text-secondary, #aaa);
    margin-top: 0.5rem;
  }

  .session-models {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-top: 0.75rem;
    padding-top: 0.75rem;
    border-top: 1px solid var(--border-color, #333);
  }

  .model-usage {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.25rem 0.75rem;
    background: var(--bg-primary, #252525);
    border-radius: 4px;
    font-size: 0.75rem;
  }

  .model-name {
    color: var(--text-secondary, #aaa);
  }

  .model-cost {
    color: var(--accent-color, #4a9eff);
    font-weight: 600;
  }

  .daily-chart {
    display: flex;
    align-items: flex-end;
    gap: 0.5rem;
    height: 300px;
    padding: 1rem;
    background: var(--bg-tertiary, #2a2a2a);
    border-radius: 8px;
  }

  .chart-bar-wrapper {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    height: 100%;
  }

  .chart-bar {
    width: 100%;
    min-height: 4px;
    background: linear-gradient(to top, var(--accent-color, #4a9eff), var(--accent-hover, #3a8eef));
    border-radius: 4px 4px 0 0;
    position: relative;
    transition: all 0.3s;
    cursor: pointer;
    display: flex;
    align-items: flex-start;
    justify-content: center;
  }

  .chart-bar:hover {
    filter: brightness(1.2);
  }

  .bar-label {
    position: absolute;
    top: -1.5rem;
    font-size: 0.625rem;
    color: var(--text-secondary, #aaa);
    white-space: nowrap;
  }

  .chart-label {
    font-size: 0.625rem;
    color: var(--text-secondary, #aaa);
    margin-top: 0.5rem;
    text-align: center;
  }

  .empty-state,
  .empty-state-large {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 2rem;
    color: var(--text-secondary, #aaa);
  }

  .empty-state-large {
    min-height: 300px;
  }

  .empty-icon {
    font-size: 3rem;
    margin-bottom: 1rem;
  }

  .empty-state-large h3 {
    margin: 0 0 0.5rem;
    color: var(--text-primary, #fff);
  }

  .empty-state-large p {
    margin: 0;
    font-size: 0.875rem;
  }
</style>
