import { writable, derived } from 'svelte/store';

export interface Pipeline {
  id: string;
  workingDir: string;
  userRequest: string;
  status: 'planning' | 'implementing' | 'testing' | 'reviewing' | 'completed' | 'failed';
  currentPhase?: string;
  createdAt: Date;
  lastActivity?: Date;
}

export interface PhaseProgressData {
  pipelineId: string;
  phases: Array<{
    name: string;
    status: 'pending' | 'in_progress' | 'completed' | 'failed';
    checkpointType?: string;
    checkpointStatus?: string;
    startTime?: number;
    endTime?: number;
  }>;
  currentPhaseIndex: number;
}

// Map of pipeline ID to pipeline data
export const pipelines = writable<Map<string, Pipeline>>(new Map());

// Map of pipeline ID to phase progress
export const pipelineProgress = writable<Map<string, PhaseProgressData>>(new Map());

// Selected pipeline ID (for viewing pipeline details)
export const selectedPipelineId = writable<string | null>(null);

// Derived store: get selected pipeline
export const selectedPipeline = derived(
  [pipelines, selectedPipelineId],
  ([$pipelines, $selectedPipelineId]) => {
    if (!$selectedPipelineId) return null;
    return $pipelines.get($selectedPipelineId) || null;
  }
);

// Add a new pipeline
export function addPipeline(pipeline: Pipeline) {
  pipelines.update((map) => {
    map.set(pipeline.id, pipeline);
    return map;
  });
}

// Update pipeline status
export function updatePipelineStatus(pipelineId: string, status: Pipeline['status']) {
  pipelines.update((map) => {
    const pipeline = map.get(pipelineId);
    if (pipeline) {
      pipeline.status = status;
      pipeline.lastActivity = new Date();
      map.set(pipelineId, pipeline);
    }
    return map;
  });
}

// Update pipeline phase
export function updatePipelinePhase(pipelineId: string, phase: string) {
  pipelines.update((map) => {
    const pipeline = map.get(pipelineId);
    if (pipeline) {
      pipeline.currentPhase = phase;
      pipeline.lastActivity = new Date();
      map.set(pipelineId, pipeline);
    }
    return map;
  });
}

// Update pipeline activity
export function updatePipelineActivity(pipelineId: string, updates: Partial<Pipeline>) {
  pipelines.update((map) => {
    const pipeline = map.get(pipelineId);
    if (pipeline) {
      Object.assign(pipeline, updates);
      pipeline.lastActivity = new Date();
      map.set(pipelineId, pipeline);
    }
    return map;
  });
}

// Update phase progress
export function updatePhaseProgress(data: PhaseProgressData) {
  pipelineProgress.update((map) => {
    map.set(data.pipelineId, data);
    return map;
  });
}

// Remove a pipeline
export function removePipeline(pipelineId: string) {
  pipelines.update((map) => {
    map.delete(pipelineId);
    return map;
  });
  pipelineProgress.update((map) => {
    map.delete(pipelineId);
    return map;
  });
}

// Open a pipeline (select it)
export function openPipeline(pipelineId: string) {
  selectedPipelineId.set(pipelineId);
}
