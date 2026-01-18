import { writable } from 'svelte/store';
import type { AutoPipeline } from '../types';
import { selectedAgentId } from './agents';

export const autoPipelines = writable<Map<string, AutoPipeline>>(new Map());
export const selectedAutoPipelineId = writable<string | null>(null);

export function selectAutoPipeline(id: string | null) {
  selectedAutoPipelineId.set(id);
  if (id) {
    selectedAgentId.set(null);
  }
}
