<script lang="ts">
  import { agents, selectedAgentId, viewMode, openChat, openAgent } from "../stores/agents";
  import type { Agent } from "../types";

  let { onNewAgent, onTogglePoolDashboard, onToggleCostTracker, onToggleDatabaseStats }: {
    onNewAgent: () => void;
    onTogglePoolDashboard?: () => void;
    onToggleCostTracker?: () => void;
    onToggleDatabaseStats?: () => void;
  } = $props();

  function selectAgent(id: string) {
    openAgent(id);
  }

  function isSelected(id: string | null): boolean {
    if (id === null) {
      return $viewMode === 'chat';
    }
    return $viewMode === 'agent' && $selectedAgentId === id;
  }

  function getStatusColor(status: Agent["status"]): string {
    switch (status) {
      case "running":
        return "var(--success)";
      case "stopped":
        return "var(--text-muted)";
      case "error":
        return "var(--error)";
    }
  }

  function formatPath(path: string): string {
    const parts = path.split("/");
    return parts[parts.length - 1] || path;
  }

  function formatTime(date: Date): string {
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const minutes = Math.floor(diff / 60000);

    if (minutes < 1) return "now";
    if (minutes < 60) return `${minutes}m`;
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return `${hours}h`;
    return `${Math.floor(hours / 24)}d`;
  }
</script>

<aside class="agent-list">
  <header>
    <div class="logo">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="3"/>
        <path d="M12 1v4M12 19v4M1 12h4M19 12h4M4.22 4.22l2.83 2.83M16.95 16.95l2.83 2.83M4.22 19.78l2.83-2.83M16.95 7.05l2.83-2.83"/>
      </svg>
      <span>Claude Agents</span>
    </div>
    <button class="primary new-btn" onclick={onNewAgent}>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
        <line x1="12" y1="5" x2="12" y2="19"/>
        <line x1="5" y1="12" x2="19" y2="12"/>
      </svg>
      New
    </button>
  </header>

  <div class="list-container">
    <!-- Chat Assistant entry -->
    <ul>
      <li
        class="chat-assistant"
        class:selected={isSelected(null)}
        onclick={() => openChat()}
        role="button"
        tabindex="0"
        onkeydown={(e) => e.key === "Enter" && openChat()}
      >
        <div class="chat-icon">💬</div>
        <div class="info">
          <div class="name-row">
            <span class="name">Chat Assistant</span>
          </div>
          <div class="meta-row">
            <span class="path">AI-native control</span>
          </div>
        </div>
        <svg class="chevron" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="9,6 15,12 9,18"/>
        </svg>
      </li>
    </ul>

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
          <li
            class:selected={isSelected(agent.id)}
            onclick={() => selectAgent(agent.id)}
            role="button"
            tabindex="0"
            onkeydown={(e) => e.key === "Enter" && selectAgent(agent.id)}
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
          </li>
        {/each}
      </ul>
    {/if}
  </div>

  {#if onTogglePoolDashboard || onToggleCostTracker || onToggleDatabaseStats}
    <footer class="agent-list-footer">
      {#if onTogglePoolDashboard}
        <button class="footer-btn" onclick={onTogglePoolDashboard} title="Toggle Pool Dashboard (Ctrl+Shift+P)">
          🏊 Pool
        </button>
      {/if}
      {#if onToggleCostTracker}
        <button class="footer-btn" onclick={onToggleCostTracker} title="Toggle Cost Tracker (Ctrl+Shift+$)">
          💰 Costs
        </button>
      {/if}
      {#if onToggleDatabaseStats}
        <button class="footer-btn" onclick={onToggleDatabaseStats} title="Toggle Database Stats (Ctrl+Shift+D)">
          💾 Database
        </button>
      {/if}
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

  .list-container {
    flex: 1;
    overflow-y: auto;
  }

  ul {
    list-style: none;
    padding: var(--space-sm);
  }

  li {
    padding: var(--space-md) var(--space-lg);
    display: flex;
    align-items: center;
    gap: 16px;
    cursor: pointer;
    border-radius: 12px;
    margin-bottom: var(--space-sm);
    transition: all 0.2s ease;
    background-color: var(--bg-tertiary);
    border: 1px solid transparent;
  }

  li:hover {
    background-color: var(--bg-elevated);
    border-color: var(--border);
  }

  li.selected {
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

  li.selected .chevron {
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
</style>
