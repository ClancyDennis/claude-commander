<script lang="ts">
  import type { ChatMessage } from "../types";
  import ToolCallDisplay from "./ToolCallDisplay.svelte";

  interface Props {
    message: ChatMessage;
  }

  let { message }: Props = $props();

  function formatTime(timestamp: number): string {
    const date = new Date(timestamp);
    return date.toLocaleTimeString('en-US', {
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  const isUser = message.role === "user";
</script>

<div class="message-wrapper" class:user={isUser} class:assistant={!isUser}>
  <div class="message">
    <div class="message-header">
      <span class="role-icon">{isUser ? "👤" : "🤖"}</span>
      <span class="role-name">{isUser ? "You" : "Assistant"}</span>
      <span class="timestamp">{formatTime(message.timestamp)}</span>
    </div>

    <div class="message-content">
      {message.content}
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
    max-width: 70%;
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

  .message-content {
    line-height: 1.6;
    font-size: 14px;
    white-space: pre-wrap;
    word-break: break-word;
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
