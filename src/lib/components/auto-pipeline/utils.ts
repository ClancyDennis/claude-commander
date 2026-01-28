import type { AutoPipeline, PipelineHistoryBundle } from '$lib/types';
import type { UnifiedOutput } from './types';

// Cache for processOutputs to avoid expensive recalculations
let outputCache: {
  pipelineId: string;
  stepsLength: number;
  historyToolCallsLength: number;
  historyStateChangesLength: number;
  historyDecisionsLength: number;
  result: UnifiedOutput[];
} = {
  pipelineId: '',
  stepsLength: 0,
  historyToolCallsLength: 0,
  historyStateChangesLength: 0,
  historyDecisionsLength: 0,
  result: []
};

/**
 * Process pipeline and history into unified outputs (with caching)
 */
export function processOutputs(
  pipeline: AutoPipeline | null,
  pipelineHistory: PipelineHistoryBundle | null
): UnifiedOutput[] {
  if (!pipeline) return [];

  // Check cache validity
  const historyToolCallsLength = pipelineHistory?.tool_calls?.length ?? 0;
  const historyStateChangesLength = pipelineHistory?.state_changes?.length ?? 0;
  const historyDecisionsLength = pipelineHistory?.decisions?.length ?? 0;

  if (
    outputCache.pipelineId === pipeline.id &&
    outputCache.stepsLength === pipeline.steps.length &&
    outputCache.historyToolCallsLength === historyToolCallsLength &&
    outputCache.historyStateChangesLength === historyStateChangesLength &&
    outputCache.historyDecisionsLength === historyDecisionsLength
  ) {
    return outputCache.result;
  }

  const outputs: UnifiedOutput[] = [];

  // Add agent outputs from steps
  for (const step of pipeline.steps) {
    if (step.output?.agent_outputs) {
      for (const output of step.output.agent_outputs) {
        outputs.push({
          stage: step.role,
          output_type: output.output_type,
          content: output.content,
          timestamp: output.timestamp || 0
        });
      }
    }
  }

  // Add orchestrator tool calls
  if (pipelineHistory?.tool_calls) {
    for (const toolCall of pipelineHistory.tool_calls) {
      // Parse JSON once and format (avoid parse+stringify round-trip when possible)
      let toolInputStr = '';
      if (toolCall.tool_input) {
        try {
          const parsed = JSON.parse(toolCall.tool_input);
          toolInputStr = JSON.stringify(parsed, null, 2);
        } catch {
          toolInputStr = toolCall.tool_input;
        }
      }
      outputs.push({
        stage: 'Orchestrator',
        output_type: 'orchestrator_tool',
        content: `${toolCall.tool_name}\n${toolInputStr}`,
        timestamp: toolCall.timestamp,
        summary: toolCall.summary
      });
    }
  }

  // Add state changes
  if (pipelineHistory?.state_changes) {
    for (const stateChange of pipelineHistory.state_changes) {
      outputs.push({
        stage: 'Orchestrator',
        output_type: 'state_change',
        content: `State: ${stateChange.old_state} -> ${stateChange.new_state}\nIteration: ${stateChange.iteration}\nSkills: ${stateChange.generated_skills}, Subagents: ${stateChange.generated_subagents}`,
        timestamp: stateChange.timestamp
      });
    }
  }

  // Add decisions
  if (pipelineHistory?.decisions) {
    for (const decision of pipelineHistory.decisions) {
      outputs.push({
        stage: 'Orchestrator',
        output_type: 'decision',
        content: `Decision: ${decision.decision}\nReasoning: ${decision.reasoning || 'N/A'}\nIssues: ${decision.issues.join(', ')}\nSuggestions: ${decision.suggestions.join(', ')}`,
        timestamp: decision.timestamp
      });
    }
  }

  // Sort by timestamp
  const result = outputs.sort((a, b) => a.timestamp - b.timestamp);

  // Update cache
  outputCache = {
    pipelineId: pipeline.id,
    stepsLength: pipeline.steps.length,
    historyToolCallsLength,
    historyStateChangesLength,
    historyDecisionsLength,
    result
  };

  return result;
}

/**
 * Count outputs per stage
 */
export function countByStage(outputs: UnifiedOutput[]): { Orchestrator: number; Planning: number; Building: number; Verifying: number } {
  const counts = { Orchestrator: 0, Planning: 0, Building: 0, Verifying: 0 };
  for (const o of outputs) {
    if (o.stage in counts) {
      counts[o.stage as keyof typeof counts]++;
    }
  }
  return counts;
}

/**
 * Count outputs per type
 */
export function countByType(outputs: UnifiedOutput[]): {
  text: number;
  tool_use: number;
  tool_result: number;
  error: number;
  orchestrator_tool: number;
  state_change: number;
  decision: number
} {
  const counts = { text: 0, tool_use: 0, tool_result: 0, error: 0, orchestrator_tool: 0, state_change: 0, decision: 0 };
  for (const o of outputs) {
    if (o.output_type in counts) {
      counts[o.output_type as keyof typeof counts]++;
    }
  }
  return counts;
}

/**
 * Count outputs per stage AND type in a single pass (performance optimization)
 */
export function countOutputs(outputs: UnifiedOutput[]): {
  stageCounts: { Orchestrator: number; Planning: number; Building: number; Verifying: number };
  typeCounts: { text: number; tool_use: number; tool_result: number; error: number; orchestrator_tool: number; state_change: number; decision: number };
} {
  const stageCounts = { Orchestrator: 0, Planning: 0, Building: 0, Verifying: 0 };
  const typeCounts = { text: 0, tool_use: 0, tool_result: 0, error: 0, orchestrator_tool: 0, state_change: 0, decision: 0 };

  for (const o of outputs) {
    if (o.stage in stageCounts) {
      stageCounts[o.stage as keyof typeof stageCounts]++;
    }
    if (o.output_type in typeCounts) {
      typeCounts[o.output_type as keyof typeof typeCounts]++;
    }
  }

  return { stageCounts, typeCounts };
}

/**
 * Filter outputs based on stage, type, and search query
 */
export function filterOutputs(
  outputs: UnifiedOutput[],
  stageFilter: string,
  typeFilter: string,
  searchQuery: string
): UnifiedOutput[] {
  return outputs.filter(o => {
    if (stageFilter !== 'all' && o.stage !== stageFilter) return false;
    if (typeFilter !== 'all' && o.output_type !== typeFilter) return false;
    if (searchQuery && !o.content.toLowerCase().includes(searchQuery.toLowerCase())) return false;
    return true;
  });
}

/**
 * Format tool display string
 */
export function formatToolDisplay(tool: { tool_name?: string; tool_input?: Record<string, unknown> } | null): string {
  if (!tool) return 'Waiting...';
  const name = tool.tool_name || 'Unknown';
  let display = name;
  if (tool.tool_input) {
    const input = tool.tool_input;
    if (typeof input === 'object') {
      const val = input.file_path || input.path || input.command || input.pattern || Object.values(input)[0];
      if (val && typeof val === 'string') {
        const short = val.split('/').pop()?.slice(0, 25) || val.slice(0, 25);
        display = `${name}(${short}${val.length > 25 ? '...' : ''})`;
      }
    }
  }
  return display;
}

/**
 * Map orchestrator state to active stage index
 */
export function getOrchestratorActiveStage(state: string): number | null {
  switch (state) {
    case 'Planning':
    case 'ReadyForExecution':
      return 0;
    case 'Executing':
      return 1;
    case 'Verifying':
      return 2;
    case 'Completed':
    case 'GaveUp':
    default:
      return null;
  }
}

/**
 * Check if orchestrator is actively working
 */
export function isOrchestratorActiveState(state: string): boolean {
  return state !== 'Idle' && state !== 'Completed' && !state.includes('Failed');
}
