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
import { setCurrentPipelineId } from "../stores/agents";

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
 *
 * TODO: Re-engineer custom pipeline without pool_manager.
 * The old implementation used orchestrator/verification stubs that never actually worked.
 * This will be rebuilt with direct agent management.
 */
export async function createCustomPipeline(
  _params: CreateCustomPipelineParams
): Promise<CreatePipelineResult> {
  throw new Error(
    "Custom pipeline feature is being re-engineered. Use auto-pipeline (Ralphline) for now."
  );
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

  // Set current pipeline ID for orchestrator event persistence
  setCurrentPipelineId(pipelineId);

  // Start pipeline execution immediately
  await invoke("start_auto_pipeline", {
    pipelineId,
  });

  return pipelineId;
}
