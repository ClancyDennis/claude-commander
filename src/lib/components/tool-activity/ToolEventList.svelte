<script lang="ts">
  import type { ToolEvent } from "../../types";
  import ToolEventItem from "./ToolEventItem.svelte";

  // Virtualization settings
  const VISIBLE_ITEMS = 20;
  const ITEM_HEIGHT = 100; // Approximate height of each tool event card

  let {
    filteredTools,
    allToolsCount,
    onContainerBind,
    onScroll,
    scrollTop = 0,
  }: {
    filteredTools: ToolEvent[];
    allToolsCount: number;
    onContainerBind: (el: HTMLDivElement | null) => void;
    onScroll: (e: Event) => void;
    scrollTop: number;
  } = $props();

  // Virtualization: calculate visible items based on scroll position
  const visibleTools = $derived.by(() => {
    if (filteredTools.length <= VISIBLE_ITEMS) {
      return {
        visible: filteredTools,
        startIndex: 0,
        totalHeight: filteredTools.length * ITEM_HEIGHT,
      };
    }

    const startIndex = Math.max(0, Math.floor(scrollTop / ITEM_HEIGHT) - 5);
    const endIndex = Math.min(filteredTools.length, startIndex + VISIBLE_ITEMS + 10);

    return {
      visible: filteredTools.slice(startIndex, endIndex),
      startIndex,
      totalHeight: filteredTools.length * ITEM_HEIGHT,
    };
  });

  function handleScroll(e: Event) {
    onScroll(e);
  }

  // Bind the container element to parent
  let containerRef: HTMLDivElement | null = $state(null);

  $effect(() => {
    onContainerBind(containerRef);
  });
</script>

<div class="events" bind:this={containerRef} onscroll={handleScroll}>
  <div class="virtual-spacer" style="height: {visibleTools.totalHeight}px; position: relative;">
    <div style="position: absolute; top: {visibleTools.startIndex * ITEM_HEIGHT}px; width: 100%;">
      {#each visibleTools.visible as event, i (event.toolCallId + (visibleTools.startIndex + i))}
        <ToolEventItem {event} />
      {/each}
    </div>
  </div>

  {#if filteredTools.length === 0 && allToolsCount > 0}
    <div class="empty">
      <svg class="empty-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <circle cx="11" cy="11" r="8"/>
        <line x1="21" y1="21" x2="16.65" y2="16.65"/>
      </svg>
      <p class="empty-title">No matching tools</p>
      <p class="empty-hint">Try adjusting your filters</p>
    </div>
  {/if}

  {#if allToolsCount === 0}
    <div class="empty">
      <svg class="empty-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/>
      </svg>
      <p class="empty-title">No tool activity</p>
      <p class="empty-hint">Tool usage will appear here</p>
    </div>
  {/if}
</div>

<style>
  .events {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-2) var(--space-3);
  }

  /* Empty State */
  .empty {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-8);
    text-align: center;
    gap: var(--space-2);
  }

  .empty-icon {
    width: 40px;
    height: 40px;
    color: var(--text-muted);
    margin-bottom: var(--space-2);
  }

  .empty-title {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--text-secondary);
    margin: 0;
  }

  .empty-hint {
    font-size: var(--text-xs);
    color: var(--text-muted);
    margin: 0;
  }
</style>
