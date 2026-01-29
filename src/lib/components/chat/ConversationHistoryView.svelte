<script lang="ts">
  import type { ChatMessage } from "$lib/types";
  import { formatMessageContent, type FormattedContent } from "$lib/utils/toolFormatter";
  import { formatTimeOfDay } from "$lib/utils/formatting";
  import { ArrowLeft, Play, Wrench, CheckCircle, XCircle } from "lucide-svelte";
  import MarkdownRenderer from "../MarkdownRenderer.svelte";

  interface Props {
    messages: ChatMessage[];
    title?: string;
    onClose: () => void;
    onResume: () => void;
  }

  let { messages, title = "Conversation History", onClose, onResume }: Props = $props();

  // Process messages to format tool content
  const processedMessages = $derived(
    messages.map((msg) => ({
      ...msg,
      formattedContent: formatMessageContent(msg.content),
    }))
  );
</script>

<div class="history-view">
  <header class="history-header">
    <button class="back-btn" onclick={onClose} title="Close">
      <ArrowLeft size={20} />
    </button>
    <div class="header-info">
      <span class="header-icon">📜</span>
      <div class="header-text">
        <h2 class="header-title">{title}</h2>
        <span class="message-count">{messages.length} messages</span>
      </div>
    </div>
    <button class="resume-btn" onclick={onResume}>
      <Play size={16} />
      <span>Resume this chat</span>
    </button>
  </header>

  <div class="messages-container">
    {#each processedMessages as message (message.timestamp)}
      {@const isUser = message.role === "user"}
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
            {#each message.formattedContent as content}
              {#if content.type === "text"}
                <MarkdownRenderer content={content.formatted} />
              {:else if content.type === "tool_use"}
                <div class="tool-block tool-use">
                  <div class="tool-header">
                    <Wrench size={14} />
                    <span class="tool-name">{content.toolName}</span>
                  </div>
                  {#if content.formatted}
                    <div class="tool-detail">{content.formatted}</div>
                  {/if}
                </div>
              {:else if content.type === "tool_result"}
                <div class="tool-block tool-result" class:error={content.isError}>
                  <div class="tool-header">
                    {#if content.isError}
                      <XCircle size={14} />
                      <span class="tool-status">Error</span>
                    {:else}
                      <CheckCircle size={14} />
                      <span class="tool-status">Result</span>
                    {/if}
                  </div>
                  {#if content.formatted}
                    <div class="tool-detail">{content.formatted}</div>
                  {/if}
                </div>
              {/if}
            {/each}
          </div>
        </div>
      </div>
    {/each}
  </div>
</div>

<style>
  .history-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
  }

  .history-header {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-4);
    border-bottom: 1px solid var(--border-hex);
    background: linear-gradient(180deg, var(--bg-secondary) 0%, var(--bg-primary) 100%);
    flex-shrink: 0;
  }

  .back-btn {
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .back-btn:hover {
    background: var(--bg-elevated);
    color: var(--text-primary);
    border-color: var(--accent-hex);
  }

  .header-info {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    flex: 1;
    min-width: 0;
  }

  .header-icon {
    font-size: 24px;
  }

  .header-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .header-title {
    font-size: var(--text-lg);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
    margin: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .message-count {
    font-size: var(--text-xs);
    color: var(--text-muted);
  }

  .resume-btn {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-4);
    background: var(--accent-hex);
    color: white;
    border: none;
    border-radius: var(--radius-md);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    cursor: pointer;
    transition: all var(--transition-fast);
    flex-shrink: 0;
  }

  .resume-btn:hover {
    background: var(--accent-hover);
    transform: translateY(-1px);
  }

  .messages-container {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-5);
  }

  /* Message styles (similar to ChatMessage.svelte) */
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
  }

  .message-content {
    line-height: var(--leading-relaxed);
    font-size: var(--text-sm);
    overflow-wrap: break-word;
    word-break: break-word;
  }

  /* Tool block styles */
  .tool-block {
    margin: var(--space-3) 0;
    padding: var(--space-3);
    border-radius: var(--radius-md);
    font-size: var(--text-sm);
  }

  .tool-use {
    background: rgba(99, 102, 241, 0.15);
    border: 1px solid rgba(99, 102, 241, 0.3);
  }

  .tool-result {
    background: rgba(34, 197, 94, 0.15);
    border: 1px solid rgba(34, 197, 94, 0.3);
  }

  .tool-result.error {
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid rgba(239, 68, 68, 0.3);
  }

  .tool-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-weight: var(--font-semibold);
    margin-bottom: var(--space-2);
  }

  .tool-use .tool-header {
    color: rgb(129, 140, 248);
  }

  .tool-result .tool-header {
    color: rgb(74, 222, 128);
  }

  .tool-result.error .tool-header {
    color: rgb(248, 113, 113);
  }

  .tool-name {
    font-family: var(--font-mono);
  }

  .tool-status {
    text-transform: uppercase;
    font-size: var(--text-xs);
    letter-spacing: 0.5px;
  }

  .tool-detail {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    color: var(--text-secondary);
    white-space: pre-wrap;
    line-height: 1.5;
  }

  .user .tool-block {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.2);
  }

  .user .tool-header {
    color: rgba(255, 255, 255, 0.9);
  }

  .user .tool-detail {
    color: rgba(255, 255, 255, 0.8);
  }
</style>
