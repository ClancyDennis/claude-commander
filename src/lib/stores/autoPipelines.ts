import { writable } from 'svelte/store';
import type { AutoPipeline } from '../types';
import { selectedAgentId, selectedHistoricalRun } from './agents';
import { selectedPipelineId } from './pipelines';

export const autoPipelines = writable<Map<string, AutoPipeline>>(new Map());
export const selectedAutoPipelineId = writable<string | null>(null);

export function selectAutoPipeline(id: string | null) {
  selectedAutoPipelineId.set(id);
  if (id) {
    selectedAgentId.set(null);
    selectedPipelineId.set(null);
    selectedHistoricalRun.set(null);
  }
}
