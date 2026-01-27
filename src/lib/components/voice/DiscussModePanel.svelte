<script lang="ts">
  import { tick } from "svelte";
  import ConversationMessage from "./ConversationMessage.svelte";
  import type { DiscussMessage } from "$lib/stores/voice";

  interface Props {
    messages: DiscussMessage[];
    currentUserTranscript: string;
    currentAssistantResponse: string;
    isActive: boolean;
    isConnecting: boolean;
    error: string | null;
    onMessageClick: (msg: DiscussMessage) => void;
  }

  let {
    messages,
    currentUserTranscript,
    currentAssistantResponse,
    isActive,
    isConnecting,
    error,
    onMessageClick,
  }: Props = $props();

  let conversationEl: HTMLDivElement | null = $state(null);

  const hasMessages = $derived(
    messages.length > 0 ||
      currentUserTranscript.length > 0 ||
      currentAssistantResponse.length > 0
  );

  // Auto-scroll to bottom when new messages arrive
  $effect(() => {
    if (messages.length > 0 || currentUserTranscript || currentAssistantResponse) {
      tick().then(() => {
        if (conversationEl) {
          conversationEl.scrollTop = conversationEl.scrollHeight;
        }
      });
    }
  });
</script>

<div class="discuss-content" bind:this={conversationEl}>
  {#if !hasMessages && !isActive && !isConnecting}
    <div class="empty-state">
      <p>Click Start to begin a voice discussion.</p>
      <p class="hint">You can ask the AI to talk to Mission Control for help.</p>
    </div>
  {:else if messages.length > 0 && !isActive && !isConnecting}
    <p class="selection-hint">Tap messages to select, then send to chat</p>
  {/if}

  {#if hasMessages}
    <div class="conversation">
      {#each messages as msg (msg.id)}
        <ConversationMessage
          message={msg}
          onclick={() => onMessageClick(msg)}
        />
      {/each}

      {#if currentUserTranscript}
        <ConversationMessage
          message={{ id: "current-user", role: "user", content: currentUserTranscript, timestamp: Date.now(), selected: false }}
          isCurrent={true}
        />
      {:else if isActive && messages.length === 0}
        <p class="listening-text">Listening...</p>
      {/if}

      {#if currentAssistantResponse}
        <ConversationMessage
          message={{ id: "current-assistant", role: "assistant", content: currentAssistantResponse, timestamp: Date.now(), selected: false }}
          isCurrent={true}
        />
      {/if}
    </div>
  {/if}

  {#if error}
    <div class="error-message">{error}</div>
  {/if}
</div>

<style>
  .discuss-content {
    min-height: 200px;
    padding: 0 0.25rem;
    overflow-y: auto;
  }

  .empty-state {
    text-align: center;
    padding: 2rem 1rem;
    color: var(--text-secondary);
  }

  .empty-state p {
    margin: 0 0 0.5rem;
    font-size: 0.875rem;
  }

  .empty-state .hint {
    font-size: 0.75rem;
    color: var(--text-tertiary);
  }

  .conversation {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .listening-text {
    font-size: 0.875rem;
    color: var(--text-tertiary);
    font-style: italic;
    text-align: center;
    margin: 0;
  }

  .selection-hint {
    font-size: 0.75rem;
    color: var(--text-tertiary);
    text-align: center;
    margin: 0 0 0.5rem 0;
    padding: 0.25rem 0.5rem;
    background: var(--bg-secondary);
    border-radius: 0.25rem;
  }

  .error-message {
    margin-top: 1rem;
    padding: 0.5rem 0.75rem;
    background: var(--error);
    color: white;
    font-size: 0.75rem;
    border-radius: 0.375rem;
  }
</style>
