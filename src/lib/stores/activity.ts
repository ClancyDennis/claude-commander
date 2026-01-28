import { writable, derived } from "svelte/store";
import { agents } from "./agents";

export interface AgentActivity {
  agentId: string;
  isProcessing: boolean;
  pendingInput: boolean;
  lastActivity: Date;
  idleTime: number; // milliseconds
  currentActivity?: string; // Human-readable activity like "Reading src/main.rs..."
  currentToolName?: string; // Tool being used
}

export const agentActivities = writable<Map<string, AgentActivity>>(new Map());

// Memoization cache for agentActivityWithIdle
let lastActivitiesRef: Map<string, AgentActivity> | null = null;
let lastIdleResult: Map<string, AgentActivity> | null = null;

// Derived store to calculate idle times (with memoization)
export const agentActivityWithIdle = derived(
  [agentActivities],
  ([$activities]) => {
    // Skip recalculation if same reference (common case during rapid updates)
    if ($activities === lastActivitiesRef && lastIdleResult) {
      return lastIdleResult;
    }
    lastActivitiesRef = $activities;

    const now = Date.now();
    const result = new Map<string, AgentActivity>();

    $activities.forEach((activity, agentId) => {
      const idleTime = now - activity.lastActivity.getTime();
      result.set(agentId, {
        ...activity,
        idleTime,
      });
    });

    lastIdleResult = result;
    return result;
  }
);

export function updateActivity(agentId: string, activity: Partial<AgentActivity>) {
  agentActivities.update((map) => {
    const existing = map.get(agentId) || {
      agentId,
      isProcessing: false,
      pendingInput: false,
      lastActivity: new Date(),
      idleTime: 0,
    };

    map.set(agentId, {
      ...existing,
      ...activity,
      lastActivity: activity.lastActivity || new Date(),
    });

    return new Map(map);
  });
}

export function clearActivity(agentId: string) {
  agentActivities.update((map) => {
    map.delete(agentId);
    return new Map(map);
  });
}

/**
 * Update the current human-readable activity for an agent
 * (e.g., "Reading src/main.rs...", "Running: npm install")
 */
export function updateActivityDetail(
  agentId: string,
  detail: { activity: string; toolName: string; timestamp: Date }
) {
  agentActivities.update((map) => {
    const existing = map.get(agentId) || {
      agentId,
      isProcessing: true,
      pendingInput: false,
      lastActivity: detail.timestamp,
      idleTime: 0,
    };

    map.set(agentId, {
      ...existing,
      currentActivity: detail.activity,
      currentToolName: detail.toolName,
      lastActivity: detail.timestamp,
      isProcessing: true, // If we're getting activity updates, the agent is processing
    });

    return new Map(map);
  });
}
