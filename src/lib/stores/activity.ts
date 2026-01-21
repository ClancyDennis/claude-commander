import { writable, derived } from "svelte/store";
import { agents } from "./agents";

export interface AgentActivity {
  agentId: string;
  isProcessing: boolean;
  pendingInput: boolean;
  lastActivity: Date;
  idleTime: number; // milliseconds
}

export const agentActivities = writable<Map<string, AgentActivity>>(new Map());

// Derived store to calculate idle times
export const agentActivityWithIdle = derived(
  [agentActivities],
  ([$activities]) => {
    const now = new Date();
    const result = new Map<string, AgentActivity>();

    $activities.forEach((activity, agentId) => {
      const idleTime = now.getTime() - activity.lastActivity.getTime();
      result.set(agentId, {
        ...activity,
        idleTime,
      });
    });

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
