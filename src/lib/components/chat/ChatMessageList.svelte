<script lang="ts">
  import { VirtualScroll } from "svelte-virtual-scroll-list";
  import ChatMessage from "../ChatMessage.svelte";
  import ThinkingIndicator from "./ThinkingIndicator.svelte";
  import type { ChatMessage as ChatMessageType } from "../../types";
  import { isResizing } from "$lib/stores/resize";

  interface Props {
    messages: ChatMessageType[];
    isThinking: boolean;
  }

  let { messages, isThinking }: Props = $props();

  let virtualScroll: VirtualScroll | null = $state(null);

  // Track previous message count to only scroll on new messages
  const scrollState = { prevLength: 0, timeoutId: null as ReturnType<typeof setTimeout> | null };

  $effect(() => {
    const currentLength = messages.length;
    const vs = virtualScroll;

    // Skip scroll operations during resize to prevent layout thrashing
    if ($isResizing) return;

    // Only auto-scroll when new messages arrive (not on every re-render)
    if (currentLength > scrollState.prevLength && vs) {
      scrollState.prevLength = currentLength;

      // Clear any pending scroll
      if (scrollState.timeoutId) clearTimeout(scrollState.timeoutId);

      // Wait slightly for render
      scrollState.timeoutId = setTimeout(() => {
        requestAnimationFrame(() => {
          vs?.scrollToIndex(currentLength - 1);
        });
      }, 50);
    } else if (currentLength < scrollState.prevLength) {
      // Reset if chat was cleared
      scrollState.prevLength = currentLength;
    }
  });
</script>

<div class="virtual-scroll-container">
  <!-- @ts-ignore - style prop is valid but not in type defs -->
  <VirtualScroll
    bind:this={virtualScroll}
    data={messages}
    key="timestamp"
    estimateSize={80}
    let:data
  >
    <ChatMessage {data} />
  </VirtualScroll>

  {#if isThinking}
    <ThinkingIndicator />
  {/if}
</div>

<style>
  .virtual-scroll-container {
    flex: 1;
    min-height: 0; /* Critical for flex children to shrink properly during resize */
    overflow: hidden;
    padding: var(--space-5);
    display: flex;
    flex-direction: column;
  }

  /* Style the VirtualScroll container */
  .virtual-scroll-container :global(.virtual-scroll-list) {
    height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
  }
</style>
