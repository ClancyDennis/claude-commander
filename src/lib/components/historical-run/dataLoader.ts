/**
 * Data loading utilities for HistoricalRunView components.
 *
 * Consolidates the repetitive loading patterns for prompts, activity, and outputs.
 */

import { invoke } from "@tauri-apps/api/core";
import type {
  AgentOutputRecord,
  OrchestratorToolCall,
  OrchestratorStateChange,
  OrchestratorDecision,
  OrchestratorToolCallRecord,
  OrchestratorStateChangeRecord,
  OrchestratorDecisionRecord
} from "$lib/types";

// ============================================================================
// Types
// ============================================================================

export interface PromptData {
  prompt: string;
  timestamp: number;
}

export interface ActivityData {
  toolCalls: OrchestratorToolCall[];
  stateChanges: OrchestratorStateChange[];
  decisions: OrchestratorDecision[];
}

export interface LoadResult<T> {
  data: T;
  error: string | null;
}

// ============================================================================
// Record Converters
// ============================================================================

/**
 * Convert database tool call record to display type
 */
export function convertToolCallRecord(record: OrchestratorToolCallRecord): OrchestratorToolCall {
  let parsedInput: Record<string, unknown> = {};
  if (record.tool_input) {
    try {
      parsedInput = JSON.parse(record.tool_input);
    } catch { /* ignore parse errors */ }
  }
  return {
    tool_name: record.tool_name,
    tool_input: parsedInput,
    is_error: record.is_error,
    summary: record.summary,
    current_state: record.current_state,
    iteration: record.iteration,
    timestamp: record.timestamp
  };
}

/**
 * Convert database state change record to display type
 */
export function convertStateChangeRecord(record: OrchestratorStateChangeRecord): OrchestratorStateChange {
  return {
    old_state: record.old_state,
    new_state: record.new_state,
    iteration: record.iteration,
    generated_skills: record.generated_skills,
    generated_subagents: record.generated_subagents,
    claudemd_generated: record.claudemd_generated,
    timestamp: record.timestamp
  };
}

/**
 * Convert database decision record to display type
 */
export function convertDecisionRecord(record: OrchestratorDecisionRecord): OrchestratorDecision {
  return {
    pipeline_id: record.pipeline_id,
    decision: record.decision as OrchestratorDecision['decision'],
    reasoning: record.reasoning || '',
    issues: record.issues || [],
    suggestions: record.suggestions || [],
    timestamp: record.timestamp
  };
}

// ============================================================================
// Data Loaders
// ============================================================================

/**
 * Load prompts for a given agent
 */
export async function loadPrompts(agentId: string): Promise<LoadResult<PromptData[]>> {
  try {
    const result = await invoke<Array<[string, number]>>("get_run_prompts", { agentId });
    const data = result.map(([prompt, timestamp]) => ({ prompt, timestamp }));
    return { data, error: null };
  } catch (e) {
    console.error("Failed to load run prompts:", e);
    return { data: [], error: "Failed to load conversation history" };
  }
}

/**
 * Load orchestrator activity (tool calls, state changes, decisions) for a pipeline
 */
export async function loadActivity(pipelineId: string): Promise<LoadResult<ActivityData>> {
  try {
    const [toolCallsResult, stateChangesResult, decisionsResult] = await Promise.all([
      invoke<OrchestratorToolCallRecord[]>("get_orchestrator_tool_calls", {
        pipelineId,
        limit: 1000
      }),
      invoke<OrchestratorStateChangeRecord[]>("get_orchestrator_state_changes", {
        pipelineId,
        limit: 500
      }),
      invoke<OrchestratorDecisionRecord[]>("get_orchestrator_decisions", {
        pipelineId,
        limit: 100
      }),
    ]);

    const data: ActivityData = {
      toolCalls: (toolCallsResult || []).map(convertToolCallRecord),
      stateChanges: (stateChangesResult || []).map(convertStateChangeRecord),
      decisions: (decisionsResult || []).map(convertDecisionRecord)
    };

    return { data, error: null };
  } catch (e) {
    console.error("Failed to load activity:", e);
    return {
      data: { toolCalls: [], stateChanges: [], decisions: [] },
      error: "Failed to load orchestrator activity"
    };
  }
}

/**
 * Load outputs for an agent
 */
export async function loadOutputs(
  agentId: string,
  pipelineId?: string
): Promise<LoadResult<AgentOutputRecord[]>> {
  try {
    const result = await invoke<AgentOutputRecord[]>("get_agent_output_history", {
      agentId,
      pipelineId,
      limit: 2000
    });
    return { data: result || [], error: null };
  } catch (e) {
    console.error("Failed to load outputs:", e);
    return { data: [], error: "Failed to load agent outputs" };
  }
}
