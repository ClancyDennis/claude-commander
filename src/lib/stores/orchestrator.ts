/**
 * Orchestrator Activity Store Module
 *
 * Handles orchestrator state management with hybrid persistence:
 * - CircularBuffer for bounded memory + async SQLite persistence for durability
 * - Recent events kept in memory for instant UI updates
 * - All events persisted to SQLite for historical queries on page reload
 */

import { writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import type {
  OrchestratorToolCall,
  OrchestratorStateChange,
  OrchestratorDecision
} from "../types";
import { CircularBuffer, BUFFER_SIZES } from "./circularBuffer";

// ============================================================================
// Circular Buffers for Memory-Bounded Storage
// ============================================================================

const toolCallsBuffer = new CircularBuffer<OrchestratorToolCall>(BUFFER_SIZES.toolCalls);
const stateChangesBuffer = new CircularBuffer<OrchestratorStateChange>(BUFFER_SIZES.stateChanges);
const decisionsBuffer = new CircularBuffer<OrchestratorDecision>(BUFFER_SIZES.decisions);

// ============================================================================
// Reactive Stores
// ============================================================================

export const orchestratorToolCalls = writable<OrchestratorToolCall[]>([]);
export const orchestratorStateChanges = writable<OrchestratorStateChange[]>([]);
export const orchestratorDecisions = writable<OrchestratorDecision[]>([]);
export const orchestratorCurrentState = writable<string>("Idle");

// Track history loading state
export const historyLoadState = writable<{
  isLoading: boolean;
  loadedPipelines: Set<string>;
}>({
  isLoading: false,
  loadedPipelines: new Set(),
});

// Real-time step tool counts (1=Planning, 2=Building, 3=Verifying)
export const stepToolCounts = writable<Map<number, number>>(new Map());

// ============================================================================
// Pipeline ID Management
// ============================================================================

let currentPipelineId: string | null = null;

export function setCurrentPipelineId(pipelineId: string | null) {
  currentPipelineId = pipelineId;
}

export function getCurrentPipelineId(): string | null {
  return currentPipelineId;
}

// ============================================================================
// Debouncing Mechanism for Batching Rapid Updates
// ============================================================================

const DEBOUNCE_MS = 100;

let pendingToolCalls: OrchestratorToolCall[] = [];
let pendingStateChanges: OrchestratorStateChange[] = [];
let pendingDecisions: OrchestratorDecision[] = [];

let toolCallTimer: ReturnType<typeof setTimeout> | null = null;
let stateChangeTimer: ReturnType<typeof setTimeout> | null = null;
let decisionTimer: ReturnType<typeof setTimeout> | null = null;

function flushToolCalls() {
  if (pendingToolCalls.length === 0) return;

  const toAdd = pendingToolCalls;
  pendingToolCalls = [];

  // Add to circular buffer
  for (const tc of toAdd) {
    toolCallsBuffer.push(tc);
  }

  // Update reactive store from buffer
  orchestratorToolCalls.set(toolCallsBuffer.toArray());
  console.log('[Orchestrator] toolCalls batch update:', toAdd.length, 'items, total:', toolCallsBuffer.length);
}

function flushStateChanges() {
  if (pendingStateChanges.length === 0) return;

  const toAdd = pendingStateChanges;
  pendingStateChanges = [];

  for (const sc of toAdd) {
    stateChangesBuffer.push(sc);
  }

  orchestratorStateChanges.set(stateChangesBuffer.toArray());
  console.log('[Orchestrator] stateChanges batch update:', toAdd.length, 'items, total:', stateChangesBuffer.length);

  // Update current state to the latest
  if (toAdd.length > 0) {
    orchestratorCurrentState.set(toAdd[toAdd.length - 1].new_state);
  }
}

function flushDecisions() {
  if (pendingDecisions.length === 0) return;

  const toAdd = pendingDecisions;
  pendingDecisions = [];

  for (const d of toAdd) {
    decisionsBuffer.push(d);
  }

  orchestratorDecisions.set(decisionsBuffer.toArray());
  console.log('[Orchestrator] decisions batch update:', toAdd.length, 'items, total:', decisionsBuffer.length);
}

// ============================================================================
// Async Persistence (fire-and-forget)
// ============================================================================

async function persistToolCall(toolCall: OrchestratorToolCall) {
  if (!currentPipelineId) return;

  try {
    await invoke('persist_tool_call', {
      toolCall: {
        id: null,
        pipeline_id: currentPipelineId,
        agent_id: null,
        tool_name: toolCall.tool_name,
        tool_input: toolCall.tool_input ? JSON.stringify(toolCall.tool_input) : null,
        is_error: toolCall.is_error ?? false,
        summary: toolCall.summary ?? null,
        current_state: toolCall.current_state,
        iteration: toolCall.iteration,
        step_number: null,
        timestamp: toolCall.timestamp,
      }
    });
  } catch (e) {
    console.warn('[Orchestrator] Failed to persist tool call:', e);
  }
}

async function persistStateChange(stateChange: OrchestratorStateChange) {
  console.log('[Orchestrator] persistStateChange called, currentPipelineId:', currentPipelineId);
  if (!currentPipelineId) {
    console.warn('[Orchestrator] Skipping state change persistence - no currentPipelineId set');
    return;
  }

  try {
    console.log('[Orchestrator] Persisting state change:', stateChange);
    await invoke('persist_state_change', {
      stateChange: {
        id: null,
        pipeline_id: currentPipelineId,
        old_state: stateChange.old_state,
        new_state: stateChange.new_state,
        iteration: stateChange.iteration,
        generated_skills: stateChange.generated_skills,
        generated_subagents: stateChange.generated_subagents,
        claudemd_generated: stateChange.claudemd_generated,
        timestamp: stateChange.timestamp,
      }
    });
    console.log('[Orchestrator] Successfully persisted state change');
  } catch (e) {
    console.error('[Orchestrator] Failed to persist state change:', e);
  }
}

async function persistDecision(decision: OrchestratorDecision) {
  console.log('[Orchestrator] persistDecision called, currentPipelineId:', currentPipelineId);
  if (!currentPipelineId) {
    console.warn('[Orchestrator] Skipping decision persistence - no currentPipelineId set');
    return;
  }

  try {
    console.log('[Orchestrator] Persisting decision:', decision);
    await invoke('persist_decision', {
      decision: {
        id: null,
        pipeline_id: currentPipelineId,
        decision: decision.decision,
        reasoning: decision.reasoning ?? null,
        issues: decision.issues,
        suggestions: decision.suggestions,
        timestamp: decision.timestamp,
      }
    });
  } catch (e) {
    console.warn('[Orchestrator] Failed to persist decision:', e);
  }
}

// ============================================================================
// Public Add Functions (with debouncing + persistence)
// ============================================================================

export function addOrchestratorToolCall(toolCall: OrchestratorToolCall) {
  pendingToolCalls.push(toolCall);

  // Fire-and-forget persistence
  persistToolCall(toolCall);

  if (toolCallTimer) {
    clearTimeout(toolCallTimer);
  }

  toolCallTimer = setTimeout(() => {
    toolCallTimer = null;
    flushToolCalls();
  }, DEBOUNCE_MS);
}

/**
 * Mark the most recent tool call with this name as completed
 */
export function completeOrchestratorToolCall(toolName: string, isError: boolean) {
  orchestratorToolCalls.update(calls => {
    // Find the most recent uncompleted call with this tool name
    for (let i = calls.length - 1; i >= 0; i--) {
      if (calls[i].tool_name === toolName && !calls[i].completed) {
        calls[i].completed = true;
        calls[i].is_error = isError;
        calls[i].completion_time = Date.now();
        break;
      }
    }
    return calls;
  });
}

export function addOrchestratorStateChange(stateChange: OrchestratorStateChange) {
  pendingStateChanges.push(stateChange);
  // Update current state immediately for responsiveness
  orchestratorCurrentState.set(stateChange.new_state);

  // Fire-and-forget persistence
  persistStateChange(stateChange);

  if (stateChangeTimer) {
    clearTimeout(stateChangeTimer);
  }

  stateChangeTimer = setTimeout(() => {
    stateChangeTimer = null;
    flushStateChanges();
  }, DEBOUNCE_MS);
}

export function addOrchestratorDecision(decision: OrchestratorDecision) {
  pendingDecisions.push(decision);

  // Fire-and-forget persistence
  persistDecision(decision);

  if (decisionTimer) {
    clearTimeout(decisionTimer);
  }

  decisionTimer = setTimeout(() => {
    decisionTimer = null;
    flushDecisions();
  }, DEBOUNCE_MS);
}

// ============================================================================
// Step Tool Count Functions
// ============================================================================

export function incrementStepToolCount(stepNumber: number) {
  stepToolCounts.update(m => {
    const current = m.get(stepNumber) || 0;
    m.set(stepNumber, current + 1);
    return new Map(m);
  });
}

export function clearStepToolCounts() {
  stepToolCounts.set(new Map());
}

// ============================================================================
// History Loading (from SQLite on page reload)
// ============================================================================

interface PipelineHistoryBundle {
  tool_calls: Array<{
    pipeline_id: string;
    tool_name: string;
    tool_input: string | null;
    is_error: boolean;
    summary: string | null;
    current_state: string;
    iteration: number;
    timestamp: number;
  }>;
  state_changes: Array<{
    pipeline_id: string;
    old_state: string;
    new_state: string;
    iteration: number;
    generated_skills: number;
    generated_subagents: number;
    claudemd_generated: boolean;
    timestamp: number;
  }>;
  decisions: Array<{
    pipeline_id: string;
    decision: string;
    reasoning: string | null;
    issues: string[];
    suggestions: string[];
    timestamp: number;
  }>;
}

export async function loadPipelineHistory(pipelineId: string): Promise<void> {
  const state = get(historyLoadState);
  if (state.loadedPipelines.has(pipelineId)) {
    console.log('[Orchestrator] Pipeline already loaded:', pipelineId);
    return;
  }

  historyLoadState.update(s => ({ ...s, isLoading: true }));
  setCurrentPipelineId(pipelineId);

  try {
    const history = await invoke<PipelineHistoryBundle>('get_pipeline_history', { pipelineId });
    console.log('[Orchestrator] Loaded pipeline history:', {
      toolCalls: history.tool_calls.length,
      stateChanges: history.state_changes.length,
      decisions: history.decisions.length,
    });

    // Convert DB records to frontend types and add to buffers (oldest first)
    // Note: query returns DESC order, so we reverse to get oldest first
    for (const tc of history.tool_calls.reverse()) {
      toolCallsBuffer.push({
        tool_name: tc.tool_name,
        tool_input: tc.tool_input ? JSON.parse(tc.tool_input) : undefined,
        is_error: tc.is_error,
        summary: tc.summary ?? undefined,
        current_state: tc.current_state,
        iteration: tc.iteration,
        timestamp: tc.timestamp,
      });
    }

    for (const sc of history.state_changes.reverse()) {
      stateChangesBuffer.push({
        old_state: sc.old_state,
        new_state: sc.new_state,
        iteration: sc.iteration,
        generated_skills: sc.generated_skills,
        generated_subagents: sc.generated_subagents,
        claudemd_generated: sc.claudemd_generated,
        timestamp: sc.timestamp,
      });
    }

    for (const d of history.decisions.reverse()) {
      decisionsBuffer.push({
        pipeline_id: d.pipeline_id,
        decision: d.decision as 'Complete' | 'Iterate' | 'Replan' | 'GiveUp',
        reasoning: d.reasoning ?? '',
        issues: d.issues,
        suggestions: d.suggestions,
        timestamp: d.timestamp,
      });
    }

    // Update reactive stores
    orchestratorToolCalls.set(toolCallsBuffer.toArray());
    orchestratorStateChanges.set(stateChangesBuffer.toArray());
    orchestratorDecisions.set(decisionsBuffer.toArray());

    // Set current state from latest state change
    const latestState = stateChangesBuffer.peek();
    if (latestState) {
      orchestratorCurrentState.set(latestState.new_state);
    }

    historyLoadState.update(s => ({
      ...s,
      isLoading: false,
      loadedPipelines: new Set([...s.loadedPipelines, pipelineId]),
    }));
  } catch (e) {
    console.error('[Orchestrator] Failed to load pipeline history:', e);
    historyLoadState.update(s => ({ ...s, isLoading: false }));
  }
}

// ============================================================================
// Clear Functions
// ============================================================================

export function clearOrchestratorActivity() {
  // Clear any pending debounced items
  pendingToolCalls = [];
  pendingStateChanges = [];
  pendingDecisions = [];

  if (toolCallTimer) {
    clearTimeout(toolCallTimer);
    toolCallTimer = null;
  }
  if (stateChangeTimer) {
    clearTimeout(stateChangeTimer);
    stateChangeTimer = null;
  }
  if (decisionTimer) {
    clearTimeout(decisionTimer);
    decisionTimer = null;
  }

  // Clear buffers
  toolCallsBuffer.clear();
  stateChangesBuffer.clear();
  decisionsBuffer.clear();

  // Clear stores
  orchestratorToolCalls.set([]);
  orchestratorStateChanges.set([]);
  orchestratorDecisions.set([]);
  orchestratorCurrentState.set("Idle");
  stepToolCounts.set(new Map());

  // Reset history load state
  historyLoadState.update(s => ({
    ...s,
    loadedPipelines: new Set(),
  }));

  currentPipelineId = null;
}

/**
 * Force flush all pending orchestrator updates (useful before cleanup)
 */
export function flushOrchestratorUpdates() {
  if (toolCallTimer) {
    clearTimeout(toolCallTimer);
    toolCallTimer = null;
  }
  if (stateChangeTimer) {
    clearTimeout(stateChangeTimer);
    stateChangeTimer = null;
  }
  if (decisionTimer) {
    clearTimeout(decisionTimer);
    decisionTimer = null;
  }

  flushToolCalls();
  flushStateChanges();
  flushDecisions();
}
