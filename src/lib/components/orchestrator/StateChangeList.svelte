<script lang="ts">
  import { formatTimeLocale } from '$lib/utils/formatting';
  import type { OrchestratorStateChange } from '$lib/types';

  // Props
  let {
    stateChanges = [],
    scrollTop = 0,
  }: {
    stateChanges: OrchestratorStateChange[];
    scrollTop?: number;
  } = $props();

  // Virtualization settings
  const VISIBLE_ITEMS = 20;
  const ITEM_HEIGHT = 80;

  // Calculate visible window for virtualization
  function getVisibleItems(items: OrchestratorStateChange[]): { visible: OrchestratorStateChange[]; startIndex: number; totalHeight: number } {
    if (items.length <= VISIBLE_ITEMS) {
      return { visible: items, startIndex: 0, totalHeight: items.length * ITEM_HEIGHT };
    }

    const startIndex = Math.max(0, Math.floor(scrollTop / ITEM_HEIGHT) - 5);
    const endIndex = Math.min(items.length, startIndex + VISIBLE_ITEMS + 10);

    return {
      visible: items.slice(startIndex, endIndex),
      startIndex,
      totalHeight: items.length * ITEM_HEIGHT
    };
  }

  let visibleItems = $derived(getVisibleItems(stateChanges));

  // State badge colors
  function getStateBadgeClass(state: string): string {
    if (state.includes("Completed")) return "badge-success";
    if (state.includes("Failed") || state.includes("GaveUp")) return "badge-error";
    if (state.includes("Running") || state.includes("Executing") || state.includes("Verifying")) return "badge-info";
    if (state.includes("Planning") || state.includes("Ready")) return "badge-warning";
    return "badge-neutral";
  }
</script>

<div class="state-list" style="height: {visibleItems.totalHeight}px; position: relative;">
  {#if stateChanges.length === 0}
    <div class="empty-state">No state changes yet</div>
  {:else}
    <div style="position: absolute; top: {visibleItems.startIndex * ITEM_HEIGHT}px; width: 100%;">
      {#each visibleItems.visible as change, i (visibleItems.startIndex + i)}
        <div class="state-item">
          <div class="state-transition">
            <span class="state-badge {getStateBadgeClass(change.old_state)}">
              {change.old_state}
            </span>
            <span class="arrow">→</span>
            <span class="state-badge {getStateBadgeClass(change.new_state)}">
              {change.new_state}
            </span>
          </div>
          <div class="state-meta">
            <span class="iteration">Iteration {change.iteration}</span>
            <span class="time">{formatTimeLocale(change.timestamp)}</span>
          </div>
          {#if change.generated_skills > 0 || change.generated_subagents > 0 || change.claudemd_generated}
            <div class="state-resources">
              {#if change.generated_skills > 0}
                <span class="resource">🎯 {change.generated_skills} skills</span>
              {/if}
              {#if change.generated_subagents > 0}
                <span class="resource">🤖 {change.generated_subagents} subagents</span>
              {/if}
              {#if change.claudemd_generated}
                <span class="resource">📝 CLAUDE.md</span>
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .state-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .empty-state {
    text-align: center;
    padding: 24px;
    color: var(--text-muted, #666);
    font-size: 13px;
  }

  .state-item {
    background: var(--bg-primary, #0f0f13);
    border: 1px solid var(--border, rgba(240, 112, 90, 0.15));
    border-radius: 8px;
    padding: 10px 12px;
  }

  .state-transition {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .arrow {
    color: var(--text-muted, #666);
    font-size: 14px;
  }

  .state-badge {
    font-size: 11px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 6px;
    text-transform: uppercase;
    letter-spacing: 0.3px;
  }

  .badge-success {
    background: rgba(34, 197, 94, 0.2);
    color: #22c55e;
  }

  .badge-error {
    background: rgba(239, 68, 68, 0.2);
    color: #ef4444;
  }

  .badge-info {
    background: rgba(59, 130, 246, 0.2);
    color: #3b82f6;
  }

  .badge-warning {
    background: rgba(234, 179, 8, 0.2);
    color: #eab308;
  }

  .badge-neutral {
    background: rgba(156, 163, 175, 0.2);
    color: #9ca3af;
  }

  .state-meta {
    display: flex;
    justify-content: space-between;
    margin-top: 8px;
    font-size: 11px;
    color: var(--text-muted, #666);
  }

  .state-resources {
    display: flex;
    gap: 12px;
    margin-top: 8px;
    font-size: 11px;
  }

  .resource {
    color: var(--text-secondary, #999);
  }
</style>
