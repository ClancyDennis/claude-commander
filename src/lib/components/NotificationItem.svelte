<script lang="ts">
  import type { SecurityAlert } from "../types";
  import { selectedAgentId } from "../stores/agents";
  import { showAlertDetail, markAlertRead, dismissAlert } from "../stores/security";

  let {
    alert,
    isRead,
  }: {
    alert: SecurityAlert;
    isRead: boolean;
  } = $props();

  function getSeverityColor(severity: string): string {
    switch (severity.toLowerCase()) {
      case "critical":
      case "high":
        return "var(--error)";
      case "medium":
        return "var(--warning)";
      case "low":
        return "var(--accent)";
      default:
        return "var(--text-muted)";
    }
  }

  function getRelativeTime(date: Date): string {
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMinutes = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMinutes < 1) return "just now";
    if (diffMinutes < 60) return `${diffMinutes}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    return date.toLocaleDateString();
  }

  function handleViewDetails() {
    showAlertDetail(alert);
    markAlertRead(alert.alertId);
  }

  function handleViewAgent() {
    selectedAgentId.set(alert.agentId);
    markAlertRead(alert.alertId);
  }

  function handleDismiss() {
    dismissAlert(alert.alertId);
  }

  function handleMarkRead() {
    markAlertRead(alert.alertId);
  }
</script>

<div class="notification-item" class:unread={!isRead}>
  <div class="indicator" style="background-color: {getSeverityColor(alert.severity)}">
    {#if !isRead}
      <div class="unread-dot"></div>
    {/if}
  </div>

  <div class="content">
    <div class="header-row">
      <div class="title-group">
        <span class="severity-badge" style="background-color: {getSeverityColor(alert.severity)}20; color: {getSeverityColor(alert.severity)}; border-color: {getSeverityColor(alert.severity)}">
          {alert.severity.toUpperCase()}
        </span>
        <h4>{alert.title}</h4>
      </div>
      <span class="timestamp">{getRelativeTime(alert.timestamp)}</span>
    </div>

    <p class="description">{alert.description}</p>

    <div class="meta">
      <span class="agent-id">Agent: {alert.agentId.slice(0, 8)}</span>
      {#if alert.threats && alert.threats.length > 0}
        <span class="threat-count">{alert.threats.length} threat{alert.threats.length > 1 ? 's' : ''}</span>
      {/if}
    </div>

    <div class="actions">
      <button class="action-btn primary" onclick={handleViewDetails}>View Details</button>
      <button class="action-btn" onclick={handleViewAgent}>View Agent</button>
      {#if !isRead}
        <button class="action-btn" onclick={handleMarkRead}>Mark Read</button>
      {/if}
      <button class="action-btn dismiss" onclick={handleDismiss}>Dismiss</button>
    </div>
  </div>
</div>

<style>
  .notification-item {
    display: flex;
    gap: var(--space-md);
    padding: var(--space-md);
    background-color: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 12px;
    transition: all 0.2s ease;
  }

  .notification-item.unread {
    background-color: var(--bg-secondary);
    border-left-width: 3px;
  }

  .indicator {
    width: 4px;
    border-radius: 2px;
    position: relative;
    flex-shrink: 0;
  }

  .unread-dot {
    position: absolute;
    top: 4px;
    left: 50%;
    transform: translateX(-50%);
    width: 8px;
    height: 8px;
    background-color: currentColor;
    border-radius: 50%;
    animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
  }

  .content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-md);
  }

  .title-group {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    flex: 1;
  }

  .severity-badge {
    padding: 2px 8px;
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.05em;
    border-radius: 4px;
    border: 1px solid;
    flex-shrink: 0;
  }

  h4 {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .timestamp {
    font-size: 12px;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .description {
    font-size: 13px;
    color: var(--text-secondary);
    line-height: 1.5;
    margin: 0;
  }

  .meta {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    font-size: 12px;
    color: var(--text-muted);
  }

  .agent-id {
    font-family: monospace;
  }

  .threat-count {
    font-weight: 500;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-sm);
    margin-top: var(--space-xs);
  }

  .action-btn {
    padding: 6px 12px;
    font-size: 12px;
    font-weight: 600;
    background-color: transparent;
    border: 1px solid var(--border);
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.2s ease;
    color: var(--text-secondary);
  }

  .action-btn:hover {
    background-color: var(--bg-tertiary);
    border-color: var(--accent);
    color: var(--text-primary);
  }

  .action-btn.primary {
    background-color: var(--accent-glow);
    border-color: var(--accent);
    color: var(--accent);
  }

  .action-btn.primary:hover {
    background-color: var(--accent);
    color: white;
  }

  .action-btn.dismiss {
    color: var(--text-muted);
  }

  .action-btn.dismiss:hover {
    color: var(--error);
    border-color: var(--error);
  }

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }
</style>
