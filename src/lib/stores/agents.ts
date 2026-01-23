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
  AgentRun
} from "../types";
import { selectedAutoPipelineId } from "./autoPipelines";

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

// ============================================================================
// History Mode Stores
// ============================================================================

export const sidebarMode = writable<'running' | 'history'>('running');
export const historicalRuns = writable<AgentRun[]>([]);
export const selectedHistoricalRun = writable<AgentRun | null>(null);

// ============================================================================
// Chat-Related Stores (Meta-Agent)
// ============================================================================

export const metaAgentChat = writable<ChatMessage[]>([]);
export const viewMode = writable<'agent' | 'chat'>('agent');
export const metaAgentThinking = writable<boolean>(false);
export const metaAgentToolCalls = writable<MetaAgentToolCallEvent[]>([]);

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

export const agentsWithOutputs = derived(
  [agents, agentOutputs],
  ([$agents, $agentOutputs]) => {
    const agentsArray: Array<{id: string; workingDir: string; outputCount: number}> = [];

    $agentOutputs.forEach((outputs, agentId) => {
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

    return agentsArray;
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
}

// ============================================================================
// Agent Output Functions
// ============================================================================

export function appendOutput(agentId: string, output: AgentOutput) {
  console.log("[Store] appendOutput called for agent:", agentId, "output:", output);
  agentOutputs.update((map) => {
    const outputs = map.get(agentId) ?? [];
    console.log("[Store] Current outputs for agent:", outputs.length);
    // Create a new array to trigger reactivity
    const newOutputs = [...outputs, output];
    const newMap = new Map(map);
    newMap.set(agentId, newOutputs);
    console.log("[Store] New output count:", newOutputs.length);
    return newMap;
  });

  // Update unread count if agent not currently viewed
  const currentViewed = get(viewedAgents);
  if (!currentViewed.has(agentId)) {
    agents.update((map) => {
      const agent = map.get(agentId);
      if (agent) {
        const unreadCount = (agent.unreadOutputs ?? 0) + 1;
        map.set(agentId, { ...agent, unreadOutputs: unreadCount });
      }
      return new Map(map);
    });
  }

  // Update last activity
  updateAgentActivity(agentId, {
    lastActivity: new Date(),
    isProcessing: false,
  });
}

export function appendToolEvent(agentId: string, event: ToolEvent) {
  toolEvents.update((map) => {
    const events = map.get(agentId) ?? [];
    // Create a new array to trigger reactivity
    const newEvents = [...events, event];
    const newMap = new Map(map);
    newMap.set(agentId, newEvents);
    return newMap;
  });
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
}

export function openAgent(agentId: string, multiSelect: boolean = false) {
  viewMode.set('agent');
  selectedAgentId.set(agentId);
  selectedAutoPipelineId.set(null);
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
}
