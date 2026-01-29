/**
 * Agent Store Module
 *
 * Core agent CRUD operations, store management, and lifecycle functions.
 * Orchestrator functionality has been extracted to ./orchestrator.ts
 */

import { writable, derived, get } from "svelte/store";
import type {
  Agent,
  AgentOutput,
  ToolEvent,
  ChatMessage,
  MetaAgentToolCallEvent,
  AgentStatistics,
  AgentRun,
  UnifiedHistoryItem,
  MetaConversation,
  ContextInfoEvent
} from "../types";
import { selectedAutoPipelineId } from "./autoPipelines";
import { selectedPipelineId } from "./pipelines";

// Re-export all orchestrator functionality for backward compatibility
export {
  // Stores
  orchestratorToolCalls,
  orchestratorStateChanges,
  orchestratorDecisions,
  orchestratorCurrentState,
  historyLoadState,
  stepToolCounts,
  // Functions
  setCurrentPipelineId,
  getCurrentPipelineId,
  addOrchestratorToolCall,
  completeOrchestratorToolCall,
  addOrchestratorStateChange,
  addOrchestratorDecision,
  incrementStepToolCount,
  clearStepToolCounts,
  loadPipelineHistory,
  clearOrchestratorActivity,
  flushOrchestratorUpdates,
} from "./orchestrator";

// ============================================================================
// Core Agent Stores
// ============================================================================

export const agents = writable<Map<string, Agent>>(new Map());
export const selectedAgentId = writable<string | null>(null);
export const selectedAgentIds = writable<Set<string>>(new Set());
export const layoutMode = writable<'single' | 'split' | 'grid'>('single');
export const gridSize = writable<number>(2);
export const agentOutputs = writable<Map<string, AgentOutput[]>>(new Map());
export const toolEvents = writable<Map<string, ToolEvent[]>>(new Map());
export const viewedAgents = writable<Set<string>>(new Set());
export const agentStats = writable<Map<string, AgentStatistics>>(new Map());

// Cache viewedAgents to avoid get() calls in hot paths
let cachedViewedAgents: Set<string> = new Set();
viewedAgents.subscribe(v => cachedViewedAgents = v);

// ============================================================================
// History Mode Stores
// ============================================================================

export const sidebarMode = writable<'running' | 'history'>('running');
export const historicalRuns = writable<AgentRun[]>([]);
export const selectedHistoricalRun = writable<AgentRun | null>(null);

// Store for pre-filling agent input when resuming without auto-start
export const pendingAgentPrompt = writable<{ agentId: string; prompt: string } | null>(null);

// ============================================================================
// Chat-Related Stores (Meta-Agent)
// ============================================================================

export const metaAgentChat = writable<ChatMessage[]>([]);
export const viewMode = writable<'agent' | 'chat'>('agent');
export const metaAgentThinking = writable<boolean>(false);
export const metaAgentToolCalls = writable<MetaAgentToolCallEvent[]>([]);
export const metaAgentContextInfo = writable<ContextInfoEvent | null>(null);

// Track dismissed agent results (ephemeral - clears on page refresh)
export const dismissedAgentResults = writable<Set<string>>(new Set());

// ============================================================================
// Derived Stores
// ============================================================================

export const selectedAgent = derived(
  [agents, selectedAgentId],
  ([$agents, $selectedAgentId]) => {
    if (!$selectedAgentId) return null;
    return $agents.get($selectedAgentId) ?? null;
  }
);

export const selectedAgentOutputs = derived(
  [agentOutputs, selectedAgentId],
  ([$agentOutputs, $selectedAgentId]) => {
    if (!$selectedAgentId) return [];
    return $agentOutputs.get($selectedAgentId) ?? [];
  }
);

export const selectedAgentTools = derived(
  [toolEvents, selectedAgentId],
  ([$toolEvents, $selectedAgentId]) => {
    if (!$selectedAgentId) return [];
    return $toolEvents.get($selectedAgentId) ?? [];
  }
);

export const selectedAgentStats = derived(
  [agentStats, selectedAgentId],
  ([$agentStats, $selectedAgentId]) => {
    if (!$selectedAgentId) return null;
    return $agentStats.get($selectedAgentId) ?? null;
  }
);

// Memoized agentsWithOutputs to prevent unnecessary re-renders
let lastAgentsWithOutputs: Array<{id: string; workingDir: string; outputCount: number}> = [];

export const agentsWithOutputs = derived(
  [agents, agentOutputs, dismissedAgentResults],
  ([$agents, $agentOutputs, $dismissedAgentResults]) => {
    const agentsArray: Array<{id: string; workingDir: string; outputCount: number}> = [];

    $agentOutputs.forEach((outputs, agentId) => {
      // Skip dismissed agents
      if ($dismissedAgentResults.has(agentId)) return;

      if (outputs.length > 0) {
        const agent = $agents.get(agentId);
        if (agent) {
          agentsArray.push({
            id: agent.id,
            workingDir: agent.workingDir,
            outputCount: outputs.length
          });
        }
      }
    });

    // Only return a new array if the data actually changed (structural comparison)
    if (agentsArray.length !== lastAgentsWithOutputs.length) {
      lastAgentsWithOutputs = agentsArray;
      return agentsArray;
    }

    // Check if any item changed
    const hasChanged = agentsArray.some((item, i) => {
      const prev = lastAgentsWithOutputs[i];
      return !prev || item.id !== prev.id || item.outputCount !== prev.outputCount;
    });

    if (hasChanged) {
      lastAgentsWithOutputs = agentsArray;
      return agentsArray;
    }

    // Return the same reference to prevent reactivity triggers
    return lastAgentsWithOutputs;
  }
);

// ============================================================================
// Agent CRUD Operations
// ============================================================================

export function addAgent(agent: Agent) {
  agents.update((map) => {
    map.set(agent.id, agent);
    return new Map(map);
  });
  agentOutputs.update((map) => {
    map.set(agent.id, []);
    return new Map(map);
  });
  toolEvents.update((map) => {
    map.set(agent.id, []);
    return new Map(map);
  });
}

export function updateAgentStatus(agentId: string, status: Agent["status"]) {
  agents.update((map) => {
    const agent = map.get(agentId);
    if (agent) {
      map.set(agentId, { ...agent, status });
    }
    return new Map(map);
  });
}

export function removeAgent(agentId: string) {
  agents.update((map) => {
    map.delete(agentId);
    return new Map(map);
  });
  agentOutputs.update((map) => {
    map.delete(agentId);
    return new Map(map);
  });
  toolEvents.update((map) => {
    map.delete(agentId);
    return new Map(map);
  });
  selectedAgentId.update((current) => (current === agentId ? null : current));

  // Clean up pending maps to prevent memory leaks
  pendingOutputs.delete(agentId);
  pendingToolEvents.delete(agentId);
  pendingUnreadCounts.delete(agentId);
  pendingActivityUpdates.delete(agentId);
}

// ============================================================================
// Agent Output Functions (with debouncing to prevent UI lag)
// ============================================================================

const OUTPUT_DEBOUNCE_MS = 150;

// Maximum outputs/events per agent to prevent unbounded memory growth
const MAX_OUTPUTS_PER_AGENT = 2000;
const MAX_TOOL_EVENTS_PER_AGENT = 1000;

// Pending outputs and events to be batched
const pendingOutputs: Map<string, AgentOutput[]> = new Map();
const pendingToolEvents: Map<string, ToolEvent[]> = new Map();
const pendingUnreadCounts: Map<string, number> = new Map();
const pendingActivityUpdates: Map<string, { lastActivity: Date; isProcessing: boolean }> = new Map();

let outputFlushTimer: ReturnType<typeof setTimeout> | null = null;
let toolEventFlushTimer: ReturnType<typeof setTimeout> | null = null;

function flushOutputs() {
  if (pendingOutputs.size === 0 && pendingUnreadCounts.size === 0 && pendingActivityUpdates.size === 0) return;

  // Batch update agentOutputs with bounded size
  if (pendingOutputs.size > 0) {
    agentOutputs.update((map) => {
      const newMap = new Map(map);
      for (const [agentId, outputs] of pendingOutputs) {
        const existing = newMap.get(agentId) ?? [];
        const combined = [...existing, ...outputs];
        // Cap array size to prevent unbounded memory growth
        if (combined.length > MAX_OUTPUTS_PER_AGENT) {
          combined.splice(0, combined.length - MAX_OUTPUTS_PER_AGENT);
        }
        newMap.set(agentId, combined);
      }
      return newMap;
    });
    pendingOutputs.clear();
  }

  // Batch update agents (unread counts + activity)
  if (pendingUnreadCounts.size > 0 || pendingActivityUpdates.size > 0) {
    agents.update((map) => {
      const newMap = new Map(map);

      // Apply unread count updates
      for (const [agentId, count] of pendingUnreadCounts) {
        const agent = newMap.get(agentId);
        if (agent) {
          const currentUnread = agent.unreadOutputs ?? 0;
          newMap.set(agentId, { ...agent, unreadOutputs: currentUnread + count });
        }
      }

      // Apply activity updates
      for (const [agentId, activity] of pendingActivityUpdates) {
        const agent = newMap.get(agentId);
        if (agent) {
          newMap.set(agentId, { ...agent, ...activity });
        }
      }

      return newMap;
    });
    pendingUnreadCounts.clear();
    pendingActivityUpdates.clear();
  }
}

function flushToolEvents() {
  if (pendingToolEvents.size === 0) return;

  toolEvents.update((map) => {
    const newMap = new Map(map);
    for (const [agentId, events] of pendingToolEvents) {
      const existing = newMap.get(agentId) ?? [];
      const combined = [...existing, ...events];
      // Cap array size to prevent unbounded memory growth
      if (combined.length > MAX_TOOL_EVENTS_PER_AGENT) {
        combined.splice(0, combined.length - MAX_TOOL_EVENTS_PER_AGENT);
      }
      newMap.set(agentId, combined);
    }
    return newMap;
  });
  pendingToolEvents.clear();
}

export function appendOutput(agentId: string, output: AgentOutput) {
  // Queue output for batching
  const existing = pendingOutputs.get(agentId) ?? [];
  existing.push(output);
  pendingOutputs.set(agentId, existing);

  // Queue unread count update if agent not currently viewed (use cached value)
  if (!cachedViewedAgents.has(agentId)) {
    const currentPending = pendingUnreadCounts.get(agentId) ?? 0;
    pendingUnreadCounts.set(agentId, currentPending + 1);
  }

  // Queue activity update (reuse timestamp from output if available)
  const activityDate = output.timestamp instanceof Date ? output.timestamp : new Date();
  pendingActivityUpdates.set(agentId, {
    lastActivity: activityDate,
    isProcessing: false,
  });

  // Debounce the flush
  if (outputFlushTimer) {
    clearTimeout(outputFlushTimer);
  }
  outputFlushTimer = setTimeout(() => {
    outputFlushTimer = null;
    flushOutputs();
  }, OUTPUT_DEBOUNCE_MS);
}

export function appendToolEvent(agentId: string, event: ToolEvent) {
  // Queue event for batching
  const existing = pendingToolEvents.get(agentId) ?? [];
  existing.push(event);
  pendingToolEvents.set(agentId, existing);

  // Debounce the flush
  if (toolEventFlushTimer) {
    clearTimeout(toolEventFlushTimer);
  }
  toolEventFlushTimer = setTimeout(() => {
    toolEventFlushTimer = null;
    flushToolEvents();
  }, OUTPUT_DEBOUNCE_MS);
}

/**
 * Force flush all pending agent updates (useful before cleanup or sync operations)
 */
export function flushAgentUpdates() {
  if (outputFlushTimer) {
    clearTimeout(outputFlushTimer);
    outputFlushTimer = null;
  }
  if (toolEventFlushTimer) {
    clearTimeout(toolEventFlushTimer);
    toolEventFlushTimer = null;
  }
  flushOutputs();
  flushToolEvents();
}

export function clearAgentOutput(agentId: string) {
  agentOutputs.update((map) => {
    map.set(agentId, []);
    return new Map(map);
  });
}

// ============================================================================
// Agent Activity & State Functions
// ============================================================================

export function updateAgentActivity(
  agentId: string,
  activity: {
    lastActivity?: Date;
    isProcessing?: boolean;
    pendingInput?: boolean;
  }
) {
  agents.update((map) => {
    const agent = map.get(agentId);
    if (agent) {
      map.set(agentId, { ...agent, ...activity });
    }
    return new Map(map);
  });
}

export function markAgentViewed(agentId: string) {
  viewedAgents.update((set) => {
    set.add(agentId);
    return new Set(set);
  });

  // Clear unread count
  agents.update((map) => {
    const agent = map.get(agentId);
    if (agent) {
      map.set(agentId, { ...agent, unreadOutputs: 0 });
    }
    return new Map(map);
  });
}

export function updateAgentStats(agentId: string, stats: AgentStatistics) {
  agentStats.update((map) => {
    map.set(agentId, stats);
    return new Map(map);
  });
}

// ============================================================================
// Layout & Selection Functions
// ============================================================================

export function toggleLayoutMode() {
  layoutMode.update((current) => {
    if (current === 'single') return 'split';
    if (current === 'split') return 'grid';
    return 'single';
  });
}

export function selectMultipleAgents(agentIds: string[]) {
  selectedAgentIds.update(() => new Set(agentIds));
}

/**
 * Toggle agent in multi-select set (for Ctrl/Cmd+click)
 */
export function toggleAgentInSelection(agentId: string) {
  selectedAgentIds.update((set) => {
    const newSet = new Set(set);
    if (newSet.has(agentId)) {
      newSet.delete(agentId);
    } else {
      newSet.add(agentId);
    }
    return newSet;
  });

  // Also update selectedAgentId to the most recent selection for single view compat
  const currentIds = get(selectedAgentIds);
  if (currentIds.size > 0) {
    const idsArray = Array.from(currentIds);
    selectedAgentId.set(idsArray[idsArray.length - 1]);
  } else {
    selectedAgentId.set(null);
  }
}

/**
 * Add agent to multi-select without removing others
 */
export function addAgentToSelection(agentId: string) {
  selectedAgentIds.update((set) => {
    const newSet = new Set(set);
    newSet.add(agentId);
    return newSet;
  });
}

// ============================================================================
// View Mode & Chat Functions
// ============================================================================

export function openChat() {
  viewMode.set('chat');
  selectedAgentId.set(null);
  selectedAutoPipelineId.set(null);
  selectedPipelineId.set(null);
  selectedHistoricalRun.set(null);
}

export function openAgent(agentId: string, multiSelect: boolean = false) {
  viewMode.set('agent');
  selectedAgentId.set(agentId);
  selectedAutoPipelineId.set(null);
  selectedPipelineId.set(null);
  selectedHistoricalRun.set(null);
  markAgentViewed(agentId);

  // When not multi-selecting, reset selectedAgentIds to just this agent
  // When multi-selecting, add to the set
  if (multiSelect) {
    selectedAgentIds.update((set) => {
      const newSet = new Set(set);
      newSet.add(agentId);
      return newSet;
    });
  } else {
    // Single select: replace the set with just this agent
    selectedAgentIds.set(new Set([agentId]));
  }
}

export function addChatMessage(message: ChatMessage) {
  metaAgentChat.update((messages) => [...messages, message]);
}

export function clearChatHistory() {
  metaAgentChat.set([]);
}

export function addMetaAgentToolCall(toolCall: MetaAgentToolCallEvent) {
  metaAgentToolCalls.update((calls) => [...calls, toolCall]);
}

/**
 * Dismiss an agent's results from the "results available" notification.
 * The agent data remains accessible, but the notification is hidden.
 */
export function dismissAgentResult(agentId: string) {
  dismissedAgentResults.update((set) => {
    const newSet = new Set(set);
    newSet.add(agentId);
    return newSet;
  });
}

/**
 * Clear all dismissed agent results (e.g., when starting fresh)
 */
export function clearDismissedResults() {
  dismissedAgentResults.set(new Set());
}

// ============================================================================
// History Mode Functions
// ============================================================================

export function toggleSidebarMode() {
  sidebarMode.update((current) => current === 'running' ? 'history' : 'running');
}

export function setSidebarMode(mode: 'running' | 'history') {
  sidebarMode.set(mode);
}

export function setHistoricalRuns(runs: AgentRun[]) {
  historicalRuns.set(runs);
}

export function selectHistoricalRun(run: AgentRun | null) {
  selectedHistoricalRun.set(run);
  if (run) {
    selectedAutoPipelineId.set(null);
    selectedPipelineId.set(null);
    sidebarMode.set('history');
  }
}

/**
 * Reconcile stale runs - marks any "running" agents in the database as "crashed"
 * This is useful for cleaning up orphaned agents from previous sessions
 * Returns the number of runs that were reconciled
 */
export async function reconcileStaleRuns(): Promise<number> {
  const { invoke } = await import("@tauri-apps/api/core");
  const count = await invoke<number>("reconcile_stale_runs");
  // Reload historical runs after reconciliation
  const runs = await invoke<AgentRun[]>("get_all_runs");
  setHistoricalRuns(runs);
  return count;
}
