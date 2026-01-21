import { writable, derived } from 'svelte/store';

// Tracks whether a resize operation is currently in progress
const resizeInProgress = writable(false);

// Counter for debouncing - allows effects to know when resize has settled
const resizeGeneration = writable(0);

let resizeTimeoutId: ReturnType<typeof setTimeout> | null = null;
let isInitialized = false;

/**
 * Initialize resize tracking. Call once in App.svelte onMount.
 * Returns a cleanup function.
 */
export function initResizeTracking(): () => void {
  if (isInitialized) {
    return () => {};
  }
  isInitialized = true;

  const handleResizeStart = () => {
    resizeInProgress.set(true);

    // Clear any pending timeout
    if (resizeTimeoutId) {
      clearTimeout(resizeTimeoutId);
    }

    // Set a timeout to mark resize as complete after a pause
    resizeTimeoutId = setTimeout(() => {
      resizeInProgress.set(false);
      resizeGeneration.update(n => n + 1);
    }, 150); // 150ms after last resize event
  };

  window.addEventListener('resize', handleResizeStart);

  return () => {
    isInitialized = false;
    window.removeEventListener('resize', handleResizeStart);
    if (resizeTimeoutId) {
      clearTimeout(resizeTimeoutId);
    }
  };
}

/**
 * Store that is true while a resize is in progress.
 * Use this to skip expensive DOM operations during resize.
 */
export const isResizing = { subscribe: resizeInProgress.subscribe };

/**
 * Store that increments when resize settles.
 * Use this as a dependency to re-run effects after resize completes.
 */
export const resizeSettled = { subscribe: resizeGeneration.subscribe };

/**
 * Derived store that is true when it's safe to do layout-dependent operations.
 * True when NOT resizing.
 */
export const canMeasure = derived(resizeInProgress, $resizing => !$resizing);
