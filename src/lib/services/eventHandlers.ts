/**
 * Event Handlers Service
 *
 * Centralized event listener setup for Tauri events.
 * Groups handlers by category: agent, pipeline, auto-pipeline, orchestrator.
 */

import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// ============================================================================
// Throttle Utility for High-Frequency Events
// ============================================================================

/**
 * Throttle function calls to at most once per `ms` milliseconds.
 * Ensures the last call is always executed (trailing edge).
 */
function throttle<T extends (...args: unknown[]) => void>(fn: T, ms: number): T {
  let lastCall = 0;
  let pending: Parameters<T> | null = null;
  let timeoutId: ReturnType<typeof setTimeout> | null = null;

  return ((...args: Parameters<T>) => {
    const now = Date.now();
    if (now - lastCall >= ms) {
      lastCall = now;
      fn(...args);
    } else {
      pending = args;
      if (!timeoutId) {
        timeoutId = setTimeout(() => {
          if (pending) {
            lastCall = Date.now();
            fn(...pending);
            pending = null;
          }
          timeoutId = null;
        }, ms - (now - lastCall));
      }
    }
  }) as T;
}

import type {
  Agent,
  AgentOutput,
  ToolEvent,
  AgentStatusEvent,
  AgentInputRequiredEvent,
  AgentActivityEvent,
  AgentStatsEvent,
  MetaAgentThinkingEvent,
  MetaAgentToolCallEvent,
  MetaTodoUpdatedEvent,
  MetaAgentUserUpdateEvent,
  MetaAgentQuestionEvent,
  MetaAgentSleepEvent,
  AutoPipeline,
  OrchestratorToolCall,
  OrchestratorStateChange,
  OrchestratorDecision,
  SecurityAlertPayload,
  SecurityAgentTerminatedPayload,
  SecurityAgentSuspendedPayload,
  SecurityPendingReviewPayload,
  SecurityReviewCompletedPayload,
  ElevatedCommandRequestEvent,
  ElevatedCommandStatusEvent,
  PendingElevatedCommand,
} from "$lib/types";
import type { Pipeline, PhaseProgressData } from "$lib/stores/pipelines";
import { fetchAutoPipeline } from "./pipelineHelpers";

// ============================================================================
// Types for Event Handlers
// ============================================================================

export interface EventHandlerCallbacks {
  // Agent callbacks
  onAgentOutput: (agentId: string, output: AgentOutput) => void;
  onToolEvent: (agentId: string, event: ToolEvent) => void;
  onAgentStatus: (agentId: string, status: string, agent?: Agent) => void;
  onInputRequired: (agentId: string) => void;
  onAgentActivity: (agentId: string, activity: {
    isProcessing: boolean;
    pendingInput: boolean;
    lastActivity: Date;
  }) => void;
  onAgentActivityDetail?: (agentId: string, detail: {
    activity: string;
    toolName: string;
    timestamp: Date;
  }) => void;
  onAgentStats: (agentId: string, stats: {
    agentId: string;
    totalPrompts: number;
    totalToolCalls: number;
    totalOutputBytes: number;
    sessionStart: string;
    lastActivity: string;
    totalTokensUsed?: number;
    totalCostUsd?: number;
  }) => void;

  // Meta-agent callbacks
  onMetaAgentThinking: (isThinking: boolean) => void;
  onMetaAgentToolCall: (toolCall: MetaAgentToolCallEvent) => void;
  onMetaAgentTodos?: (event: MetaTodoUpdatedEvent) => void;
  onMetaAgentUserUpdate?: (event: MetaAgentUserUpdateEvent) => void;
  onMetaAgentQuestion?: (event: MetaAgentQuestionEvent) => void;
  onMetaAgentSleep?: (event: MetaAgentSleepEvent) => void;
  onNavigate: (agentId: string) => void;

  // Pipeline callbacks
  onPipelineCreated: (pipeline: Pipeline) => void;
  onPipelineStatus: (pipelineId: string, status: Pipeline['status']) => void;
  onPipelinePhase: (pipelineId: string, phase: string) => void;
  onPhaseProgress: (data: PhaseProgressData) => void;

  // Auto-pipeline callbacks
  onAutoPipelineStarted: (pipeline: AutoPipeline) => void;
  onAutoPipelineStep: (pipelineId: string, stepNumber: number) => void;
  onAutoPipelineComplete: (pipelineId: string) => void;
  onStepStatus: (pipelineId: string, stepNumber: number, status: string) => void;

  // Orchestrator callbacks
  onOrchestratorToolStart: (toolCall: OrchestratorToolCall) => void;
  onOrchestratorToolComplete: (data: {
    toolName: string;
    isError: boolean;
    summary: string;
    currentState: string;
    stepNumber?: number;
  }) => void;
  onOrchestratorStateChange: (stateChange: OrchestratorStateChange) => void;
  onOrchestratorDecision: (decision: OrchestratorDecision) => void;

  // Security callbacks
  onSecurityAlert?: (payload: SecurityAlertPayload) => void;
  onSecurityAgentTerminated?: (payload: SecurityAgentTerminatedPayload) => void;
  onSecurityAgentSuspended?: (payload: SecurityAgentSuspendedPayload) => void;
  onSecurityPendingReview?: (payload: SecurityPendingReviewPayload) => void;
  onSecurityReviewCompleted?: (payload: SecurityReviewCompletedPayload) => void;

  // Toast callback
  onToast?: (toast: {
    type: "info" | "success" | "warning" | "error";
    message: string;
    duration?: number;
  }) => void;

  // Elevated command callbacks
  onElevatedCommandRequest?: (request: PendingElevatedCommand) => void;
  onElevatedCommandStatus?: (requestId: string, status: string, error?: string) => void;
}

// ============================================================================
// Agent Event Handlers
// ============================================================================

async function setupAgentOutputListener(
  onAgentOutput: EventHandlerCallbacks['onAgentOutput']
): Promise<UnlistenFn> {
  return listen<{
    agent_id: string;
    output_type: string;
    content: string;
  }>("agent:output", (event) => {
    const output: AgentOutput = {
      agentId: event.payload.agent_id,
      type: event.payload.output_type as AgentOutput["type"],
      content: event.payload.content,
      timestamp: new Date(),
    };
    onAgentOutput(event.payload.agent_id, output);
  });
}

async function setupToolEventListener(
  onToolEvent: EventHandlerCallbacks['onToolEvent']
): Promise<UnlistenFn> {
  return listen<{
    agent_id: string;
    session_id: string;
    hook_event_name: string;
    tool_name: string;
    tool_input: Record<string, unknown>;
    tool_response?: Record<string, unknown>;
    tool_call_id: string;
    status?: string;
    error_message?: string;
    execution_time_ms?: number;
    timestamp: number;
  }>("agent:tool", (event) => {
    const toolEvent: ToolEvent = {
      agentId: event.payload.agent_id,
      sessionId: event.payload.session_id,
      hookEventName: event.payload.hook_event_name,
      toolName: event.payload.tool_name,
      toolInput: event.payload.tool_input,
      toolResponse: event.payload.tool_response,
      timestamp: new Date(event.payload.timestamp),
      toolCallId: event.payload.tool_call_id,
      status: event.payload.status,
      errorMessage: event.payload.error_message,
      executionTimeMs: event.payload.execution_time_ms,
    };
    onToolEvent(event.payload.agent_id, toolEvent);
  });
}

async function setupStatusListener(
  onAgentStatus: EventHandlerCallbacks['onAgentStatus']
): Promise<UnlistenFn> {
  return listen<AgentStatusEvent>("agent:status", (event) => {
    console.log("[Frontend] Received agent:status event:", event.payload.agent_id, event.payload.status);

    // Check if this is a new agent (has info property)
    let agent: Agent | undefined;
    if (event.payload.info) {
      const info = event.payload.info;
      agent = {
        id: info.id,
        workingDir: info.working_dir,
        status: event.payload.status,
        createdAt: new Date(),
        lastActivity: info.last_activity ? new Date(info.last_activity) : undefined,
        isProcessing: info.is_processing,
        pendingInput: info.pending_input,
        githubContext: info.github_context ? {
          repositoryUrl: info.github_context.repository_url,
          owner: info.github_context.owner,
          repo: info.github_context.repo,
          branch: info.github_context.branch,
          commitSha: info.github_context.commit_sha,
          lastSynced: info.github_context.last_synced,
        } : undefined,
        title: info.title,
      };
    }

    onAgentStatus(event.payload.agent_id, event.payload.status, agent);
  });
}

async function setupInputRequiredListener(
  onInputRequired: EventHandlerCallbacks['onInputRequired']
): Promise<UnlistenFn> {
  return listen<AgentInputRequiredEvent>("agent:input_required", (event) => {
    onInputRequired(event.payload.agent_id);
  });
}

async function setupActivityListener(
  onAgentActivity: EventHandlerCallbacks['onAgentActivity']
): Promise<UnlistenFn> {
  return listen<AgentActivityEvent>("agent:activity", (event) => {
    const lastActivity = event.payload.last_activity
      ? new Date(event.payload.last_activity)
      : new Date();

    onAgentActivity(event.payload.agent_id, {
      isProcessing: event.payload.is_processing,
      pendingInput: event.payload.pending_input,
      lastActivity,
    });
  });
}

async function setupStatsListener(
  onAgentStats: EventHandlerCallbacks['onAgentStats']
): Promise<UnlistenFn> {
  // Throttle stats updates to max 1 per second per agent (stats don't need real-time updates)
  const throttledHandlers = new Map<string, (stats: {
    agentId: string;
    totalPrompts: number;
    totalToolCalls: number;
    totalOutputBytes: number;
    sessionStart: string;
    lastActivity: string;
    totalTokensUsed?: number;
    totalCostUsd?: number;
  }) => void>();

  return listen<AgentStatsEvent>("agent:stats", (event) => {
    const agentId = event.payload.agent_id;

    // Create throttled handler for this agent if not exists
    if (!throttledHandlers.has(agentId)) {
      throttledHandlers.set(agentId, throttle((stats) => {
        onAgentStats(agentId, stats);
      }, 1000));
    }

    throttledHandlers.get(agentId)!({
      agentId: event.payload.stats.agent_id,
      totalPrompts: event.payload.stats.total_prompts,
      totalToolCalls: event.payload.stats.total_tool_calls,
      totalOutputBytes: event.payload.stats.total_output_bytes,
      sessionStart: event.payload.stats.session_start,
      lastActivity: event.payload.stats.last_activity,
      totalTokensUsed: event.payload.stats.total_tokens_used,
      totalCostUsd: event.payload.stats.total_cost_usd,
    });
  });
}

async function setupActivityDetailListener(
  onAgentActivityDetail: EventHandlerCallbacks['onAgentActivityDetail']
): Promise<UnlistenFn> {
  // Throttle activity detail updates to max 2 per second per agent
  const throttledHandlers = new Map<string, (detail: { activity: string; toolName: string; timestamp: Date }) => void>();

  return listen<{
    agent_id: string;
    activity: string;
    tool_name: string;
    timestamp: number;
  }>("agent:activity:detail", (event) => {
    const agentId = event.payload.agent_id;

    // Create throttled handler for this agent if not exists
    if (!throttledHandlers.has(agentId)) {
      throttledHandlers.set(agentId, throttle((detail: { activity: string; toolName: string; timestamp: Date }) => {
        onAgentActivityDetail?.(agentId, detail);
      }, 500));
    }

    throttledHandlers.get(agentId)!({
      activity: event.payload.activity,
      toolName: event.payload.tool_name,
      timestamp: new Date(event.payload.timestamp),
    });
  });
}

// ============================================================================
// Meta-Agent Event Handlers
// ============================================================================

async function setupThinkingListener(
  onMetaAgentThinking: EventHandlerCallbacks['onMetaAgentThinking']
): Promise<UnlistenFn> {
  return listen<MetaAgentThinkingEvent>("meta-agent:thinking", (event) => {
    onMetaAgentThinking(event.payload.is_thinking);
  });
}

async function setupMetaAgentToolCallListener(
  onMetaAgentToolCall: EventHandlerCallbacks['onMetaAgentToolCall']
): Promise<UnlistenFn> {
  return listen<MetaAgentToolCallEvent>("meta-agent:tool-call", (event) => {
    onMetaAgentToolCall(event.payload);
  });
}

async function setupNavigateListener(
  onNavigate: EventHandlerCallbacks['onNavigate']
): Promise<UnlistenFn> {
  return listen<{ agent_id: string }>("agent:navigate", (event) => {
    onNavigate(event.payload.agent_id);
  });
}

async function setupMetaAgentTodosListener(
  onMetaAgentTodos: EventHandlerCallbacks['onMetaAgentTodos']
): Promise<UnlistenFn> {
  return listen<MetaTodoUpdatedEvent>("meta-agent:todos", (event) => {
    console.log("[Frontend] Meta-agent todos updated:", event.payload.todos.length, "items");
    onMetaAgentTodos?.(event.payload);
  });
}

async function setupMetaAgentUserUpdateListener(
  onMetaAgentUserUpdate: EventHandlerCallbacks['onMetaAgentUserUpdate']
): Promise<UnlistenFn> {
  return listen<MetaAgentUserUpdateEvent>("meta-agent:user-update", (event) => {
    console.log("[Frontend] Meta-agent user update:", event.payload.level, "-", event.payload.message.substring(0, 50));
    onMetaAgentUserUpdate?.(event.payload);
  });
}

async function setupMetaAgentQuestionListener(
  onMetaAgentQuestion: EventHandlerCallbacks['onMetaAgentQuestion']
): Promise<UnlistenFn> {
  return listen<MetaAgentQuestionEvent>("meta-agent:question", (event) => {
    console.log("[Frontend] Meta-agent question:", event.payload.question_id, "-", event.payload.question.substring(0, 50));
    onMetaAgentQuestion?.(event.payload);
  });
}

async function setupMetaAgentSleepListener(
  onMetaAgentSleep: EventHandlerCallbacks['onMetaAgentSleep']
): Promise<UnlistenFn> {
  return listen<MetaAgentSleepEvent>("meta-agent:status", (event) => {
    // Only handle sleep-related status events
    if (event.payload.status === "sleeping" || event.payload.status === "awake") {
      console.log("[Frontend] Meta-agent sleep status:", event.payload.status, event.payload.reason || "");
      onMetaAgentSleep?.(event.payload);
    }
  });
}

// ============================================================================
// Pipeline Event Handlers
// ============================================================================

async function setupPipelineCreatedListener(
  onPipelineCreated: EventHandlerCallbacks['onPipelineCreated']
): Promise<UnlistenFn> {
  return listen<{
    pipeline_id: string;
    working_dir: string;
    user_request: string;
  }>("pipeline:created", (event) => {
    const pipeline: Pipeline = {
      id: event.payload.pipeline_id,
      workingDir: event.payload.working_dir,
      userRequest: event.payload.user_request,
      status: 'planning',
      createdAt: new Date(),
    };
    onPipelineCreated(pipeline);
  });
}

async function setupPipelineStatusListener(
  onPipelineStatus: EventHandlerCallbacks['onPipelineStatus']
): Promise<UnlistenFn> {
  return listen<{
    pipeline_id: string;
    status: Pipeline['status'];
  }>("pipeline:status", (event) => {
    onPipelineStatus(event.payload.pipeline_id, event.payload.status);
  });
}

async function setupPipelinePhaseListener(
  onPipelinePhase: EventHandlerCallbacks['onPipelinePhase']
): Promise<UnlistenFn> {
  return listen<{
    pipeline_id: string;
    phase: string;
  }>("pipeline:phase", (event) => {
    onPipelinePhase(event.payload.pipeline_id, event.payload.phase);
  });
}

async function setupPhaseProgressListener(
  onPhaseProgress: EventHandlerCallbacks['onPhaseProgress']
): Promise<UnlistenFn> {
  return listen<PhaseProgressData>("pipeline:progress", (event) => {
    onPhaseProgress(event.payload);
  });
}

// ============================================================================
// Auto-Pipeline Event Handlers
// ============================================================================

async function setupAutoPipelineStartedListener(
  onAutoPipelineStarted: EventHandlerCallbacks['onAutoPipelineStarted']
): Promise<UnlistenFn> {
  return listen<AutoPipeline | { pipeline_id: string }>("auto_pipeline:started", async (event) => {
    const payload = event.payload;
    const pipelineId = 'id' in payload ? payload.id : payload.pipeline_id;

    console.log("[Frontend] Auto-pipeline started:", pipelineId);

    // Check if we received the full pipeline object
    if ('id' in payload && 'status' in payload) {
      onAutoPipelineStarted(payload as AutoPipeline);
    } else {
      // Fallback to fetch if we only got the ID
      const pipeline = await fetchAutoPipeline(pipelineId);
      if (pipeline) {
        onAutoPipelineStarted(pipeline);
      }
    }
  });
}

async function setupAutoPipelineStepListener(
  onAutoPipelineStep: EventHandlerCallbacks['onAutoPipelineStep']
): Promise<UnlistenFn> {
  return listen<{
    pipeline_id: string;
    step_number: number;
    output: unknown;
  }>("auto_pipeline:step_completed", (event) => {
    onAutoPipelineStep(event.payload.pipeline_id, event.payload.step_number);
  });
}

async function setupAutoPipelineCompleteListener(
  onAutoPipelineComplete: EventHandlerCallbacks['onAutoPipelineComplete']
): Promise<UnlistenFn> {
  return listen<{
    pipeline_id: string;
    verification_report: unknown;
  }>("auto_pipeline:completed", (event) => {
    onAutoPipelineComplete(event.payload.pipeline_id);
  });
}

async function setupStepStatusListener(
  onStepStatus: EventHandlerCallbacks['onStepStatus']
): Promise<UnlistenFn> {
  return listen<{
    pipeline_id: string;
    step_number: number;
    status: string;
  }>("auto_pipeline:step_status", (event) => {
    console.log("[Frontend] Step status change:", event.payload.step_number, event.payload.status);
    onStepStatus(event.payload.pipeline_id, event.payload.step_number, event.payload.status);
  });
}

// ============================================================================
// Orchestrator Event Handlers
// ============================================================================

async function setupOrchestratorToolStartListener(
  onOrchestratorToolStart: EventHandlerCallbacks['onOrchestratorToolStart']
): Promise<UnlistenFn> {
  return listen<{
    tool_name: string;
    tool_input: Record<string, unknown>;
    current_state: string;
    iteration: number;
  }>("orchestrator:tool_start", (event) => {
    console.log("[Frontend] Orchestrator tool start:", event.payload.tool_name);
    onOrchestratorToolStart({
      tool_name: event.payload.tool_name,
      tool_input: event.payload.tool_input,
      current_state: event.payload.current_state,
      iteration: event.payload.iteration,
      timestamp: Date.now(),
    });
  });
}

async function setupOrchestratorToolCompleteListener(
  onOrchestratorToolComplete: EventHandlerCallbacks['onOrchestratorToolComplete']
): Promise<UnlistenFn> {
  return listen<{
    tool_name: string;
    is_error: boolean;
    summary: string;
    current_state: string;
    step_number?: number;
  }>("orchestrator:tool_complete", (event) => {
    console.log("[Frontend] Orchestrator tool complete:", event.payload.tool_name, event.payload.is_error ? "(error)" : "", "step:", event.payload.step_number);
    onOrchestratorToolComplete({
      toolName: event.payload.tool_name,
      isError: event.payload.is_error,
      summary: event.payload.summary,
      currentState: event.payload.current_state,
      stepNumber: event.payload.step_number,
    });
  });
}

async function setupOrchestratorStateChangeListener(
  onOrchestratorStateChange: EventHandlerCallbacks['onOrchestratorStateChange']
): Promise<UnlistenFn> {
  return listen<{
    old_state: string;
    new_state: string;
    iteration: number;
    generated_skills: number;
    generated_subagents: number;
    claudemd_generated: boolean;
  }>("orchestrator:state_changed", (event) => {
    console.log("[Frontend] Orchestrator state change:", event.payload.old_state, "->", event.payload.new_state);
    onOrchestratorStateChange({
      old_state: event.payload.old_state,
      new_state: event.payload.new_state,
      iteration: event.payload.iteration,
      generated_skills: event.payload.generated_skills,
      generated_subagents: event.payload.generated_subagents,
      claudemd_generated: event.payload.claudemd_generated,
      timestamp: Date.now(),
    });
  });
}

async function setupOrchestratorDecisionListener(
  onOrchestratorDecision: EventHandlerCallbacks['onOrchestratorDecision']
): Promise<UnlistenFn> {
  return listen<{
    pipeline_id: string;
    decision: string;
    reasoning: string;
    issues: string[];
    suggestions: string[];
  }>("auto_pipeline:decision", (event) => {
    console.log("[Frontend] Orchestrator decision:", event.payload.decision);
    onOrchestratorDecision({
      pipeline_id: event.payload.pipeline_id,
      decision: event.payload.decision as OrchestratorDecision['decision'],
      reasoning: event.payload.reasoning,
      issues: event.payload.issues,
      suggestions: event.payload.suggestions,
      timestamp: Date.now(),
    });
  });
}

// ============================================================================
// Security Event Handlers
// ============================================================================

async function setupSecurityAlertListener(
  onSecurityAlert: EventHandlerCallbacks['onSecurityAlert']
): Promise<UnlistenFn> {
  return listen<SecurityAlertPayload>("security:alert", (event) => {
    console.log("[Frontend] Security alert received:", event.payload.title, "severity:", event.payload.risk_level);
    onSecurityAlert?.(event.payload);
  });
}

async function setupSecurityAgentTerminatedListener(
  onSecurityAgentTerminated: EventHandlerCallbacks['onSecurityAgentTerminated']
): Promise<UnlistenFn> {
  return listen<SecurityAgentTerminatedPayload>("security:agent_terminated", (event) => {
    console.log("[Frontend] Agent terminated by security:", event.payload.agent_id, "reason:", event.payload.reason);
    onSecurityAgentTerminated?.(event.payload);
  });
}

async function setupSecurityAgentSuspendedListener(
  onSecurityAgentSuspended: EventHandlerCallbacks['onSecurityAgentSuspended']
): Promise<UnlistenFn> {
  return listen<SecurityAgentSuspendedPayload>("security:agent_suspended", (event) => {
    console.log("[Frontend] Agent suspended by security:", event.payload.agent_id, "reason:", event.payload.reason);
    onSecurityAgentSuspended?.(event.payload);
  });
}

async function setupSecurityPendingReviewListener(
  onSecurityPendingReview: EventHandlerCallbacks['onSecurityPendingReview']
): Promise<UnlistenFn> {
  return listen<SecurityPendingReviewPayload>("security:pending_review", (event) => {
    console.log("[Frontend] Security review pending:", event.payload.id, "action:", event.payload.recommended_action);
    onSecurityPendingReview?.(event.payload);
  });
}

async function setupSecurityReviewCompletedListener(
  onSecurityReviewCompleted: EventHandlerCallbacks['onSecurityReviewCompleted']
): Promise<UnlistenFn> {
  return listen<SecurityReviewCompletedPayload>("security:review_completed", (event) => {
    console.log("[Frontend] Security review completed:", event.payload.review_id, "approved:", event.payload.approved);
    onSecurityReviewCompleted?.(event.payload);
  });
}

// ============================================================================
// Toast Event Handler
// ============================================================================

async function setupToastListener(
  onToast: EventHandlerCallbacks['onToast']
): Promise<UnlistenFn> {
  return listen<{
    type: "info" | "success" | "warning" | "error";
    message: string;
    duration?: number;
  }>("toast", (event) => {
    console.log("[Frontend] Toast received:", event.payload.type, event.payload.message);
    onToast?.(event.payload);
  });
}

// ============================================================================
// Elevated Command Event Handlers
// ============================================================================

async function setupElevatedRequestListener(
  onElevatedCommandRequest: EventHandlerCallbacks['onElevatedCommandRequest']
): Promise<UnlistenFn> {
  return listen<ElevatedCommandRequestEvent>("elevated:request", (event) => {
    console.log("[Frontend] Elevated command request:", event.payload.request.command);
    onElevatedCommandRequest?.(event.payload.request);
  });
}

async function setupElevatedStatusListener(
  onElevatedCommandStatus: EventHandlerCallbacks['onElevatedCommandStatus']
): Promise<UnlistenFn> {
  return listen<ElevatedCommandStatusEvent>("elevated:status", (event) => {
    console.log("[Frontend] Elevated command status:", event.payload.requestId, event.payload.status);
    onElevatedCommandStatus?.(event.payload.requestId, event.payload.status, event.payload.error);
  });
}

// ============================================================================
// Main Setup Function
// ============================================================================

/**
 * Sets up all event listeners and returns a cleanup function.
 * Call the returned function to unsubscribe from all events.
 */
export async function setupEventListeners(
  callbacks: EventHandlerCallbacks
): Promise<() => void> {
  // Setup all listeners in parallel
  const unlistenPromises = await Promise.all([
    // Agent events (7)
    setupAgentOutputListener(callbacks.onAgentOutput),
    setupToolEventListener(callbacks.onToolEvent),
    setupStatusListener(callbacks.onAgentStatus),
    setupInputRequiredListener(callbacks.onInputRequired),
    setupActivityListener(callbacks.onAgentActivity),
    setupStatsListener(callbacks.onAgentStats),
    setupActivityDetailListener(callbacks.onAgentActivityDetail),

    // Meta-agent events (7)
    setupThinkingListener(callbacks.onMetaAgentThinking),
    setupMetaAgentToolCallListener(callbacks.onMetaAgentToolCall),
    setupMetaAgentTodosListener(callbacks.onMetaAgentTodos),
    setupMetaAgentUserUpdateListener(callbacks.onMetaAgentUserUpdate),
    setupMetaAgentQuestionListener(callbacks.onMetaAgentQuestion),
    setupMetaAgentSleepListener(callbacks.onMetaAgentSleep),
    setupNavigateListener(callbacks.onNavigate),

    // Pipeline events (4)
    setupPipelineCreatedListener(callbacks.onPipelineCreated),
    setupPipelineStatusListener(callbacks.onPipelineStatus),
    setupPipelinePhaseListener(callbacks.onPipelinePhase),
    setupPhaseProgressListener(callbacks.onPhaseProgress),

    // Auto-pipeline events (4)
    setupAutoPipelineStartedListener(callbacks.onAutoPipelineStarted),
    setupAutoPipelineStepListener(callbacks.onAutoPipelineStep),
    setupAutoPipelineCompleteListener(callbacks.onAutoPipelineComplete),
    setupStepStatusListener(callbacks.onStepStatus),

    // Orchestrator events (4)
    setupOrchestratorToolStartListener(callbacks.onOrchestratorToolStart),
    setupOrchestratorToolCompleteListener(callbacks.onOrchestratorToolComplete),
    setupOrchestratorStateChangeListener(callbacks.onOrchestratorStateChange),
    setupOrchestratorDecisionListener(callbacks.onOrchestratorDecision),

    // Security events (5)
    setupSecurityAlertListener(callbacks.onSecurityAlert),
    setupSecurityAgentTerminatedListener(callbacks.onSecurityAgentTerminated),
    setupSecurityAgentSuspendedListener(callbacks.onSecurityAgentSuspended),
    setupSecurityPendingReviewListener(callbacks.onSecurityPendingReview),
    setupSecurityReviewCompletedListener(callbacks.onSecurityReviewCompleted),

    // Toast event (1)
    setupToastListener(callbacks.onToast),

    // Elevated command events (2)
    setupElevatedRequestListener(callbacks.onElevatedCommandRequest),
    setupElevatedStatusListener(callbacks.onElevatedCommandStatus),
  ]);

  // Return cleanup function
  return () => {
    unlistenPromises.forEach((unlisten) => unlisten());
  };
}
