<script lang="ts">
  import { unreadAlertCount, highestUnreadSeverity, openNotificationsModal } from "../stores/security";

  function getBadgeColor(severity: string | null): string {
    if (!severity) return "var(--accent)";
    switch (severity) {
      case "critical":
      case "high":
        return "var(--error)";
      case "medium":
        return "var(--warning)";
      case "low":
        return "var(--accent)";
      default:
        return "var(--accent)";
    }
  }

  function handleClick() {
    openNotificationsModal();
  }
</script>

<button class="notification-bell" onclick={handleClick} title="Security Notifications">
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
    <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/>
    <path d="M13.73 21a2 2 0 0 1-3.46 0"/>
  </svg>
  {#if $unreadAlertCount > 0}
    <span class="badge" class:pulse={$unreadAlertCount > 0} style="background-color: {getBadgeColor($highestUnreadSeverity)}">
      {$unreadAlertCount > 99 ? '99+' : $unreadAlertCount}
    </span>
  {/if}
</button>

<style>
  .notification-bell {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0.75rem;
    border: 1px solid var(--border);
    background: var(--bg-secondary);
    color: var(--text-primary);
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .notification-bell:hover {
    background: var(--bg-tertiary);
    border-color: var(--accent);
  }

  .notification-bell svg {
    width: 16px;
    height: 16px;
  }

  .badge {
    position: absolute;
    top: -4px;
    right: -4px;
    min-width: 18px;
    height: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0 4px;
    border-radius: 9px;
    font-size: 11px;
    font-weight: 700;
    color: white;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  }

  .badge.pulse {
    animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
  }

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.7;
    }
  }
</style>
