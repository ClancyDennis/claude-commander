import { writable } from 'svelte/store';
import type { AutoPipeline } from '$lib/types';

export const autoPipelines = writable<Map<string, AutoPipeline>>(new Map());
