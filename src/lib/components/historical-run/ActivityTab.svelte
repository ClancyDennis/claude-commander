<script lang="ts">
  import type {
    OrchestratorToolCall,
    OrchestratorStateChange,
    OrchestratorDecision
  } from "$lib/types";
  import { ToolCallList, StateChangeList, DecisionList } from '../orchestrator';
  import type { ActivitySubtab } from './types';

  interface Props {
    loading: boolean;
    error: string | null;
    hasPipeline: boolean;
    toolCalls: OrchestratorToolCall[];
    stateChanges: OrchestratorStateChange[];
    decisions: OrchestratorDecision[];
    scrollTop: number;
  }

  let {
    loading,
    error,
    hasPipeline,
    toolCalls,
    stateChanges,
    decisions,
    scrollTop
  }: Props = $props();

  // Local state for activity subtab
  let activitySubtab = $state<ActivitySubtab>('tools');

  // Check if there's any activity
  let hasActivity = $derived(
    toolCalls.length > 0 || stateChanges.length > 0 || decisions.length > 0
  );
</script>

<div class="activity-content">
  {#if loading}
    <div class="loading">
      <div class="spinner"></div>
      <p>Loading activity...</p>
    </div>
  {:else if error}
    <div class="error-message">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <line x1="12" y1="8" x2="12" y2="12"/>
        <line x1="12" y1="16" x2="12.01" y2="16"/>
      </svg>
      <p>{error}</p>
    </div>
  {:else if !hasPipeline}
    <div class="empty-state">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M9 17H5a2 2 0 0 0-2 2 2 2 0 0 0 2 2h2a2 2 0 0 0 2-2zm12-2h-4a2 2 0 0 0-2 2 2 2 0 0 0 2 2h2a2 2 0 0 0 2-2z"/>
        <circle cx="9" cy="7" r="4"/>
        <path d="M9 11a4 4 0 0 0 4 4"/>
      </svg>
      <p>No orchestrator activity available</p>
      <p class="hint">This run was not created by an orchestrator pipeline</p>
    </div>
  {:else if !hasActivity}
    <div class="empty-state">
      <p>No activity recorded for this pipeline</p>
    </div>
  {:else}
    <div class="activity-subtabs">
      <button
        class="subtab"
        class:active={activitySubtab === 'tools'}
        onclick={() => activitySubtab = 'tools'}
      >
        Tools ({toolCalls.length})
      </button>
      <button
        class="subtab"
        class:active={activitySubtab === 'states'}
        onclick={() => activitySubtab = 'states'}
      >
        States ({stateChanges.length})
      </button>
      <button
        class="subtab"
        class:active={activitySubtab === 'decisions'}
        onclick={() => activitySubtab = 'decisions'}
      >
        Decisions ({decisions.length})
      </button>
    </div>

    <div class="activity-list">
      {#if activitySubtab === 'tools'}
        <ToolCallList {toolCalls} {scrollTop} />
      {:else if activitySubtab === 'states'}
        <StateChangeList {stateChanges} {scrollTop} />
      {:else if activitySubtab === 'decisions'}
        <DecisionList {decisions} {scrollTop} />
      {/if}
    </div>
  {/if}
</div>

<style>
  .activity-content {
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .activity-subtabs {
    display: flex;
    padding: var(--space-2);
    gap: var(--space-2);
    border-bottom: 1px solid var(--border-hex);
    background: var(--bg-tertiary);
  }

  .subtab {
    /* Override global button styles */
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-secondary);
    font-size: var(--text-xs);
    font-weight: var(--font-medium);
    line-height: var(--leading-normal);
    cursor: pointer;
    transition: background var(--transition-fast), color var(--transition-fast), border-color var(--transition-fast);
    box-shadow: none;
  }

  .subtab:hover {
    background: rgba(255, 255, 255, 0.05);
    color: var(--text-primary);
    box-shadow: none;
  }

  .subtab:active {
    transform: none;
  }

  .subtab.active {
    background: var(--accent-hex);
    border-color: var(--accent-hex);
    color: white;
  }

  .subtab.active:hover {
    background: var(--accent-hover);
  }

  .activity-list {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-4);
  }

  /* Loading and empty states */
  .loading,
  .error-message,
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-8);
    gap: var(--space-4);
    color: var(--text-muted);
    flex: 1;
    text-align: center;
  }

  .loading p,
  .error-message p,
  .empty-state p {
    font-size: var(--text-sm);
    margin: 0;
  }

  .empty-state .hint {
    font-size: var(--text-xs);
    opacity: 0.7;
  }

  .error-message svg,
  .empty-state svg {
    width: 40px;
    height: 40px;
    opacity: 0.5;
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 2px solid var(--border-hex);
    border-top-color: var(--accent-hex);
    border-radius: var(--radius-full);
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  /* Scrollbar styles */
  .activity-list::-webkit-scrollbar {
    width: 6px;
  }

  .activity-list::-webkit-scrollbar-track {
    background: transparent;
  }

  .activity-list::-webkit-scrollbar-thumb {
    background: var(--border-hex);
    border-radius: 3px;
  }

  .activity-list::-webkit-scrollbar-thumb:hover {
    background: var(--accent-hex);
  }
</style>
