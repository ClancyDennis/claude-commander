<script lang="ts">
  import type { Agent } from '$lib/types';
  import { formatPath, formatTimeRelative } from '$lib/utils/formatting';
  import { getStatusColor } from '$lib/utils/status';
  import { agentsWithAlerts } from '$lib/stores/security';
  import { selectedAgentIds } from '$lib/stores/agents';

  let {
    agent,
    isSelected = false,
    isMultiSelected = false,
    onSelect,
    onMultiSelect
  }: {
    agent: Agent;
    isSelected?: boolean;
    isMultiSelected?: boolean;
    onSelect: (id: string) => void;
    onMultiSelect?: (id: string) => void;
  } = $props();

  let hasSecurityAlert = $derived($agentsWithAlerts.has(agent.id));
  let inMultiSelection = $derived($selectedAgentIds.has(agent.id));

  function handleClick(e: MouseEvent) {
    // Ctrl+click (Windows/Linux) or Cmd+click (Mac) for multi-select
    if ((e.ctrlKey || e.metaKey) && onMultiSelect) {
      onMultiSelect(agent.id);
    } else {
      onSelect(agent.id);
    }
  }
</script>

<li>
  <button
    class="agent-btn"
    class:selected={isSelected}
    class:multi-selected={inMultiSelection && !isSelected}
    onclick={handleClick}
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
        {#if hasSecurityAlert}
          <div class="security-indicator" title="Security alert">
            <svg viewBox="0 0 24 24" width="14" height="14">
              <path fill="currentColor" d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4z"/>
            </svg>
          </div>
        {/if}
        {#if agent.unreadOutputs && agent.unreadOutputs > 0}
          <span class="unread-badge animate-badge-pop">{agent.unreadOutputs}</span>
        {/if}
      </div>
      <div class="meta-row">
        <span class="path">{agent.workingDir}</span>
        {#if agent.lastActivity}
          <span class="activity-time">{formatTimeRelative(agent.lastActivity)}</span>
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

<style>
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

  .agent-btn.multi-selected {
    background: linear-gradient(135deg, rgba(59, 130, 246, 0.12) 0%, rgba(99, 102, 241, 0.08) 100%);
    border-color: rgba(59, 130, 246, 0.5);
    box-shadow: 0 0 8px rgba(59, 130, 246, 0.2);
  }

  .agent-btn.multi-selected:hover {
    border-color: rgba(59, 130, 246, 0.7);
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

  .security-indicator {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: var(--error);
    flex-shrink: 0;
    animation: security-pulse 2s ease-in-out infinite;
  }

  @keyframes security-pulse {
    0%, 100% { opacity: 1; transform: scale(1); }
    50% { opacity: 0.7; transform: scale(1.1); }
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

  .animate-badge-pop {
    animation: animate-badge-pop 0.3s ease-out;
  }
</style>
