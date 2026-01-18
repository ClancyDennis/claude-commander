<script lang="ts">
  import { agents, selectedAgentId, viewMode, openChat, openAgent, sidebarMode, historicalRuns, toggleSidebarMode, setHistoricalRuns, selectHistoricalRun } from "../stores/agents";
  import { autoPipelines, selectedAutoPipelineId, selectAutoPipeline } from "../stores/autoPipelines";
  import type { Agent, AgentRun, RunStatus, AutoPipeline } from "../types";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let { onNewAgent, onToggleDatabaseStats }: {
    onNewAgent: () => void;
    onToggleDatabaseStats?: () => void;
  } = $props();

  // Load historical runs when sidebar mode changes to history
  $effect(() => {
    if ($sidebarMode === 'history') {
      loadHistoricalRuns();
    }
  });

  async function loadHistoricalRuns() {
    try {
      const runs = await invoke<AgentRun[]>("get_all_runs");
      setHistoricalRuns(runs);
    } catch (error) {
      console.error("Failed to load historical runs:", error);
    }
  }

  function handleToggleSidebarMode() {
    toggleSidebarMode();
  }

  function selectHistoricalRunItem(run: AgentRun) {
    selectHistoricalRun(run);
  }

  function selectAgent(id: string) {
    openAgent(id);
  }

  function isSelected(id: string | null): boolean {
    if (id === null) {
      return $viewMode === 'chat' && !$selectedAutoPipelineId;
    }
    return $viewMode === 'agent' && $selectedAgentId === id && !$selectedAutoPipelineId;
  }

  function isPipelineSelected(id: string): boolean {
    return $selectedAutoPipelineId === id;
  }

  function handleSelectPipeline(id: string) {
    selectAutoPipeline(id);
  }

  function getPipelineStatusColor(status: string): string {
    switch (status) {
      case 'Completed': return '#10b981'; // green
      case 'Running': return 'var(--accent)';
      case 'Failed': return 'var(--error)';
      default: return 'var(--text-muted)';
    }
  }

  function getStatusColor(status: Agent["status"] | RunStatus): string {
    switch (status) {
      case "running":
        return "var(--success)";
      case "stopped":
        return "var(--text-muted)";
      case "error":
      case "crashed":
        return "var(--error)";
      case "completed":
        return "#10b981"; // green-500
      case "waiting_input":
        return "var(--warning)";
      default:
        return "var(--text-muted)";
    }
  }

  function getStatusLabel(status: RunStatus): string {
    switch (status) {
      case "running":
        return "Running";
      case "completed":
        return "Completed";
      case "stopped":
        return "Stopped";
      case "crashed":
        return "Crashed";
      case "waiting_input":
        return "Waiting";
      default:
        return status;
    }
  }

  function formatPath(path: string): string {
    const parts = path.split("/");
    return parts[parts.length - 1] || path;
  }

  function formatTime(date: Date | number): string {
    const now = new Date();
    const targetDate = typeof date === 'number' ? new Date(date) : date;
    const diff = now.getTime() - targetDate.getTime();
    const minutes = Math.floor(diff / 60000);

    if (minutes < 1) return "now";
    if (minutes < 60) return `${minutes}m`;
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return `${hours}h`;
    return `${Math.floor(hours / 24)}d`;
  }

  function formatDate(timestamp: number): string {
    const date = new Date(timestamp);
    const now = new Date();
    const isToday = date.toDateString() === now.toDateString();

    if (isToday) {
      return date.toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit' });
    }

    const yesterday = new Date(now);
    yesterday.setDate(yesterday.getDate() - 1);
    if (date.toDateString() === yesterday.toDateString()) {
      return 'Yesterday';
    }

    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
  }

  function formatDuration(startTime: number, endTime?: number): string {
    const end = endTime || Date.now();
    const duration = end - startTime;
    const minutes = Math.floor(duration / 60000);

    if (minutes < 1) return "< 1m";
    if (minutes < 60) return `${minutes}m`;
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return `${hours}h ${minutes % 60}m`;
    const days = Math.floor(hours / 24);
    return `${days}d ${hours % 24}h`;
  }
</script>

<aside class="agent-list">
  <header>
    <div class="logo">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <circle cx="12" cy="12" r="3"/>
        <line x1="12" y1="2" x2="12" y2="6"/>
        <line x1="12" y1="18" x2="12" y2="22"/>
        <line x1="2" y1="12" x2="6" y2="12"/>
        <line x1="18" y1="12" x2="22" y2="12"/>
      </svg>
      <span>Claude Commander</span>
    </div>
    <button class="primary new-btn" onclick={onNewAgent}>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
        <line x1="12" y1="5" x2="12" y2="19"/>
        <line x1="5" y1="12" x2="19" y2="12"/>
      </svg>
      New
    </button>
  </header>

  <!-- Toggle between Running and History -->
  <div class="sidebar-toggle">
    <button
      class="toggle-btn"
      class:active={$sidebarMode === 'running'}
      onclick={() => handleToggleSidebarMode()}
      disabled={$sidebarMode === 'running'}
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <polyline points="12 6 12 12 16 14"/>
      </svg>
      Running
    </button>
    <button
      class="toggle-btn"
      class:active={$sidebarMode === 'history'}
      onclick={() => handleToggleSidebarMode()}
      disabled={$sidebarMode === 'history'}
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
        <path d="M3 3v5h5"/>
        <polyline points="12 7 12 12 15 15"/>
      </svg>
      History
    </button>
  </div>

  <div class="list-container">
    {#if $sidebarMode === 'running'}
      <!-- Chat Assistant entry -->
      <ul>
        <li>
          <button
            class="agent-btn chat-assistant"
            class:selected={isSelected(null)}
            onclick={() => openChat()}
          >
            <div class="chat-icon">🎯</div>
            <div class="info">
              <div class="name-row">
                <span class="name">System Commander</span>
              </div>
              <div class="meta-row">
                <span class="path">Mission control for Claude</span>
              </div>
            </div>
            <svg class="chevron" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="9,6 15,12 9,18"/>
            </svg>
          </button>
        </li>
      </ul>

      <!-- Auto Pipelines Section -->
      {#if $autoPipelines.size > 0}
        <div class="separator">
          <span>Auto Pipelines ({$autoPipelines.size})</span>
        </div>
        <ul>
          {#each [...$autoPipelines.values()] as pipeline (pipeline.id)}
            <li>
              <button
                class="agent-btn pipeline-btn"
                class:selected={isPipelineSelected(pipeline.id)}
                onclick={() => handleSelectPipeline(pipeline.id)}
              >
                <div class="status-indicator" style="background-color: {getPipelineStatusColor(pipeline.status)}">
                  {#if pipeline.status === 'Running'}
                    <span class="pulse"></span>
                  {/if}
                </div>
                <div class="info">
                  <div class="name-row">
                    <span class="name">Pipeline</span>
                    <span class="pipeline-status-badge" style="background-color: {getPipelineStatusColor(pipeline.status)}">
                      {pipeline.status}
                    </span>
                  </div>
                  <div class="meta-row">
                    <span class="path pipeline-request">{pipeline.user_request.length > 40 ? pipeline.user_request.substring(0, 40) + '...' : pipeline.user_request}</span>
                  </div>
                </div>
                <svg class="chevron" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="9,6 15,12 9,18"/>
                </svg>
              </button>
            </li>
          {/each}
        </ul>
      {/if}

      <!-- Separator -->
      {#if $agents.size > 0}
        <div class="separator">
          <span>Worker Agents</span>
        </div>
      {/if}

      {#if $agents.size === 0}
        <div class="empty">
          <div class="empty-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <rect x="3" y="3" width="18" height="18" rx="2"/>
              <circle cx="12" cy="10" r="3"/>
              <path d="M7 21v-2a4 4 0 0 1 4-4h2a4 4 0 0 1 4 4v2"/>
            </svg>
          </div>
          <p class="empty-title">No worker agents</p>
          <p class="empty-hint">Tap "New" or use Chat to create agents</p>
        </div>
      {:else}
        <ul>
          {#each [...$agents.values()] as agent (agent.id)}
            <li>
              <button
                class="agent-btn"
                class:selected={isSelected(agent.id)}
                onclick={() => selectAgent(agent.id)}
              >
                <div class="status-indicator" style="background-color: {getStatusColor(agent.status)}">
                  {#if agent.status === "running" || agent.status === "waitingforinput"}
                    <span class="pulse"></span>
                  {/if}
                  {#if agent.isProcessing}
                    <div class="processing-ring"></div>
                  {/if}
                </div>
                <div class="info">
                  <div class="name-row">
                    <span class="name">{formatPath(agent.workingDir)}</span>
                    {#if agent.unreadOutputs && agent.unreadOutputs > 0}
                      <span class="unread-badge animate-badge-pop">{agent.unreadOutputs}</span>
                    {/if}
                  </div>
                  <div class="meta-row">
                    <span class="path">{agent.workingDir}</span>
                    {#if agent.lastActivity}
                      <span class="activity-time">{formatTime(agent.lastActivity)}</span>
                    {/if}
                  </div>
                </div>
                {#if agent.pendingInput}
                  <div class="pending-input-icon" title="Waiting for input">
                    <svg viewBox="0 0 24 24" fill="currentColor">
                      <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
                    </svg>
                  </div>
                {:else}
                  <svg class="chevron" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="9,6 15,12 9,18"/>
                  </svg>
                {/if}
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    {:else}
      <!-- Historical Runs View -->
      <div class="separator">
        <span>Historical Runs ({$historicalRuns.length})</span>
      </div>

      {#if $historicalRuns.length === 0}
        <div class="empty">
          <div class="empty-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
              <path d="M3 3v5h5"/>
              <circle cx="12" cy="12" r="1"/>
            </svg>
          </div>
          <p class="empty-title">No history yet</p>
          <p class="empty-hint">Agent runs will appear here</p>
        </div>
      {:else}
        <ul>
          {#each $historicalRuns as run (run.agent_id)}
            <li>
              <button
                class="agent-btn"
                onclick={() => selectHistoricalRunItem(run)}
              >
                <div class="status-indicator" style="background-color: {getStatusColor(run.status)}">
                  {#if run.status === "running"}
                    <span class="pulse"></span>
                  {/if}
                </div>
                <div class="info">
                  <div class="name-row">
                    <span class="name">{formatPath(run.working_dir)}</span>
                    <span class="status-badge" style="background-color: {getStatusColor(run.status)}">
                      {getStatusLabel(run.status)}
                    </span>
                  </div>
                  <div class="meta-row">
                    <span class="path">{formatDate(run.started_at)}</span>
                    <span class="activity-time">{formatDuration(run.started_at, run.ended_at)}</span>
                  </div>
                  {#if run.initial_prompt}
                    <div class="run-prompt">
                      {run.initial_prompt.length > 60 ? run.initial_prompt.substring(0, 60) + '...' : run.initial_prompt}
                    </div>
                  {/if}
                </div>
                <svg class="chevron" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="9,6 15,12 9,18"/>
                </svg>
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    {/if}
  </div>

  {#if onToggleDatabaseStats}
    <footer class="agent-list-footer">
      <button class="footer-btn" onclick={onToggleDatabaseStats} title="Toggle Database Stats (Ctrl+Shift+D)">
        💾 Database
      </button>
    </footer>
  {/if}
</aside>

<style>
  .agent-list {
    width: 320px;
    min-width: 320px;
    height: 100%;
    background-color: var(--bg-secondary);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
  }

  header {
    padding: var(--space-lg);
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--border);
    background: linear-gradient(180deg, var(--bg-tertiary) 0%, var(--bg-secondary) 100%);
  }

  .logo {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 18px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .logo svg {
    width: 28px;
    height: 28px;
    color: var(--accent);
  }

  .new-btn {
    padding: 12px 20px;
  }

  .new-btn svg {
    width: 18px;
    height: 18px;
  }

  .sidebar-toggle {
    display: flex;
    gap: 8px;
    padding: var(--space-md) var(--space-lg);
    border-bottom: 1px solid var(--border);
    background-color: var(--bg-primary);
  }

  .toggle-btn {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 10px 16px;
    border: 1px solid var(--border);
    background-color: var(--bg-tertiary);
    color: var(--text-muted);
    border-radius: 8px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .toggle-btn svg {
    width: 16px;
    height: 16px;
  }

  .toggle-btn:hover:not(:disabled) {
    background-color: var(--bg-elevated);
    border-color: var(--accent);
    color: var(--text-primary);
  }

  .toggle-btn.active {
    background: linear-gradient(135deg, rgba(124, 58, 237, 0.15) 0%, rgba(147, 51, 234, 0.1) 100%);
    border-color: var(--accent);
    color: var(--accent);
    box-shadow: 0 0 8px var(--accent-glow);
  }

  .toggle-btn:disabled {
    cursor: default;
  }

  .list-container {
    flex: 1;
    overflow-y: auto;
  }

  ul {
    list-style: none;
    padding: var(--space-sm);
  }

  li {
    padding: 0;
    margin-bottom: var(--space-sm);
  }

  .agent-btn {
    width: 100%;
    padding: var(--space-md) var(--space-lg);
    display: flex;
    align-items: center;
    gap: 16px;
    cursor: pointer;
    border-radius: 12px;
    transition: all 0.2s ease;
    background-color: var(--bg-tertiary);
    border: 1px solid transparent;
    text-align: left;
    font: inherit;
    color: inherit;
  }

  .agent-btn:hover {
    background-color: var(--bg-elevated);
    border-color: var(--border);
  }

  .agent-btn.selected {
    background: linear-gradient(135deg, rgba(124, 58, 237, 0.15) 0%, rgba(147, 51, 234, 0.1) 100%);
    border-color: var(--accent);
    box-shadow: 0 0 16px var(--accent-glow);
  }

  .status-indicator {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    flex-shrink: 0;
    position: relative;
  }

  .pulse {
    position: absolute;
    inset: -3px;
    border-radius: 50%;
    background: inherit;
    opacity: 0.4;
    animation: pulse 2s ease-in-out infinite;
  }

  .processing-ring {
    position: absolute;
    inset: -5px;
    border: 2px solid var(--accent);
    border-radius: 50%;
    border-top-color: transparent;
    animation: spinner-rotate 1s linear infinite;
  }

  .info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .name-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .name {
    font-weight: 600;
    font-size: 16px;
    color: var(--text-primary);
  }

  .unread-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 18px;
    height: 18px;
    padding: 0 5px;
    background-color: var(--error);
    color: white;
    font-size: 11px;
    font-weight: 700;
    border-radius: 9px;
    flex-shrink: 0;
  }

  .status-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 2px 8px;
    color: white;
    font-size: 10px;
    font-weight: 600;
    border-radius: 10px;
    flex-shrink: 0;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .run-prompt {
    font-size: 12px;
    color: var(--text-muted);
    margin-top: 4px;
    font-style: italic;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .meta-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .path {
    font-size: 13px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
  }

  .activity-time {
    font-size: 11px;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .pending-input-icon {
    width: 20px;
    height: 20px;
    color: var(--warning);
    flex-shrink: 0;
    animation: glow-pulse 2s ease-in-out infinite;
  }

  .pending-input-icon svg {
    width: 100%;
    height: 100%;
  }

  .chevron {
    width: 20px;
    height: 20px;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .agent-btn.selected .chevron {
    color: var(--accent);
  }

  .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-xl);
    text-align: center;
    height: 100%;
    min-height: 300px;
  }

  .empty-icon {
    width: 80px;
    height: 80px;
    border-radius: 24px;
    background: linear-gradient(135deg, var(--bg-tertiary) 0%, var(--bg-elevated) 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: var(--space-lg);
    border: 1px solid var(--border);
  }

  .empty-icon svg {
    width: 40px;
    height: 40px;
    color: var(--text-muted);
  }

  .empty-title {
    font-size: 18px;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: var(--space-sm);
  }

  .empty-hint {
    font-size: 14px;
    color: var(--text-muted);
  }

  @keyframes pulse {
    0%, 100% { transform: scale(1); opacity: 0.4; }
    50% { transform: scale(1.5); opacity: 0; }
  }

  @keyframes spinner-rotate {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  @keyframes glow-pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  @keyframes animate-badge-pop {
    0% { transform: scale(0.5); }
    50% { transform: scale(1.1); }
    100% { transform: scale(1); }
  }

  .chat-assistant {
    background: linear-gradient(135deg, rgba(124, 58, 237, 0.1) 0%, rgba(147, 51, 234, 0.05) 100%);
    border: 1px solid rgba(124, 58, 237, 0.3);
  }

  .chat-assistant.selected {
    background: linear-gradient(135deg, rgba(124, 58, 237, 0.2) 0%, rgba(147, 51, 234, 0.15) 100%);
    border-color: var(--accent);
    box-shadow: 0 0 16px var(--accent-glow);
  }

  .chat-icon {
    font-size: 24px;
    width: 28px;
    text-align: center;
  }

  .separator {
    padding: 12px var(--space-lg);
    font-size: 12px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .agent-list-footer {
    display: flex;
    gap: 0.5rem;
    padding: 1rem;
    border-top: 1px solid var(--border);
    background: var(--bg-primary);
  }

  .footer-btn {
    flex: 1;
    padding: 0.75rem;
    border: 1px solid var(--border);
    background: var(--bg-secondary);
    color: var(--text);
    border-radius: 6px;
    font-size: 0.875rem;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.25rem;
  }

  .footer-btn:hover {
    background: var(--bg-tertiary);
    border-color: var(--accent);
  }

  .pipeline-btn {
    background: linear-gradient(135deg, rgba(16, 185, 129, 0.1) 0%, rgba(5, 150, 105, 0.05) 100%);
    border: 1px solid rgba(16, 185, 129, 0.3);
  }

  .pipeline-btn.selected {
    background: linear-gradient(135deg, rgba(16, 185, 129, 0.2) 0%, rgba(5, 150, 105, 0.15) 100%);
    border-color: #10b981;
    box-shadow: 0 0 16px rgba(16, 185, 129, 0.3);
  }

  .pipeline-status-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 2px 8px;
    color: white;
    font-size: 10px;
    font-weight: 600;
    border-radius: 10px;
    flex-shrink: 0;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .pipeline-request {
    font-style: italic;
  }
</style>
