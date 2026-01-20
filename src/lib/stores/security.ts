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
