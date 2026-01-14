<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { selectedAgentId, selectedAgentStats, updateAgentStats } from "../stores/agents";
  import type { AgentStatistics } from "../types";

  let { agentId }: { agentId?: string } = $props();

  // Use provided agentId or fall back to selected agent
  const effectiveAgentId = $derived(agentId || $selectedAgentId);

  const stats = $derived(
    effectiveAgentId && agentId ? null : $selectedAgentStats
  );

  let loading = $state(false);
  let error = $state<string | null>(null);
  let localStats = $state<AgentStatistics | null>(null);

  // Load stats when agent changes
  $effect(() => {
    if (effectiveAgentId) {
      loadStats();
    }
  });

  async function loadStats() {
    if (!effectiveAgentId) return;

    loading = true;
    error = null;

    try {
      const result = await invoke<{ agent_id: string; total_prompts: number; total_tool_calls: number; total_output_bytes: number; session_start: string; last_activity: string; total_tokens_used?: number; total_cost_usd?: number }>("get_agent_statistics", {
        agentId: effectiveAgentId,
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

      localStats = converted;
      updateAgentStats(effectiveAgentId, converted);
    } catch (e) {
      console.error("Failed to load stats:", e);
      error = e as string;
    } finally {
      loading = false;
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round(bytes / Math.pow(k, i) * 100) / 100 + " " + sizes[i];
  }

  function formatCost(cost?: number): string {
    if (cost === undefined || cost === null) return "N/A";
    return `$${cost.toFixed(4)}`;
  }

  function formatNumber(num?: number): string {
    if (num === undefined || num === null) return "N/A";
    return num.toLocaleString();
  }

  function formatDuration(start: string): string {
    try {
      const startTime = new Date(start);
      const now = new Date();
      const diffMs = now.getTime() - startTime.getTime();
      const diffMins = Math.floor(diffMs / 60000);
      const diffHours = Math.floor(diffMins / 60);
      const diffDays = Math.floor(diffHours / 24);

      if (diffDays > 0) return `${diffDays}d ${diffHours % 24}h`;
      if (diffHours > 0) return `${diffHours}h ${diffMins % 60}m`;
      return `${diffMins}m`;
    } catch {
      return "N/A";
    }
  }

  function formatTime(dateStr: string): string {
    try {
      const date = new Date(dateStr);
      return date.toLocaleTimeString();
    } catch {
      return "N/A";
    }
  }

  const displayStats = $derived(localStats || stats);
</script>

{#if loading}
  <div class="stats-panel loading">
    <div class="spinner"></div>
    <p>Loading statistics...</p>
  </div>
{:else if error}
  <div class="stats-panel error">
    <p>Failed to load statistics</p>
    <button class="secondary" onclick={loadStats}>Retry</button>
  </div>
{:else if displayStats}
  <div class="stats-panel">
    <header>
      <h3>Statistics</h3>
      <button class="icon-btn" onclick={loadStats} title="Refresh">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2"/>
        </svg>
      </button>
    </header>

    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-icon prompts">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
          </svg>
        </div>
        <div class="stat-content">
          <div class="stat-value">{displayStats.totalPrompts}</div>
          <div class="stat-label">Prompts Sent</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon tools">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/>
          </svg>
        </div>
        <div class="stat-content">
          <div class="stat-value">{displayStats.totalToolCalls}</div>
          <div class="stat-label">Tool Calls</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon tokens">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"/>
          </svg>
        </div>
        <div class="stat-content">
          <div class="stat-value">{formatNumber(displayStats.totalTokensUsed)}</div>
          <div class="stat-label">Tokens Used</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon cost">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <path d="M12 6v6l4 2"/>
          </svg>
        </div>
        <div class="stat-content">
          <div class="stat-value">{formatCost(displayStats.totalCostUsd)}</div>
          <div class="stat-label">Estimated Cost</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon output">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
            <polyline points="14 2 14 8 20 8"/>
          </svg>
        </div>
        <div class="stat-content">
          <div class="stat-value">{formatBytes(displayStats.totalOutputBytes)}</div>
          <div class="stat-label">Output Volume</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon duration">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <polyline points="12 6 12 12 16 14"/>
          </svg>
        </div>
        <div class="stat-content">
          <div class="stat-value">{formatDuration(displayStats.sessionStart)}</div>
          <div class="stat-label">Session Duration</div>
        </div>
      </div>
    </div>

    <div class="stats-footer">
      <div class="footer-item">
        <span class="footer-label">Started:</span>
        <span class="footer-value">{formatTime(displayStats.sessionStart)}</span>
      </div>
      <div class="footer-item">
        <span class="footer-label">Last Activity:</span>
        <span class="footer-value">{formatTime(displayStats.lastActivity)}</span>
      </div>
    </div>
  </div>
{:else}
  <div class="stats-panel empty">
    <p>No statistics available</p>
  </div>
{/if}

<style>
  .stats-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background-color: var(--bg-secondary);
    border-left: 1px solid var(--border);
    overflow-y: auto;
  }

  .stats-panel.loading,
  .stats-panel.error,
  .stats-panel.empty {
    align-items: center;
    justify-content: center;
    padding: var(--space-xl);
    text-align: center;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 4px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: var(--space-md);
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-lg);
    border-bottom: 1px solid var(--border);
    background: linear-gradient(180deg, var(--bg-primary) 0%, var(--bg-secondary) 100%);
  }

  h3 {
    font-size: 18px;
    font-weight: 700;
    margin: 0;
    color: var(--text-primary);
  }

  .icon-btn {
    padding: 8px;
    background: none;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    color: var(--text-secondary);
    transition: all 0.2s ease;
  }

  .icon-btn:hover {
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .icon-btn svg {
    width: 20px;
    height: 20px;
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
    gap: var(--space-md);
    padding: var(--space-lg);
  }

  .stat-card {
    display: flex;
    gap: var(--space-md);
    padding: var(--space-md);
    background: linear-gradient(135deg, var(--bg-primary) 0%, var(--bg-tertiary) 100%);
    border: 1px solid var(--border);
    border-radius: 12px;
    transition: transform 0.2s ease, box-shadow 0.2s ease;
  }

  .stat-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .stat-icon {
    width: 40px;
    height: 40px;
    border-radius: 10px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .stat-icon svg {
    width: 20px;
    height: 20px;
    color: white;
  }

  .stat-icon.prompts {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  }

  .stat-icon.tools {
    background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
  }

  .stat-icon.tokens {
    background: linear-gradient(135deg, #4facfe 0%, #00f2fe 100%);
  }

  .stat-icon.cost {
    background: linear-gradient(135deg, #43e97b 0%, #38f9d7 100%);
  }

  .stat-icon.output {
    background: linear-gradient(135deg, #fa709a 0%, #fee140 100%);
  }

  .stat-icon.duration {
    background: linear-gradient(135deg, #30cfd0 0%, #330867 100%);
  }

  .stat-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: center;
    min-width: 0;
  }

  .stat-value {
    font-size: 24px;
    font-weight: 700;
    color: var(--text-primary);
    margin-bottom: 2px;
    line-height: 1;
  }

  .stat-label {
    font-size: 12px;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .stats-footer {
    padding: var(--space-lg);
    border-top: 1px solid var(--border);
    background-color: var(--bg-primary);
    margin-top: auto;
  }

  .footer-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-sm) 0;
  }

  .footer-label {
    font-size: 13px;
    color: var(--text-muted);
    font-weight: 600;
  }

  .footer-value {
    font-size: 13px;
    color: var(--text-primary);
    font-family: 'SF Mono', 'Monaco', 'Menlo', monospace;
  }
</style>
