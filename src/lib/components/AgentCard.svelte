<script lang="ts">
  import type { Agent } from "../types";
  import { selectedAgentOutputs, agentOutputs } from "../stores/agents";
  import { agentsWithAlerts } from "../stores/security";
  import { agentActivities } from "../stores/activity";
  import StatusBadge from "./StatusBadge.svelte";
  import TypingIndicator from "./TypingIndicator.svelte";
  import { formatPath, formatTimeAgo } from '$lib/utils/formatting';

  let { agent }: { agent: Agent } = $props();

  const outputs = $derived($agentOutputs.get(agent.id) ?? []);
  const recentOutputs = $derived(outputs.slice(-3));
  const hasOutputs = $derived(outputs.length > 0);
  const hasSecurityAlert = $derived($agentsWithAlerts.has(agent.id));
  const activity = $derived($agentActivities.get(agent.id));
  const currentActivity = $derived(activity?.currentActivity);
</script>

<div class="agent-card">
  <div class="card-header">
    <div class="agent-info">
      <div class="agent-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="4" y="4" width="16" height="16" rx="2"/>
          <path d="M9 9h6M9 13h6M9 17h4"/>
        </svg>
      </div>
      <div class="agent-details">
        <h3 class="agent-name">{#if agent.title}{agent.title} - {/if}{formatPath(agent.workingDir)}</h3>
        <span class="agent-path">{agent.workingDir}</span>
      </div>
    </div>
    <div class="header-badges">
      {#if hasSecurityAlert}
        <span class="security-badge">Alert</span>
      {/if}
      <StatusBadge status={agent.status} size="small" showLabel={false} />
    </div>
  </div>

  <div class="card-content">
    {#if agent.isProcessing}
      <div class="processing-status">
        <TypingIndicator />
        {#if currentActivity}
          <span class="current-activity">{currentActivity}</span>
        {/if}
      </div>
    {:else if hasOutputs}
      <div class="recent-outputs">
        {#each recentOutputs as output (output.timestamp)}
          <div class="output-preview {output.type}">
            <span class="output-label">{output.type}</span>
            <p class="output-text">{output.content.slice(0, 100)}{output.content.length > 100 ? "..." : ""}</p>
          </div>
        {/each}
      </div>
    {:else}
      <div class="no-output">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
        </svg>
        <span>No output yet</span>
      </div>
    {/if}
  </div>

  <div class="card-footer">
    <div class="stats">
      <div class="stat">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"/>
        </svg>
        <span>{outputs.length}</span>
      </div>
      {#if agent.unreadOutputs && agent.unreadOutputs > 0}
        <div class="unread-badge animate-badge-pop">
          {agent.unreadOutputs}
        </div>
      {/if}
    </div>
    <div class="last-activity">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <polyline points="12 6 12 12 16 14"/>
      </svg>
      <span>{formatTimeAgo(agent.lastActivity)}</span>
    </div>
  </div>
</div>

<style>
  .agent-card {
    display: flex;
    flex-direction: column;
    height: 100%;
    background-color: var(--bg-secondary);
  }

  .card-header {
    padding: var(--space-md);
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-sm);
    border-bottom: 1px solid var(--border);
    background: linear-gradient(180deg, var(--bg-tertiary) 0%, var(--bg-secondary) 100%);
  }

  .agent-info {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    flex: 1;
    min-width: 0;
  }

  .agent-icon {
    width: 36px;
    height: 36px;
    border-radius: 10px;
    background: linear-gradient(135deg, var(--accent-hex) 0%, #e85a45 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    box-shadow: 0 4px 12px var(--accent-glow);
  }

  .agent-icon svg {
    width: 18px;
    height: 18px;
    color: white;
  }

  .agent-details {
    flex: 1;
    min-width: 0;
  }

  .agent-name {
    font-size: 15px;
    font-weight: 700;
    margin-bottom: 2px;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .agent-path {
    font-size: 11px;
    color: var(--text-muted);
    display: block;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .header-badges {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    flex-shrink: 0;
  }

  .security-badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: var(--error);
    color: white;
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 600;
    animation: security-badge-pulse 2s ease-in-out infinite;
  }

  @keyframes security-badge-pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.7; }
  }

  .card-content {
    flex: 1;
    padding: var(--space-md);
    overflow-y: auto;
    min-height: 0;
  }

  .recent-outputs {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .output-preview {
    padding: var(--space-sm);
    background-color: var(--bg-tertiary);
    border-radius: 8px;
    border: 1px solid var(--border);
  }

  .output-label {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--accent);
    display: block;
    margin-bottom: 4px;
  }

  .output-text {
    font-size: 12px;
    line-height: 1.4;
    color: var(--text-secondary);
    margin: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .output-preview.error .output-label {
    color: var(--error);
  }

  .output-preview.error {
    border-color: var(--error);
    background-color: var(--error-glow);
  }

  .processing-status {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-sm);
    height: 100%;
  }

  .current-activity {
    font-size: 12px;
    color: var(--text-secondary);
    text-align: center;
    padding: 0 var(--space-sm);
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .no-output {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-sm);
    color: var(--text-muted);
  }

  .no-output svg {
    width: 32px;
    height: 32px;
  }

  .no-output span {
    font-size: 13px;
  }

  .card-footer {
    padding: var(--space-md);
    border-top: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    background-color: var(--bg-tertiary);
  }

  .stats {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .stat {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    color: var(--text-muted);
  }

  .stat svg {
    width: 16px;
    height: 16px;
  }

  .unread-badge {
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 20px;
    height: 20px;
    padding: 0 6px;
    background-color: var(--error);
    color: white;
    font-size: 11px;
    font-weight: 700;
    border-radius: 10px;
  }

  .last-activity {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-muted);
  }

  .last-activity svg {
    width: 14px;
    height: 14px;
  }
</style>
