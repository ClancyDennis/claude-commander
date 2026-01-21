/**
 * Agent Creation Service
 *
 * Handles the backend invocation logic for creating agents and pipelines.
 * Extracted from NewAgentDialog to keep the component focused on UI.
 */

import { invoke } from "@tauri-apps/api/core";
import type { Agent } from "../types";
import type { Pipeline } from "../stores/pipelines";
import type { AgentPipelineSettings } from "../stores/pipelineSettings";

export interface CreateSingleAgentParams {
  workingDir: string;
  githubUrl?: string;
  instructions: string[];
}

export interface CreateCustomPipelineParams {
  workingDir: string;
  task: string;
  settings: AgentPipelineSettings;
  instructions: string[];
}

export interface CreateAutoPipelineParams {
  workingDir: string;
  task: string;
  instructions: string[];
}

export interface CreateAgentResult {
  agent: Agent;
  agentId: string;
}

export interface CreatePipelineResult {
  pipeline: Pipeline;
  pipelineId: string;
}

/**
 * Creates a single agent with the given parameters.
 * Returns the agent ID and Agent object for store updates.
 */
export async function createSingleAgent(
  params: CreateSingleAgentParams
): Promise<CreateAgentResult> {
  const { workingDir, githubUrl, instructions } = params;

  const agentId = await invoke<string>("create_agent", {
    workingDir,
    githubUrl: githubUrl?.trim() || null,
    selectedInstructionFiles: instructions.length > 0 ? instructions : null,
  });

  const agent: Agent = {
    id: agentId,
    workingDir,
    status: "running",
    createdAt: new Date(),
  };

  return { agent, agentId };
}

/**
 * Creates a custom pipeline with detailed settings.
 * Builds the backend config from the pipeline settings and starts execution.
 */
export async function createCustomPipeline(
  params: CreateCustomPipelineParams
): Promise<CreatePipelineResult> {
  const { workingDir, task, settings } = params;

  // Build backend config from detailed settings
  const config = {
    // P-Thread
    use_agent_pool: settings.useAgentPool,
    pool_priority: settings.poolPriority,

    // B-Thread
    enable_orchestration: settings.enableOrchestration,
    auto_decompose: settings.autoDecompose,
    max_parallel_tasks: settings.maxParallelTasks,

    // F-Thread
    enable_verification: settings.enableVerification,
    verification_strategy: settings.verificationStrategy,
    verification_n: settings.verificationN,
    confidence_threshold: settings.confidenceThreshold,

    // C-Thread
    require_plan_review: settings.requirePlanReview,
    require_final_review: settings.requireFinalReview,
    auto_validation_command: settings.autoValidationCommand,
    auto_approve_on_verification: settings.autoApproveOnVerification,
  };

  // Create pipeline with task and config
  const pipelineId = await invoke<string>("create_pipeline", {
    userRequest: task.trim(),
    config,
  });

  // Start pipeline execution immediately
  await invoke("start_pipeline", {
    pipelineId,
    userRequest: task.trim(),
  });

  const pipeline: Pipeline = {
    id: pipelineId,
    workingDir,
    userRequest: task.trim(),
    status: "planning",
    createdAt: new Date(),
  };

  return { pipeline, pipelineId };
}

/**
 * Creates an auto-pipeline (Ralphline) with minimal configuration.
 * Uses the automated 3-step workflow.
 */
export async function createAutoPipeline(
  params: CreateAutoPipelineParams
): Promise<string> {
  const { workingDir, task } = params;

  const pipelineId = await invoke<string>("create_auto_pipeline", {
    userRequest: task.trim(),
    workingDir,
  });

  // Start pipeline execution immediately
  await invoke("start_auto_pipeline", {
    pipelineId,
  });

  return pipelineId;
}
