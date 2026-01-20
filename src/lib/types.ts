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

export interface AgentOutputEvent {
  agent_id: string;
  output_type: string;
  content: string;
  parsed_json?: Record<string, unknown>;
  metadata?: OutputMetadata;
  // Enhanced fields matching Rust struct
  session_id?: string;
  uuid?: string;
  parent_tool_use_id?: string;
  subtype?: string;
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

export interface InstructionFileInfo {
  id: string;
  name: string;
  path: string;
  relativePath: string;
  fileType: 'txt' | 'md';
  size: number;
  modified: string;
  hasSkill?: boolean;      // Whether a skill was generated from this file
  skillName?: string;      // Name of the generated skill
}

// ============================================================================
// Instruction Analysis Types (LLM-assisted editing)
// ============================================================================

export type IssueSeverity = "critical" | "warning" | "info";
export type SuggestionCategory = "clarity" | "structure" | "completeness" | "specificity" | "formatting" | "other";
export type SuggestionStatus = "pending" | "accepted" | "rejected";

export interface InstructionIssue {
  id: string;
  severity: IssueSeverity;
  title: string;
  description: string;
  lineStart?: number;
  lineEnd?: number;
}

export interface InstructionSuggestion {
  id: string;
  category: SuggestionCategory;
  title: string;
  description: string;
  originalText?: string;
  suggestedText: string;
  lineStart?: number;
  lineEnd?: number;
}

export interface InstructionAnalysisResult {
  qualityScore: number;         // 1-10
  qualitySummary: string;
  issues: InstructionIssue[];
  suggestions: InstructionSuggestion[];
  improvedContent?: string;
}

export interface GeneratedSkill {
  skillName: string;
  skillPath: string;
  sourceFile: string;
  generatedAt: string;
}

export interface SkillContent {
  skillName: string;
  skillMd: string;
  referenceMd?: string;
  examplesMd?: string;
  scripts: SkillScript[];
}

export interface SkillScript {
  name: string;
  content: string;
  language: string;
}

// Agent Run History Types
export type RunStatus = "running" | "completed" | "stopped" | "crashed" | "waiting_input";

export interface AgentRun {
  id?: number;
  agent_id: string;
  session_id?: string;
  working_dir: string;
  github_url?: string;
  github_context?: string;
  source: string;
  status: RunStatus;
  started_at: number;
  ended_at?: number;
  last_activity: number;
  initial_prompt?: string;
  error_message?: string;
  total_prompts: number;
  total_tool_calls: number;
  total_output_bytes: number;
  total_tokens_used?: number;
  total_cost_usd?: number;
  model_usage?: string;
  can_resume: boolean;
  resume_data?: string;
}

export interface ModelCostBreakdown {
  input_tokens: number;
  output_tokens: number;
  cache_creation_input_tokens: number;
  cache_read_input_tokens: number;
  cost_usd: number;
}

export interface RunStats {
  total_runs: number;
  by_status: [string, number][];
  by_source: [string, number][];
  total_cost_usd: number;
  resumable_runs: number;
}

export interface AutoPipeline {
  id: string;
  user_request: string;
  working_dir: string;
  status: 'running' | 'completed' | 'failed';
  steps: AutoPipelineStep[];
  questions: string[];
  answers: string[];
  created_at: string;
  completed_at?: string;
}

export interface AutoPipelineStep {
  step_number: number;
  role: 'Planning' | 'Building' | 'Verifying';
  agent_id?: string;
  status: 'Pending' | 'Running' | 'Completed' | 'Failed';
  output?: {
    raw_text: string;
    structured_data?: any;
    agent_outputs?: AgentOutputEvent[];
  };
  started_at?: string;
  completed_at?: string;
}

// Orchestrator activity types
export interface OrchestratorToolCall {
  tool_name: string;
  tool_input: Record<string, unknown>;
  is_error?: boolean;
  summary?: string;
  current_state: string;
  iteration: number;
  timestamp: number;
}

export interface OrchestratorStateChange {
  old_state: string;
  new_state: string;
  iteration: number;
  generated_skills: number;
  generated_subagents: number;
  claudemd_generated: boolean;
  timestamp: number;
}

export interface OrchestratorDecision {
  pipeline_id: string;
  decision: 'Complete' | 'Iterate' | 'Replan' | 'GiveUp';
  reasoning: string;
  issues: string[];
  suggestions: string[];
  timestamp: number;
}

// Security Types
export type SecurityAlertSeverity = "low" | "medium" | "high" | "critical";

export interface SecurityAlertPayload {
  alert_id: string;
  agent_id: string;
  risk_level: SecurityAlertSeverity;
  title: string;
  description: string;
  affected_agents: string[];
  recommended_actions: string[];
  timestamp: number;
}

export interface SecurityAlert {
  agentId: string;
  alertId: string;
  severity: SecurityAlertSeverity;
  title: string;
  description: string;
  timestamp: Date;
}

// Agent terminated by security system
export interface SecurityAgentTerminatedPayload {
  agent_id: string;
  batch_id: string;
  reason: string;
}

// Agent suspended by security system
export interface SecurityAgentSuspendedPayload {
  agent_id: string;
  batch_id: string;
  reason: string;
}

// Action requires human review
export interface SecurityPendingReviewPayload {
  id: string;
  batch_id: string;
  analysis_summary: string;
  overall_risk_level: SecurityAlertSeverity;
  recommended_action: string;
  agent_id?: string;
  created_at: number;
}

// Review completed
export interface SecurityReviewCompletedPayload {
  review_id: string;
  approved: boolean;
}

// Pending review for UI tracking
export interface PendingSecurityReview {
  id: string;
  batchId: string;
  summary: string;
  riskLevel: SecurityAlertSeverity;
  recommendedAction: string;
  agentId?: string;
  createdAt: Date;
}
