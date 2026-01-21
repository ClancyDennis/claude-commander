/**
 * Skill Generation Hook
 *
 * Provides reactive state for skill generation progress and manages
 * the Tauri event listeners. Use in components that need to display
 * skill generation progress.
 *
 * Usage:
 *   const skillGen = useSkillGeneration();
 *   onMount(() => skillGen.start());
 *   // Access skillGen.state for reactive updates
 */

import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface SkillGenerationState {
  active: boolean;
  total: number;
  completed: number;
  skipped: number;
  failed: number;
  currentFile: string | null;
}

interface SkillStartedPayload {
  total: number;
}

interface SkillProgressPayload {
  file: string;
  completed: number;
  total: number;
}

interface SkillCompletedPayload {
  file: string;
  skill_name: string;
}

interface SkillSkippedPayload {
  file: string;
  skill_name: string;
}

interface SkillFailedPayload {
  file: string;
  error: string;
}

interface SkillGenerationCompletedPayload {
  completed: number;
  skipped: number;
  failed: number;
  total: number;
}

const initialState: SkillGenerationState = {
  active: false,
  total: 0,
  completed: 0,
  skipped: 0,
  failed: 0,
  currentFile: null,
};

export function useSkillGeneration() {
  let state = $state<SkillGenerationState>({ ...initialState });
  let unlisteners: UnlistenFn[] = [];

  /**
   * Start listening for skill generation events.
   * Call this in onMount. Returns a cleanup function.
   */
  async function start(): Promise<() => void> {
    unlisteners.push(
      await listen<SkillStartedPayload>("skill_generation:started", (event) => {
        state = {
          active: true,
          total: event.payload.total,
          completed: 0,
          skipped: 0,
          failed: 0,
          currentFile: null,
        };
      })
    );

    unlisteners.push(
      await listen<SkillProgressPayload>("skill_generation:progress", (event) => {
        state.currentFile = event.payload.file;
      })
    );

    unlisteners.push(
      await listen<SkillCompletedPayload>("skill_generation:skill_completed", () => {
        state.completed += 1;
        state.currentFile = null;
      })
    );

    unlisteners.push(
      await listen<SkillSkippedPayload>("skill_generation:skill_skipped", () => {
        state.skipped += 1;
      })
    );

    unlisteners.push(
      await listen<SkillFailedPayload>("skill_generation:skill_failed", () => {
        state.failed += 1;
        state.currentFile = null;
      })
    );

    unlisteners.push(
      await listen<SkillGenerationCompletedPayload>("skill_generation:completed", () => {
        state.active = false;
      })
    );

    return cleanup;
  }

  /**
   * Clean up all event listeners.
   */
  function cleanup() {
    unlisteners.forEach((unlisten) => unlisten());
    unlisteners = [];
  }

  /**
   * Reset state to initial values.
   */
  function reset() {
    state = { ...initialState };
  }

  return {
    get state() {
      return state;
    },
    start,
    cleanup,
    reset,
  };
}
