export type AgentStatus = "running" | "stopped" | "error" | "waitingforinput" | "idle" | "processing";

export interface GitHubContext {
  repositoryUrl: string;
  owner: string;
  repo: string;
  branch: string;
  commitSha?: string;
  lastSynced?: string;
}

export interface Agent {
  id: string;
  workingDir: string;
  status: AgentStatus;
  createdAt: Date;
  lastActivity?: Date;
  isProcessing?: boolean;
  pendingInput?: boolean;
  unreadOutputs?: number;
  githubContext?: GitHubContext;
}

export interface AgentOutput {
  agentId: string;
  type: "text" | "tool_use" | "tool_result" | "error" | "system" | "result";
  content: string;
  timestamp: Date;
  parsedJson?: Record<string, unknown>;
  metadata?: OutputMetadata;
}

export interface OutputMetadata {
  language?: string;
  lineCount?: number;
  byteSize?: number;
  isTruncated: boolean;
}

export interface ToolEvent {
  agentId: string;
  sessionId: string;
  hookEventName: string;
  toolName: string;
  toolInput: Record<string, unknown>;
  toolResponse?: Record<string, unknown>;
  timestamp: Date;

  // Enhanced fields for Phase 3
  toolCallId: string;
  status?: string; // "pending", "success", "failed"
  errorMessage?: string;
  executionTimeMs?: number;
}

export interface ToolCallStatistics {
  totalCalls: number;
  successfulCalls: number;
  failedCalls: number;
  pendingCalls: number;
  averageExecutionTimeMs: number;
  callsByTool: Record<string, number>;
}

export interface AgentInfo {
  id: string;
  working_dir: string;
  status: string;
  session_id: string | null;
  last_activity: number | null;
  is_processing: boolean;
  pending_input: boolean;
  github_context?: {
    repository_url: string;
    owner: string;
    repo: string;
    branch: string;
    commit_sha?: string;
    last_synced?: string;
  };
}

export interface AgentStatusEvent {
  agent_id: string;
  status: AgentStatus;
  info?: AgentInfo;
}

export interface AgentInputRequiredEvent {
  agent_id: string;
  last_output: string;
}

export interface AgentActivityEvent {
  agent_id: string;
  is_processing: boolean;
  pending_input: boolean;
  last_activity: number;
}

export interface AgentStatistics {
  agentId: string;
  totalPrompts: number;
  totalToolCalls: number;
  totalOutputBytes: number;
  sessionStart: string;
  lastActivity: string;
  totalTokensUsed?: number;
  totalCostUsd?: number;
}

export interface AgentStatsEvent {
  agent_id: string;
  stats: {
    agent_id: string;
    total_prompts: number;
    total_tool_calls: number;
    total_output_bytes: number;
    session_start: string;
    last_activity: string;
    total_tokens_used?: number;
    total_cost_usd?: number;
  };
}

// Chat-related types for meta-agent
export interface ChatMessage {
  role: "user" | "assistant";
  content: string;
  toolCalls?: ToolCall[];
  timestamp: number;
}

export interface ToolCall {
  id: string;
  toolName: string;
  input: Record<string, unknown>;
  output?: Record<string, unknown>;
}

export interface ChatResponse {
  message: ChatMessage;
  usage: ChatUsage;
}

export interface ChatUsage {
  input_tokens: number;
  output_tokens: number;
}

export interface MetaAgentToolCallEvent {
  tool_name: string;
  input: Record<string, unknown>;
  output: Record<string, unknown>;
  timestamp: number;
}

export interface MetaAgentThinkingEvent {
  is_thinking: boolean;
}

// Cost Tracking Types

export interface SessionCostRecord {
  sessionId: string;
  agentId: string;
  workingDir: string;
  startedAt: string;
  endedAt?: string;
  totalCostUsd: number;
  totalTokens: number;
  totalPrompts: number;
  totalToolCalls: number;
  modelUsage?: Record<string, ModelCostBreakdown>;
}

export interface ModelCostBreakdown {
  inputTokens: number;
  outputTokens: number;
  cacheCreationInputTokens: number;
  cacheReadInputTokens: number;
  costUsd: number;
}

export interface CostSummary {
  totalCostUsd: number;
  totalSessions: number;
  totalTokens: number;
  totalPrompts: number;
  totalToolCalls: number;
  sessionRecords: SessionCostRecord[];
  costByModel: Record<string, number>;
  costByWorkingDir: Record<string, number>;
}

export interface DateRangeCostSummary {
  startDate?: string;
  endDate?: string;
  totalCostUsd: number;
  sessionCount: number;
  dailyCosts: DailyCost[];
}

export interface DailyCost {
  date: string;
  costUsd: number;
  sessionCount: number;
}
