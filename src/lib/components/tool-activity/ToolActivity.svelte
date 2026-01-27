<script lang="ts">
  import { selectedAgentTools } from "../../stores/agents";
  import { isResizing } from "$lib/stores/resize";
  import {
    calculateToolStats,
    getUniqueToolNames,
    filterToolEvents,
    type FilterType,
  } from "$lib/utils/toolStats";
  import ToolStatsSection from "./ToolStatsSection.svelte";
  import ToolFilters from "./ToolFilters.svelte";
  import ToolEventList from "./ToolEventList.svelte";

  // Filter state
  let filterType = $state<FilterType>("all");
  let selectedTool = $state<string | "all">("all");
  let searchQuery = $state("");

  // Scroll state
  let eventsContainer: HTMLDivElement | null = $state(null);
  let scrollTop = $state(0);

  // Virtualization settings
  const ITEM_HEIGHT = 100;

  // Calculate statistics from tool events
  const toolStats = $derived(calculateToolStats($selectedAgentTools));

  // Get unique tool names
  const uniqueTools = $derived(getUniqueToolNames($selectedAgentTools));

  // Filter tool events
  const filteredTools = $derived(
    filterToolEvents($selectedAgentTools, filterType, selectedTool, searchQuery)
  );

  // Calculate total height for auto-scroll
  const totalHeight = $derived(filteredTools.length * ITEM_HEIGHT);

  function handleScroll(e: Event) {
    const target = e.target as HTMLElement;
    scrollTop = target.scrollTop;
  }

  function handleContainerBind(el: HTMLDivElement | null) {
    eventsContainer = el;
  }

  // Track previous tools length to only scroll on actual new events
  const scrollState = { prevLength: 0 };

  // Auto-scroll to latest tool events when new events arrive
  $effect(() => {
    const currentLength = $selectedAgentTools.length;
    const container = eventsContainer;

    // Skip scroll operations during resize to prevent layout thrashing
    if ($isResizing) return;

    // Only scroll if we have new events and no filters active
    if (
      container &&
      currentLength > scrollState.prevLength &&
      currentLength > 0 &&
      filterType === "all" &&
      selectedTool === "all" &&
      !searchQuery.trim()
    ) {
      scrollState.prevLength = currentLength;

      const { scrollTop: currentScrollTop, clientHeight } = container;
      const distanceToBottom = totalHeight - currentScrollTop - clientHeight;

      // Auto-scroll if we are within 500px of the bottom (or at the top)
      if (distanceToBottom < 500 || currentScrollTop === 0) {
        requestAnimationFrame(() => {
          if (container) {
            container.scrollTop = totalHeight;
          }
        });
      }
    } else if (currentLength < scrollState.prevLength) {
      // Reset if tools were cleared
      scrollState.prevLength = currentLength;
    }
  });
</script>

<aside class="tool-activity">
  <header>
    <h3>Tool Activity</h3>
    <span class="count">{toolStats.totalCalls}</span>
  </header>

  {#if toolStats.totalCalls > 0}
    <ToolStatsSection stats={toolStats} />
  {/if}

  <ToolFilters
    bind:filterType
    bind:selectedTool
    bind:searchQuery
    {uniqueTools}
  />

  <ToolEventList
    {filteredTools}
    allToolsCount={$selectedAgentTools.length}
    onContainerBind={handleContainerBind}
    onScroll={handleScroll}
    {scrollTop}
  />
</aside>

<style>
  .tool-activity {
    height: 100%;
    background-color: var(--bg-secondary);
    border-left: 1px solid var(--border-hex);
    display: flex;
    flex-direction: column;
  }

  header {
    padding: var(--space-4) var(--space-5);
    border-bottom: 1px solid var(--border-hex);
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: var(--bg-secondary);
    flex-shrink: 0;
  }

  h3 {
    font-size: var(--text-base);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
    margin: 0;
  }

  .count {
    background-color: var(--bg-tertiary);
    padding: 2px 10px;
    border-radius: var(--radius-full);
    font-size: var(--text-xs);
    font-weight: var(--font-medium);
    color: var(--text-secondary);
    font-variant-numeric: tabular-nums;
  }
</style>
