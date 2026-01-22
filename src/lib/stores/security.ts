import { writable, derived } from "svelte/store";
import type { SecurityAlert, PendingSecurityReview } from "../types";

// Map of agent_id -> alerts
export const securityAlerts = writable<Map<string, SecurityAlert[]>>(new Map());

// Set of terminated agent IDs
export const terminatedAgents = writable<Set<string>>(new Set());

// Set of suspended agent IDs
export const suspendedAgents = writable<Set<string>>(new Set());

// Map of review_id -> pending review
export const pendingReviews = writable<Map<string, PendingSecurityReview>>(new Map());

// Derived: agents with active alerts
export const agentsWithAlerts = derived(securityAlerts, ($alerts) => {
  return new Set($alerts.keys());
});

// Derived: total alert count
export const totalAlertCount = derived(securityAlerts, ($alerts) => {
  let count = 0;
  $alerts.forEach((alerts) => {
    count += alerts.length;
  });
  return count;
});

export function addSecurityAlert(alert: SecurityAlert) {
  securityAlerts.update((map) => {
    const existing = map.get(alert.agentId) ?? [];
    map.set(alert.agentId, [...existing, alert]);
    return new Map(map);
  });
}

export function clearAlertsForAgent(agentId: string) {
  securityAlerts.update((map) => {
    map.delete(agentId);
    return new Map(map);
  });
}

export function clearAllAlerts() {
  securityAlerts.set(new Map());
}

export function getAlertsForAgent(agentId: string): SecurityAlert[] {
  let alerts: SecurityAlert[] = [];
  securityAlerts.subscribe((map) => {
    alerts = map.get(agentId) ?? [];
  })();
  return alerts;
}

// Derived: total pending reviews count
export const pendingReviewCount = derived(pendingReviews, ($reviews) => {
  return $reviews.size;
});

// Derived: agents that are stopped (terminated or suspended)
export const stoppedAgents = derived(
  [terminatedAgents, suspendedAgents],
  ([$terminated, $suspended]) => {
    return new Set([...$terminated, ...$suspended]);
  }
);

// Mark agent as terminated
export function markAgentTerminated(agentId: string) {
  terminatedAgents.update((set) => {
    set.add(agentId);
    return new Set(set);
  });
}

// Mark agent as suspended
export function markAgentSuspended(agentId: string) {
  suspendedAgents.update((set) => {
    set.add(agentId);
    return new Set(set);
  });
}

// Resume a suspended agent (remove from suspended set)
export function resumeAgent(agentId: string) {
  suspendedAgents.update((set) => {
    set.delete(agentId);
    return new Set(set);
  });
}

// Add a pending review
export function addPendingReview(review: PendingSecurityReview) {
  pendingReviews.update((map) => {
    map.set(review.id, review);
    return new Map(map);
  });
}

// Remove a pending review (after it's been handled)
export function removePendingReview(reviewId: string) {
  pendingReviews.update((map) => {
    map.delete(reviewId);
    return new Map(map);
  });
}

// Check if agent is terminated
export function isAgentTerminated(agentId: string): boolean {
  let result = false;
  terminatedAgents.subscribe((set) => {
    result = set.has(agentId);
  })();
  return result;
}

// Check if agent is suspended
export function isAgentSuspended(agentId: string): boolean {
  let result = false;
  suspendedAgents.subscribe((set) => {
    result = set.has(agentId);
  })();
  return result;
}

// Store for selected alert detail (for modal display)
export const selectedAlertDetail = writable<SecurityAlert | null>(null);

// Show alert detail modal
export function showAlertDetail(alert: SecurityAlert) {
  selectedAlertDetail.set(alert);
}

// Hide alert detail modal
export function hideAlertDetail() {
  selectedAlertDetail.set(null);
}

// ========== Notifications Area State ==========

// Track read state per alert (Map of alertId -> read status)
export const alertReadState = writable<Map<string, boolean>>(new Map());

// Track dismissed alerts (Set of alertId)
export const dismissedAlerts = writable<Set<string>>(new Set());

// Controls notifications modal visibility
export const showNotificationsModal = writable<boolean>(false);

// Derived: All alerts flattened into a single sorted array (newest first)
export const allAlertsSorted = derived(securityAlerts, ($alerts) => {
  const all: SecurityAlert[] = [];
  $alerts.forEach((agentAlerts) => all.push(...agentAlerts));
  return all.sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime());
});

// Derived: Non-dismissed alerts only
export const activeAlerts = derived(
  [allAlertsSorted, dismissedAlerts],
  ([$all, $dismissed]) => $all.filter((a) => !$dismissed.has(a.alertId))
);

// Derived: Unread count (non-dismissed, non-read)
export const unreadAlertCount = derived(
  [activeAlerts, alertReadState],
  ([$active, $readState]) =>
    $active.filter((a) => !$readState.get(a.alertId)).length
);

// Derived: Highest severity among unread alerts
export const highestUnreadSeverity = derived(
  [activeAlerts, alertReadState],
  ([$active, $readState]) => {
    const unread = $active.filter((a) => !$readState.get(a.alertId));
    const severityOrder = ["critical", "high", "medium", "low"];
    for (const severity of severityOrder) {
      if (unread.some((a) => a.severity === severity)) {
        return severity as SecurityAlert["severity"];
      }
    }
    return null;
  }
);

// Mark single alert as read
export function markAlertRead(alertId: string) {
  alertReadState.update((map) => {
    map.set(alertId, true);
    return new Map(map);
  });
}

// Mark all alerts as read
export function markAllAlertsRead() {
  let alerts: SecurityAlert[] = [];
  activeAlerts.subscribe((a) => (alerts = a))();
  alertReadState.update((map) => {
    alerts.forEach((a) => map.set(a.alertId, true));
    return new Map(map);
  });
}

// Dismiss an alert (hide from list, not delete)
export function dismissAlert(alertId: string) {
  dismissedAlerts.update((set) => {
    set.add(alertId);
    return new Set(set);
  });
}

// Clear all dismissed alerts (restore them)
export function clearDismissedAlerts() {
  dismissedAlerts.set(new Set());
}

// Open notifications modal
export function openNotificationsModal() {
  showNotificationsModal.set(true);
}

// Close notifications modal
export function closeNotificationsModal() {
  showNotificationsModal.set(false);
}
