<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { selectedAgentId, selectedAgentStats, updateAgentStats } from "../stores/agents";
  import type { AgentStatistics } from "../types";
  import { formatBytes, formatCost, formatNumber, formatDuration, formatTimeLocale } from '$lib/utils/formatting';
  import { useAsyncData } from '$lib/hooks/useAsyncData.svelte';
  import HelpTip from "./new-agent/HelpTip.svelte";

  let { agentId }: { agentId?: string } = $props();

  // Use provided agentId or fall back to selected agent
  const effectiveAgentId = $derived(agentId || $selectedAgentId);

  const stats = $derived(
    effectiveAgentId && agentId ? null : $selectedAgentStats
  );

  // Track the agent ID we're fetching for (needed for the closure)
  let fetchAgentId = $state<string | null>(null);

  const asyncStats = useAsyncData(async () => {
    if (!fetchAgentId) return null;

    const result = await invoke<{ agent_id: string; total_prompts: number; total_tool_calls: number; total_output_bytes: number; session_start: string; last_activity: string; total_tokens_used?: number; total_cost_usd?: number }>("get_agent_statistics", {
      agentId: fetchAgentId,
    });

    // Convert snake_case to camelCase
    const converted: AgentStatistics = {
      agentId: result.agent_id,
      totalPrompts: result.total_prompts,
      totalToolCalls: result.total_tool_calls,
      totalOutputBytes: result.total_output_bytes,
      sessionStart: result.session_start,
      lastActivity: result.last_activity,
      totalTokensUsed: result.total_tokens_used,
      totalCostUsd: result.total_cost_usd,
    };

    updateAgentStats(fetchAgentId, converted);
    return converted;
  });

  async function loadStats() {
    if (!effectiveAgentId) return;
    fetchAgentId = effectiveAgentId;
    await asyncStats.fetch();
  }

  // Load stats when agent changes
  $effect(() => {
    if (effectiveAgentId) {
      loadStats();
    }
  });

  const displayStats = $derived(asyncStats.data || stats);
</script>

{#if asyncStats.loading}
  <div class="stats-panel loading">
    <div class="spinner"></div>
    <p class="loading-text">Loading statistics...</p>
  </div>
{:else if asyncStats.error}
  <div class="stats-panel error">
    <p class="error-text">Failed to load statistics</p>
    <button class="retry-btn" onclick={loadStats}>Retry</button>
  </div>
{:else if displayStats}
  <div class="stats-panel">
    <header>
      <h3>Statistics</h3>
      <button class="refresh-btn" onclick={loadStats} title="Refresh">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2"/>
        </svg>
      </button>
    </header>

    <div class="stats-content">
      <!-- Activity Section -->
      <section class="stats-section">
        <h4 class="section-title">Activity</h4>
        <div class="stats-list">
          <div class="stat-row">
            <div class="stat-info">
              <svg class="stat-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
              </svg>
              <span class="stat-label">Prompts</span>
            </div>
            <span class="stat-value">{displayStats.totalPrompts}</span>
          </div>

          <div class="stat-row">
            <div class="stat-info">
              <svg class="stat-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/>
              </svg>
              <span class="stat-label">
                Tool calls
                <HelpTip text="Number of tool invocations (file reads, writes, searches) made by the agent." placement="top" />
              </span>
            </div>
            <span class="stat-value">{displayStats.totalToolCalls}</span>
          </div>

          <div class="stat-row">
            <div class="stat-info">
              <svg class="stat-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                <polyline points="14 2 14 8 20 8"/>
              </svg>
              <span class="stat-label">Output</span>
            </div>
            <span class="stat-value">{formatBytes(displayStats.totalOutputBytes)}</span>
          </div>
        </div>
      </section>

      <!-- Resources Section -->
      <section class="stats-section">
        <h4 class="section-title">Resources</h4>
        <div class="stats-list">
          <div class="stat-row">
            <div class="stat-info">
              <svg class="stat-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <circle cx="12" cy="12" r="10"/>
                <path d="M12 6v6l4 2"/>
              </svg>
              <span class="stat-label">
                Tokens
                <HelpTip text="Total tokens sent to the AI model. Higher counts mean more context and cost." placement="top" />
              </span>
            </div>
            <span class="stat-value">{formatNumber(displayStats.totalTokensUsed)}</span>
          </div>

          <div class="stat-row">
            <div class="stat-info">
              <svg class="stat-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"/>
              </svg>
              <span class="stat-label">
                Est. cost
                <HelpTip text="Approximate API cost based on token usage. Actual billing may vary." placement="top" />
              </span>
            </div>
            <span class="stat-value accent">{formatCost(displayStats.totalCostUsd)}</span>
          </div>
        </div>
      </section>

      <!-- Session Section -->
      <section class="stats-section">
        <h4 class="section-title">Session</h4>
        <div class="stats-list">
          <div class="stat-row">
            <div class="stat-info">
              <svg class="stat-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <circle cx="12" cy="12" r="10"/>
                <polyline points="12 6 12 12 16 14"/>
              </svg>
              <span class="stat-label">Duration</span>
            </div>
            <span class="stat-value">{formatDuration(displayStats.sessionStart)}</span>
          </div>

          <div class="stat-row">
            <div class="stat-info">
              <svg class="stat-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <rect x="3" y="4" width="18" height="18" rx="2" ry="2"/>
                <line x1="16" y1="2" x2="16" y2="6"/>
                <line x1="8" y1="2" x2="8" y2="6"/>
                <line x1="3" y1="10" x2="21" y2="10"/>
              </svg>
              <span class="stat-label">Started</span>
            </div>
            <span class="stat-value mono">{formatTimeLocale(displayStats.sessionStart)}</span>
          </div>

          <div class="stat-row">
            <div class="stat-info">
              <svg class="stat-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/>
              </svg>
              <span class="stat-label">Last activity</span>
            </div>
            <span class="stat-value mono">{formatTimeLocale(displayStats.lastActivity)}</span>
          </div>
        </div>
      </section>
    </div>
  </div>
{:else}
  <div class="stats-panel empty">
    <p class="empty-text">No statistics available</p>
  </div>
{/if}

<style>
  .stats-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background-color: var(--bg-secondary);
    border-left: 1px solid var(--border-hex);
    overflow-y: auto;
  }

  .stats-panel.loading,
  .stats-panel.error,
  .stats-panel.empty {
    align-items: center;
    justify-content: center;
    padding: var(--space-8);
    text-align: center;
    gap: var(--space-4);
  }

  .loading-text,
  .error-text,
  .empty-text {
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 2px solid var(--border-hex);
    border-top-color: var(--text-secondary);
    border-radius: var(--radius-full);
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .retry-btn {
    padding: var(--space-2) var(--space-4);
    background: var(--bg-tertiary);
    color: var(--text-primary);
    border: none;
    border-radius: var(--radius-md);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    cursor: pointer;
    transition: background var(--transition-fast);
  }

  .retry-btn:hover {
    background: var(--bg-elevated);
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-4) var(--space-5);
    border-bottom: 1px solid var(--border-hex);
    background: var(--bg-secondary);
    flex-shrink: 0;
  }

  h3 {
    font-size: var(--text-base);
    font-weight: var(--font-semibold);
    margin: 0;
    color: var(--text-primary);
  }

  .refresh-btn {
    width: 28px;
    height: 28px;
    padding: 0;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    color: var(--text-muted);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--transition-fast);
  }

  .refresh-btn:hover {
    background-color: var(--bg-tertiary);
    color: var(--text-secondary);
  }

  .refresh-btn:active {
    transform: scale(0.95);
  }

  .refresh-btn svg {
    width: 16px;
    height: 16px;
  }

  .stats-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-3) 0;
  }

  .stats-section {
    padding: var(--space-3) var(--space-5);
  }

  .stats-section:not(:last-child) {
    border-bottom: 1px solid var(--border-hex);
  }

  .section-title {
    font-size: var(--text-xs);
    font-weight: var(--font-semibold);
    color: var(--text-muted);
    margin: 0 0 var(--space-3) 0;
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .stats-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .stat-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-2) var(--space-3);
    margin: 0 calc(var(--space-3) * -1);
    border-radius: var(--radius-sm);
    transition: background var(--transition-fast);
  }

  .stat-row:hover {
    background: var(--bg-tertiary);
  }

  .stat-info {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    min-width: 0;
  }

  .stat-icon {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
    color: var(--text-muted);
  }

  .stat-label {
    font-size: var(--text-sm);
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    gap: var(--space-1);
  }

  .stat-value {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--text-primary);
    font-variant-numeric: tabular-nums;
  }

  .stat-value.accent {
    color: var(--accent-hex);
  }

  .stat-value.mono {
    font-family: 'SF Mono', 'Monaco', 'Menlo', monospace;
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }
</style>
