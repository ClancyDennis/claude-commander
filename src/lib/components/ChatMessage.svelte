<script lang="ts">
  import type { ChatMessage } from "../types";
  import ToolCallDisplay from "./ToolCallDisplay.svelte";
  import { formatTimeOfDay } from '$lib/utils/formatting';
  import MarkdownRenderer from "./MarkdownRenderer.svelte";

  interface Props {
    data: ChatMessage; // VirtualScroll passes 'data'
  }

  let { data: message }: Props = $props();

  const isUser = $derived(message.role === "user");
</script>

<div class="message-wrapper" class:user={isUser} class:assistant={!isUser}>
  <div class="message">
    <div class="message-header">
      <span class="role-icon">{isUser ? "👤" : "🤖"}</span>
      <span class="role-name">{isUser ? "You" : "Assistant"}</span>
      <span class="timestamp">{formatTimeOfDay(message.timestamp)}</span>
    </div>

    {#if message.image}
      <div class="message-image-container">
        <img
          src={message.image.previewUrl || `data:${message.image.mimeType};base64,${message.image.base64Data}`}
          alt={message.image.filename}
          class="message-image"
        />
      </div>
    {/if}

    <div class="message-content">
      <MarkdownRenderer content={message.content} />
    </div>

    {#if message.toolCalls && message.toolCalls.length > 0}
      <div class="tool-calls">
        <div class="tool-calls-header">Used Tools:</div>
        {#each message.toolCalls as toolCall (toolCall.id)}
          <ToolCallDisplay {toolCall} />
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .message-wrapper {
    display: flex;
    margin-bottom: var(--space-4);
  }

  .message-wrapper.user {
    justify-content: flex-end;
  }

  .message-wrapper.assistant {
    justify-content: flex-start;
  }

  .message {
    max-width: 85%;
    min-width: 180px;
    border-radius: var(--radius-lg);
    padding: var(--space-3) var(--space-4);
    box-shadow: var(--shadow-sm);
  }

  .user .message {
    background: var(--accent-hex);
    color: white;
  }

  .assistant .message {
    background: var(--bg-tertiary);
    border: 1px solid var(--border-hex);
    color: var(--text-primary);
  }

  .message-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-2);
    font-size: var(--text-xs);
  }

  .user .message-header {
    opacity: 0.9;
  }

  .assistant .message-header {
    color: var(--text-secondary);
  }

  .role-icon {
    font-size: var(--text-sm);
  }

  .role-name {
    font-weight: var(--font-semibold);
    flex: 1;
  }

  .timestamp {
    font-size: var(--text-xs);
    opacity: 0.7;
  }

  .message-image-container {
    margin-bottom: var(--space-3);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .message-image {
    max-width: 100%;
    max-height: 300px;
    width: auto;
    height: auto;
    display: block;
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: transform var(--transition-fast);
  }

  .message-image:hover {
    transform: scale(1.02);
  }

  .message-content {
    line-height: var(--leading-relaxed);
    font-size: var(--text-sm);
    overflow-wrap: break-word;
    word-break: break-word;
  }

  .tool-calls {
    margin-top: var(--space-3);
    padding-top: var(--space-3);
    border-top: 1px solid rgba(255, 255, 255, 0.1);
  }

  .user .tool-calls {
    border-top-color: rgba(255, 255, 255, 0.2);
  }

  .tool-calls-header {
    font-size: var(--text-xs);
    font-weight: var(--font-semibold);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: var(--space-2);
  }

  .user .tool-calls-header {
    color: rgba(255, 255, 255, 0.7);
  }
</style>
