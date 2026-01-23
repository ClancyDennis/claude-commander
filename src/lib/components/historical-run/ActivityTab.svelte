<script lang="ts">
  import type {
    OrchestratorToolCall,
    OrchestratorStateChange,
    OrchestratorDecision
  } from "$lib/types";
  import { ToolCallList, StateChangeList, DecisionList } from '../orchestrator';

  export type ActivitySubtab = 'tools' | 'states' | 'decisions';

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
    padding: var(--space-sm);
    gap: var(--space-sm);
    border-bottom: 1px solid var(--border);
    background: var(--bg-tertiary);
  }

  .subtab {
    padding: 8px 14px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: transparent;
    color: var(--text-secondary);
    font-size: 12px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .subtab:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .subtab.active {
    background: var(--accent);
    border-color: var(--accent);
    color: white;
  }

  .activity-list {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-md);
  }

  /* Loading and empty states */
  .loading,
  .error-message,
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-xl);
    gap: var(--space-md);
    color: var(--text-muted);
    flex: 1;
    text-align: center;
  }

  .empty-state .hint {
    font-size: 13px;
    opacity: 0.7;
  }

  .error-message svg,
  .empty-state svg {
    width: 48px;
    height: 48px;
    opacity: 0.5;
  }

  .spinner {
    width: 36px;
    height: 36px;
    border: 3px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
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
    background: var(--border);
    border-radius: 3px;
  }

  .activity-list::-webkit-scrollbar-thumb:hover {
    background: var(--accent);
  }
</style>
