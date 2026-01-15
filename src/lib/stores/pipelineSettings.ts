import { writable } from 'svelte/store';

/**
 * Pipeline Settings Store
 *
 * This store manages per-agent pipeline configuration. Settings control:
 * - Phase A (Agent Pool): Shared agent pool for parallel task execution
 * - Phase B (Orchestration): Automatic task decomposition and workflow management
 * - Phase C (Verification): Best-of-N verification with multiple agents
 * - Phase D (Checkpoints): Human approval gates and validation
 *
 * Settings Flow:
 * 1. User configures settings via AgentSettings.svelte UI
 * 2. Settings stored in localStorage (persistent)
 * 3. NewAgentDialog.svelte reads settings via getAgentSettings()
 * 4. Settings passed to backend create_pipeline command
 * 5. Backend applies settings to PipelineConfig
 *
 * Safety Defaults:
 * - All expensive features (pool, orchestration, verification) disabled by default
 * - Only human checkpoints enabled (safe, adds manual control)
 * - Conservative parallelism (maxParallelTasks: 3)
 * - Minimal verification (verificationN: 1)
 */

export interface AgentPipelineSettings {
  agentId: string;

  // Pipeline features
  enablePipeline: boolean;

  // Phase A: Agent Pool
  useAgentPool: boolean;
  poolPriority: 'low' | 'normal' | 'high';

  // Phase B: Task Orchestration
  enableOrchestration: boolean;
  autoDecompose: boolean;
  maxParallelTasks: number;

  // Phase C: Verification (F-Thread)
  enableVerification: boolean;
  verificationStrategy: 'majority' | 'weighted' | 'meta' | 'first';
  verificationN: number;
  confidenceThreshold: number;

  // Phase D: Checkpoints (C-Thread)
  enableCheckpoints: boolean;
  requirePlanReview: boolean;
  requireFinalReview: boolean;
  autoValidationCommand: string;
  autoApproveOnVerification: boolean;

  // Instruction Files
  instructionFiles: string[];  // Array of relative paths to enabled files
}

export const defaultSettings: AgentPipelineSettings = {
  agentId: '',

  // MASTER KILL SWITCH - Pipeline disabled by default for safety
  enablePipeline: false,

  // Phase A: Agent Pool - Disabled until tested
  useAgentPool: false,
  poolPriority: 'normal',

  // Phase B: Task Orchestration - Disabled until tested
  enableOrchestration: false,
  autoDecompose: false,
  maxParallelTasks: 3,  // Reduced from 5 to 3 for conservative testing

  // Phase C: Verification - Disabled until tested
  enableVerification: false,
  verificationStrategy: 'weighted',
  verificationN: 1,  // Minimal: reduced from 3 to 1
  confidenceThreshold: 0.8,

  // Phase D: Checkpoints - Enabled (human gates are safe)
  enableCheckpoints: true,
  requirePlanReview: true,
  requireFinalReview: true,
  autoValidationCommand: 'cargo check',
  autoApproveOnVerification: false,

  // Instruction Files
  instructionFiles: [],
};

// Map of agent ID to settings
export const agentPipelineSettings = writable<Map<string, AgentPipelineSettings>>(new Map());

// Get settings for an agent (returns default if not found)
export function getAgentSettings(agentId: string): AgentPipelineSettings {
  let settings: AgentPipelineSettings | undefined;
  agentPipelineSettings.subscribe(map => {
    settings = map.get(agentId);
  })();

  if (!settings) {
    return { ...defaultSettings, agentId };
  }

  return settings;
}

// Update settings for an agent
export function updateAgentSettings(agentId: string, updates: Partial<AgentPipelineSettings>) {
  agentPipelineSettings.update(map => {
    const current = map.get(agentId) || { ...defaultSettings, agentId };
    map.set(agentId, { ...current, ...updates });
    return map;
  });
}

// Save to localStorage
export function saveSettings() {
  agentPipelineSettings.subscribe(map => {
    const obj = Object.fromEntries(map);
    localStorage.setItem('agent_pipeline_settings', JSON.stringify(obj));
  })();
}

// Load from localStorage
export function loadSettings() {
  const stored = localStorage.getItem('agent_pipeline_settings');
  if (stored) {
    try {
      const obj = JSON.parse(stored);
      const map = new Map(Object.entries(obj));
      agentPipelineSettings.set(map as Map<string, AgentPipelineSettings>);
    } catch (e) {
      console.error('Failed to load pipeline settings:', e);
    }
  }
}
