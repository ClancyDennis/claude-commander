<script lang="ts">
  import { VirtualScroll } from "svelte-virtual-scroll-list";
  import ChatMessage from "../ChatMessage.svelte";
  import ThinkingIndicator from "./ThinkingIndicator.svelte";
  import SleepIndicator from "./SleepIndicator.svelte";
  import type { ChatMessage as ChatMessageType } from "../../types";
  import { isResizing } from "$lib/stores/resize";

  interface Props {
    messages: ChatMessageType[];
    isThinking: boolean;
    isSleeping?: boolean;
    sleepDuration?: number;
    sleepReason?: string;
  }

  let { messages, isThinking, isSleeping = false, sleepDuration, sleepReason }: Props = $props();

  // DEBUG: Set to true to bypass VirtualScroll and test if it's the performance issue
  const DEBUG_SKIP_VIRTUAL_SCROLL = false;

  let virtualScroll: VirtualScroll | null = $state(null);
  let scrollContainer: HTMLDivElement | null = $state(null);

  // Track previous message count to only scroll on new messages
  const scrollState = { prevLength: 0, timeoutId: null as ReturnType<typeof setTimeout> | null };

  $effect(() => {
    const currentLength = messages.length;
    const vs = virtualScroll;

    // Skip scroll operations during resize to prevent layout thrashing
    if ($isResizing) return;

    // Only auto-scroll when new messages arrive (not on every re-render)
    if (currentLength > scrollState.prevLength) {
      scrollState.prevLength = currentLength;

      // Clear any pending scroll
      if (scrollState.timeoutId) clearTimeout(scrollState.timeoutId);

      // Wait slightly for render
      scrollState.timeoutId = setTimeout(() => {
        requestAnimationFrame(() => {
          if (DEBUG_SKIP_VIRTUAL_SCROLL && scrollContainer) {
            scrollContainer.scrollTop = scrollContainer.scrollHeight;
          } else if (vs) {
            vs.scrollToIndex(currentLength - 1);
          }
        });
      }, 50);
    } else if (currentLength < scrollState.prevLength) {
      // Reset if chat was cleared
      scrollState.prevLength = currentLength;
    }
  });
</script>

<div class="virtual-scroll-container">
  {#if DEBUG_SKIP_VIRTUAL_SCROLL}
    <!-- Debug mode: Regular scroll without virtualization -->
    <div class="regular-scroll" bind:this={scrollContainer}>
      {#each messages as message (message.timestamp)}
        <ChatMessage data={message} />
      {/each}
    </div>
  {:else}
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
  {/if}

  {#if isThinking}
    {#if isSleeping}
      <SleepIndicator duration={sleepDuration} reason={sleepReason} />
    {:else}
      <ThinkingIndicator />
    {/if}
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

  /* Debug mode regular scroll */
  .regular-scroll {
    height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
  }
</style>
