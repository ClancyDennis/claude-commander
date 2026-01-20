/**
 * Pipeline Helpers Service
 *
 * Utility functions for pipeline operations, including fetching
 * auto-pipeline data from the backend.
 */

import { invoke } from "@tauri-apps/api/core";
import type { AutoPipeline } from "$lib/types";

/**
 * Fetches an auto-pipeline from the backend by ID.
 * Returns null if the pipeline cannot be found or an error occurs.
 */
export async function fetchAutoPipeline(pipelineId: string): Promise<AutoPipeline | null> {
  try {
    const pipeline = await invoke<AutoPipeline>("get_auto_pipeline", {
      pipelineId
    });
    return pipeline;
  } catch (err) {
    console.error("[Frontend] Failed to fetch pipeline:", err);
    return null;
  }
}

/**
 * Fetches an auto-pipeline and updates the store.
 * This is a convenience function that combines fetching and store update.
 */
export async function fetchAndUpdateAutoPipeline(
  pipelineId: string,
  updateStore: (pipeline: AutoPipeline) => void
): Promise<void> {
  const pipeline = await fetchAutoPipeline(pipelineId);
  if (pipeline) {
    updateStore(pipeline);
  }
}

/**
 * Creates a store updater function for auto-pipelines.
 * This is used to create callbacks for event handlers that need to
 * update the autoPipelines store.
 */
export function createAutoPipelineUpdater(
  autoPipelinesUpdate: (updater: (m: Map<string, AutoPipeline>) => Map<string, AutoPipeline>) => void
) {
  return (pipeline: AutoPipeline) => {
    autoPipelinesUpdate((m) => {
      m.set(pipeline.id, pipeline);
      return new Map(m);
    });
  };
}

/**
 * Fetches and updates an auto-pipeline in the store.
 * This is the pattern that was duplicated 4 times in App.svelte.
 */
export async function refreshAutoPipeline(
  pipelineId: string,
  autoPipelinesUpdate: (updater: (m: Map<string, AutoPipeline>) => Map<string, AutoPipeline>) => void
): Promise<void> {
  const pipeline = await fetchAutoPipeline(pipelineId);
  if (pipeline) {
    autoPipelinesUpdate((m) => {
      m.set(pipeline.id, pipeline);
      return new Map(m);
    });
  }
}
