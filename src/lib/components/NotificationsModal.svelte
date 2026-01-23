<script lang="ts">
  import { get } from "svelte/store";
  import {
    showNotificationsModal,
    closeNotificationsModal,
    activeAlerts,
    alertReadState,
    markAllAlertsRead
  } from "../stores/security";
  import { agents } from "../stores/agents";
  import NotificationItem from "./NotificationItem.svelte";
  import type { SecurityAlert, SecurityAlertSeverity } from "../types";

  let severityFilter = $state<SecurityAlertSeverity | "all">("all");
  let agentFilter = $state<string | "all">("all");
  let timeFilter = $state<"1h" | "24h" | "7d" | "all">("all");

  // Get filtered alerts
  let filteredAlerts = $derived.by(() => {
    let alerts = $activeAlerts;

    // Filter by severity
    if (severityFilter !== "all") {
      alerts = alerts.filter((a) => a.severity === severityFilter);
    }

    // Filter by agent
    if (agentFilter !== "all") {
      alerts = alerts.filter((a) => a.agentId === agentFilter);
    }

    // Filter by time
    if (timeFilter !== "all") {
      const now = new Date();
      const cutoff = new Date();
      if (timeFilter === "1h") {
        cutoff.setHours(now.getHours() - 1);
      } else if (timeFilter === "24h") {
        cutoff.setHours(now.getHours() - 24);
      } else if (timeFilter === "7d") {
        cutoff.setDate(now.getDate() - 7);
      }
      alerts = alerts.filter((a) => a.timestamp >= cutoff);
    }

    return alerts;
  });

  // Get unique agent IDs from alerts
  let agentIds = $derived.by(() => {
    const ids = new Set<string>();
    $activeAlerts.forEach((alert) => {
      if (alert.agentId) {
        ids.add(alert.agentId);
      }
    });
    return Array.from(ids).sort();
  });

  function handleOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      closeNotificationsModal();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      closeNotificationsModal();
    }
  }

  function handleMarkAllRead() {
    markAllAlertsRead();
  }

  function getAgentName(agentId: string): string {
    if (!agentId) return "Unknown";
    const agentsMap = get(agents);
    const agent = agentsMap.get(agentId);
    return agent?.title || agentId.slice(0, 8);
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if $showNotificationsModal}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="overlay animate-fade-in" onclick={handleOverlayClick}>
    <div class="dialog animate-slide-up" role="dialog" aria-modal="true" tabindex="-1">
      <div class="header">
        <h2>Security Notifications</h2>
        <button class="close-btn" onclick={closeNotificationsModal} aria-label="Close dialog">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M6 18L18 6M6 6l12 12" stroke-linecap="round"/>
          </svg>
        </button>
      </div>

      <div class="filters">
        <div class="filter-group">
          <label for="severity-filter">Severity</label>
          <select id="severity-filter" bind:value={severityFilter}>
            <option value="all">All</option>
            <option value="critical">Critical</option>
            <option value="high">High</option>
            <option value="medium">Medium</option>
            <option value="low">Low</option>
          </select>
        </div>

        <div class="filter-group">
          <label for="agent-filter">Agent</label>
          <select id="agent-filter" bind:value={agentFilter}>
            <option value="all">All Agents</option>
            {#each agentIds as id}
              <option value={id}>{getAgentName(id)}</option>
            {/each}
          </select>
        </div>

        <div class="filter-group">
          <label for="time-filter">Time</label>
          <select id="time-filter" bind:value={timeFilter}>
            <option value="all">All Time</option>
            <option value="1h">Last Hour</option>
            <option value="24h">Last 24 Hours</option>
            <option value="7d">Last 7 Days</option>
          </select>
        </div>

        <button class="mark-all-btn" onclick={handleMarkAllRead}>
          Mark All Read
        </button>
      </div>

      <div class="content">
        {#if filteredAlerts.length === 0}
          <div class="empty-state">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
            </svg>
            <p>No security notifications</p>
          </div>
        {:else}
          <div class="notifications-list">
            {#each filteredAlerts as alert (alert.alertId)}
              <NotificationItem
                {alert}
                isRead={$alertReadState.get(alert.alertId) || false}
              />
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.75);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: var(--space-lg);
  }

  .dialog {
    width: 800px;
    max-width: 100%;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    background-color: var(--bg-secondary);
    border-radius: 16px;
    border: 1px solid var(--border);
    box-shadow: var(--shadow-lg);
    overflow: hidden;
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-lg);
    border-bottom: 1px solid var(--border);
  }

  .header h2 {
    font-size: 20px;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
  }

  .close-btn {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: transparent;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    color: var(--text-muted);
    transition: all 0.2s ease;
  }

  .close-btn:hover {
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .close-btn svg {
    width: 18px;
    height: 18px;
  }

  .filters {
    display: flex;
    align-items: flex-end;
    gap: var(--space-md);
    padding: var(--space-md) var(--space-lg);
    border-bottom: 1px solid var(--border);
    background-color: var(--bg-primary);
  }

  .filter-group {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
    flex: 1;
  }

  .filter-group label {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .filter-group select {
    padding: 8px 12px;
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    color: var(--text-primary);
    font-size: 14px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .filter-group select:hover {
    border-color: var(--accent);
  }

  .filter-group select:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent-glow);
  }

  .mark-all-btn {
    padding: 8px 16px;
    font-size: 14px;
    font-weight: 600;
    color: var(--accent);
    background-color: var(--accent-glow);
    border: 1px solid var(--accent);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s ease;
    white-space: nowrap;
  }

  .mark-all-btn:hover {
    background-color: var(--accent);
    color: white;
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-lg);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-md);
    padding: var(--space-xl);
    color: var(--text-muted);
  }

  .empty-state svg {
    width: 64px;
    height: 64px;
    color: var(--success);
  }

  .empty-state p {
    font-size: 16px;
    margin: 0;
  }

  .notifications-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  /* Animations */
  :global(.animate-fade-in) {
    animation: fadeIn 0.2s ease-out;
  }

  :global(.animate-slide-up) {
    animation: slideUp 0.3s ease-out;
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  @keyframes slideUp {
    from {
      opacity: 0;
      transform: translateY(20px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
