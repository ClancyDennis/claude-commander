<script lang="ts">
  import { formatTimeLocale } from '$lib/utils/formatting';
  import type { OrchestratorDecision } from '$lib/types';

  // Props
  let {
    decisions = [],
    scrollTop = 0,
  }: {
    decisions: OrchestratorDecision[];
    scrollTop?: number;
  } = $props();

  // Virtualization settings
  const VISIBLE_ITEMS = 10;
  const ITEM_HEIGHT = 150;

  // Calculate visible window for virtualization
  function getVisibleItems(items: OrchestratorDecision[]): { visible: OrchestratorDecision[]; startIndex: number; totalHeight: number } {
    if (items.length <= VISIBLE_ITEMS) {
      return { visible: items, startIndex: 0, totalHeight: items.length * ITEM_HEIGHT };
    }

    const startIndex = Math.max(0, Math.floor(scrollTop / ITEM_HEIGHT) - 2);
    const endIndex = Math.min(items.length, startIndex + VISIBLE_ITEMS + 4);

    return {
      visible: items.slice(startIndex, endIndex),
      startIndex,
      totalHeight: items.length * ITEM_HEIGHT
    };
  }

  let visibleItems = $derived(getVisibleItems(decisions));

  // Decision badge colors
  function getDecisionBadgeClass(decision: string): string {
    if (decision === "Complete") return "badge-success";
    if (decision === "Iterate") return "badge-warning";
    if (decision === "Replan") return "badge-info";
    if (decision === "GiveUp") return "badge-error";
    return "badge-neutral";
  }
</script>

<div class="decision-list" style="height: {visibleItems.totalHeight}px; position: relative;">
  {#if decisions.length === 0}
    <div class="empty-state">No decisions yet</div>
  {:else}
    <div style="position: absolute; top: {visibleItems.startIndex * ITEM_HEIGHT}px; width: 100%;">
      {#each visibleItems.visible as decision, i (visibleItems.startIndex + i)}
        <div class="decision-item">
          <div class="decision-header">
            <span class="decision-badge {getDecisionBadgeClass(decision.decision)}">
              {decision.decision}
            </span>
            <span class="decision-time">{formatTimeLocale(decision.timestamp)}</span>
          </div>
          <div class="decision-reasoning">{decision.reasoning}</div>
          {#if decision.issues.length > 0}
            <div class="decision-issues">
              <strong>Issues:</strong>
              <ul>
                {#each decision.issues as issue}
                  <li>{issue}</li>
                {/each}
              </ul>
            </div>
          {/if}
          {#if decision.suggestions.length > 0}
            <div class="decision-suggestions">
              <strong>Suggestions:</strong>
              <ul>
                {#each decision.suggestions as suggestion}
                  <li>{suggestion}</li>
                {/each}
              </ul>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .decision-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .empty-state {
    text-align: center;
    padding: 24px;
    color: var(--text-muted, #666);
    font-size: 13px;
  }

  .decision-item {
    background: var(--bg-primary, #0f0f13);
    border: 1px solid var(--border, rgba(240, 112, 90, 0.15));
    border-radius: 8px;
    padding: 12px;
  }

  .decision-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }

  .decision-badge {
    font-size: 12px;
    font-weight: 600;
    padding: 4px 12px;
    border-radius: 6px;
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

  .decision-time {
    font-size: 11px;
    color: var(--text-muted, #666);
  }

  .decision-reasoning {
    font-size: 13px;
    color: var(--text-primary, #e0e0e0);
    line-height: 1.5;
    margin-bottom: 10px;
  }

  .decision-issues,
  .decision-suggestions {
    font-size: 12px;
    margin-top: 8px;
  }

  .decision-issues strong,
  .decision-suggestions strong {
    color: var(--text-secondary, #999);
  }

  .decision-issues ul,
  .decision-suggestions ul {
    margin: 4px 0 0 16px;
    padding: 0;
    list-style: disc;
  }

  .decision-issues li,
  .decision-suggestions li {
    color: var(--text-primary, #e0e0e0);
    margin-bottom: 2px;
  }
</style>
