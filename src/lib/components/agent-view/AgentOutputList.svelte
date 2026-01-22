<script lang="ts">
  import type { AgentOutput } from "$lib/types";
  import TypingIndicator from "../TypingIndicator.svelte";
  import { VirtualScroll } from "svelte-virtual-scroll-list";
  import AgentOutputItem from "./AgentOutputItem.svelte";
  import { isResizing } from "$lib/stores/resize";

  let {
    outputs,
    hasAnyOutput,
    isProcessing,
    onClearFilter
  }: {
    outputs: AgentOutput[];
    hasAnyOutput: boolean;
    isProcessing: boolean;
    onClearFilter: () => void;
  } = $props();

  let virtualScroll: VirtualScroll | null = $state(null);
  let isUserScrolling = $state(false);

  // Using a plain object to avoid reactive tracking issues in Svelte 5
  const scrollState = {
    timeoutId: null as ReturnType<typeof setTimeout> | null,
    lastLength: 0
  };

  // Smart auto-scroll for new content - only trigger when outputs actually change
  $effect(() => {
    const currentLength = outputs.length;
    const vs = virtualScroll; // capture current value
    const userScrolling = isUserScrolling; // capture current value
    // Skip scroll operations during resize to prevent layout thrashing
    if ($isResizing) return;

    if (vs && currentLength > 0 && !userScrolling && currentLength !== scrollState.lastLength) {
      scrollState.lastLength = currentLength;

      // Clear any pending scroll
      if (scrollState.timeoutId) {
        clearTimeout(scrollState.timeoutId);
      }

      // Use requestAnimationFrame for smoother scrolling that respects resize
      scrollState.timeoutId = setTimeout(() => {
        requestAnimationFrame(() => {
          vs?.scrollToIndex(currentLength - 1);
        });
      }, 50);
    }
  });

  function handleScroll(e: Event) {
    const target = e.target as HTMLElement;
    const { scrollTop, scrollHeight, clientHeight } = target;
    const distanceToBottom = scrollHeight - scrollTop - clientHeight;
    
    // If user pulls up more than 100px, we consider them "scrolling up"
    isUserScrolling = distanceToBottom > 100;
  }
</script>

<div class="output-container">
  {#if hasAnyOutput && outputs.length === 0}
    <div class="empty-search-results">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <circle cx="11" cy="11" r="8"/>
        <line x1="21" y1="21" x2="16.65" y2="16.65"/>
      </svg>
      <p>No matching results found</p>
      <button class="secondary small" onclick={onClearFilter}>Clear filters</button>
    </div>
  {:else if !hasAnyOutput}
    <div class="empty-output">
      <div class="empty-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
        </svg>
      </div>
      <p class="empty-title">Ready for input</p>
      <p class="empty-hint">Send a prompt to start the conversation</p>
    </div>
  {:else}
    <div class="virtual-scroll-wrapper">
      <VirtualScroll
        bind:this={virtualScroll}
        data={outputs}
        key="timestamp"
        estimateSize={100}
        style="height: 100%; overflow-y: auto;"
        onscroll={handleScroll}
        let:data
        let:index
      >
        <AgentOutputItem {data} {index} />
      </VirtualScroll>

      {#if isProcessing}
        <div class="typing-container">
          <TypingIndicator />
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .output-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    position: relative;
    background-color: var(--bg-primary);
  }

  .virtual-scroll-wrapper {
    flex: 1;
    min-height: 0; /* Critical for flex children to shrink properly */
    overflow: hidden;
    padding: var(--space-lg);
    display: flex;
    flex-direction: column;
  }

  .typing-container {
    padding-top: var(--space-md);
    padding-bottom: var(--space-md);
  }

  .empty-output,
  .empty-search-results {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-xl);
    text-align: center;
  }

  .empty-search-results svg {
    width: 64px;
    height: 64px;
    color: var(--text-muted);
    opacity: 0.5;
    margin-bottom: var(--space-md);
  }
  
  .empty-search-results p {
    color: var(--text-secondary);
    font-size: 16px;
    margin-bottom: var(--space-md);
  }

  .empty-output .empty-icon {
    width: 80px;
    height: 80px;
    border-radius: 24px;
    background: linear-gradient(135deg, var(--bg-secondary) 0%, var(--bg-tertiary) 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: var(--space-lg);
    border: 1px solid var(--border);
  }

  .empty-output .empty-icon svg {
    width: 40px;
    height: 40px;
    color: var(--text-muted);
  }

  .empty-output .empty-title {
    font-size: 18px;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: var(--space-sm);
  }

  .empty-output .empty-hint {
    font-size: 14px;
    color: var(--text-muted);
  }
  
  /* Ensure virtual scroll inner container has padding/space if needed, 
     though wrapper padding handles most of it. */
</style>
