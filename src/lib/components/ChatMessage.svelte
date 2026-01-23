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
    margin-bottom: 16px;
    animation: fadeIn 0.3s ease;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .message-wrapper.user {
    justify-content: flex-end;
  }

  .message-wrapper.assistant {
    justify-content: flex-start;
  }

  .message {
    max-width: 85%;
    min-width: 200px;
    border-radius: 12px;
    padding: 12px 16px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
  }

  .user .message {
    background: linear-gradient(135deg, #7c3aed 0%, #6d28d9 100%);
    color: white;
  }

  .assistant .message {
    background: #1a1a1f;
    border: 1px solid rgba(124, 58, 237, 0.2);
    color: #e0e0e0;
  }

  .message-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
    font-size: 12px;
    opacity: 0.9;
  }

  .role-icon {
    font-size: 14px;
  }

  .role-name {
    font-weight: 600;
    flex: 1;
  }

  .timestamp {
    font-size: 11px;
    opacity: 0.7;
  }

  .message-image-container {
    margin-bottom: 10px;
    border-radius: 8px;
    overflow: hidden;
  }

  .message-image {
    max-width: 100%;
    max-height: 300px;
    width: auto;
    height: auto;
    display: block;
    border-radius: 8px;
    cursor: pointer;
    transition: transform 0.2s ease;
  }

  .message-image:hover {
    transform: scale(1.02);
  }

  .message-content {
    line-height: 1.6;
    font-size: 14px;
    /* MarkdownRenderer handles wrapping */
  }

  .tool-calls {
    margin-top: 12px;
    padding-top: 12px;
    border-top: 1px solid rgba(124, 58, 237, 0.2);
  }

  .tool-calls-header {
    font-size: 11px;
    font-weight: 600;
    color: #999;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 8px;
  }
</style>
