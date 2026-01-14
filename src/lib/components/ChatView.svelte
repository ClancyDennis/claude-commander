<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { metaAgentChat, metaAgentThinking, addChatMessage } from "../stores/agents";
  import ChatMessage from "./ChatMessage.svelte";
  import type { ChatResponse } from "../types";

  let input = $state("");
  let messagesContainer: HTMLDivElement | undefined = $state();
  let error = $state<string | null>(null);

  async function sendMessage() {
    if (!input.trim() || $metaAgentThinking) return;

    const userMessage = input.trim();
    input = "";
    error = null;

    // Add user message to chat
    addChatMessage({
      role: "user",
      content: userMessage,
      timestamp: Date.now(),
    });

    // Scroll to bottom
    setTimeout(scrollToBottom, 100);

    try {
      const response = await invoke<ChatResponse>("send_chat_message", {
        message: userMessage,
      });

      // Add assistant response
      addChatMessage(response.message);

      // Scroll to bottom after response
      setTimeout(scrollToBottom, 100);
    } catch (e) {
      console.error("Chat error:", e);
      error = e instanceof Error ? e.message : String(e);

      // Add error message to chat
      addChatMessage({
        role: "assistant",
        content: `Error: ${error}`,
        timestamp: Date.now(),
      });
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      sendMessage();
    }
  }

  function scrollToBottom() {
    if (messagesContainer) {
      messagesContainer.scrollTop = messagesContainer.scrollHeight;
    }
  }

  function handleClear() {
    if (confirm("Clear all chat history?")) {
      invoke("clear_chat_history");
      metaAgentChat.set([]);
    }
  }

  $effect(() => {
    // Auto-scroll when new messages arrive
    if ($metaAgentChat.length > 0) {
      setTimeout(scrollToBottom, 100);
    }
  });
</script>

<div class="chat-view">
  <div class="chat-header">
    <div class="header-left">
      <span class="header-icon">💬</span>
      <span class="header-title">Chat Assistant</span>
      {#if $metaAgentThinking}
        <span class="thinking-indicator">Thinking...</span>
      {/if}
    </div>
    <div class="header-actions">
      <button onclick={handleClear} class="action-btn" title="Clear chat">
        Clear
      </button>
    </div>
  </div>

  <div class="messages-container" bind:this={messagesContainer}>
    {#if $metaAgentChat.length === 0}
      <div class="empty-state">
        <div class="empty-icon">🤖</div>
        <div class="empty-title">AI-Native Control</div>
        <div class="empty-description">
          Ask me to manage agents, send prompts, or control the application.
        </div>
        <div class="example-prompts">
          <div class="example-title">Try asking:</div>
          <div class="example">"List all running agents"</div>
          <div class="example">"Create a new agent in /tmp/test"</div>
          <div class="example">"Send a prompt to the most recent agent"</div>
        </div>
      </div>
    {:else}
      {#each $metaAgentChat as message (message.timestamp)}
        <ChatMessage {message} />
      {/each}
    {/if}

    {#if $metaAgentThinking}
      <div class="thinking-message">
        <div class="thinking-dots">
          <span></span>
          <span></span>
          <span></span>
        </div>
      </div>
    {/if}
  </div>

  <div class="input-area">
    {#if error}
      <div class="error-banner">
        ⚠️ {error}
      </div>
    {/if}
    <div class="input-wrapper">
      <textarea
        bind:value={input}
        onkeydown={handleKeydown}
        placeholder="Type a message... (Enter to send, Shift+Enter for newline)"
        disabled={$metaAgentThinking}
        rows="3"
      ></textarea>
      <button
        onclick={sendMessage}
        disabled={!input.trim() || $metaAgentThinking}
        class="send-btn"
      >
        Send
      </button>
    </div>
  </div>
</div>

<style>
  .chat-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #0f0f13;
  }

  .chat-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    background: #1a1a1f;
    border-bottom: 1px solid rgba(124, 58, 237, 0.2);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .header-icon {
    font-size: 24px;
  }

  .header-title {
    font-size: 18px;
    font-weight: 600;
    color: #e0e0e0;
  }

  .thinking-indicator {
    font-size: 12px;
    color: #7c3aed;
    font-weight: 500;
    animation: pulse 1.5s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }

  .header-actions {
    display: flex;
    gap: 8px;
  }

  .action-btn {
    padding: 8px 16px;
    background: transparent;
    border: 1px solid rgba(124, 58, 237, 0.3);
    border-radius: 6px;
    color: #7c3aed;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .action-btn:hover {
    background: rgba(124, 58, 237, 0.1);
    border-color: rgba(124, 58, 237, 0.5);
  }

  .messages-container {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
    display: flex;
    flex-direction: column;
  }

  .messages-container::-webkit-scrollbar {
    width: 8px;
  }

  .messages-container::-webkit-scrollbar-track {
    background: rgba(0, 0, 0, 0.2);
  }

  .messages-container::-webkit-scrollbar-thumb {
    background: rgba(124, 58, 237, 0.4);
    border-radius: 4px;
  }

  .messages-container::-webkit-scrollbar-thumb:hover {
    background: rgba(124, 58, 237, 0.6);
  }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: 40px;
    color: #999;
  }

  .empty-icon {
    font-size: 64px;
    margin-bottom: 16px;
    opacity: 0.6;
  }

  .empty-title {
    font-size: 24px;
    font-weight: 600;
    color: #e0e0e0;
    margin-bottom: 8px;
  }

  .empty-description {
    font-size: 14px;
    margin-bottom: 24px;
    max-width: 400px;
  }

  .example-prompts {
    background: rgba(124, 58, 237, 0.05);
    border: 1px solid rgba(124, 58, 237, 0.2);
    border-radius: 12px;
    padding: 16px;
    max-width: 400px;
  }

  .example-title {
    font-size: 12px;
    font-weight: 600;
    color: #7c3aed;
    margin-bottom: 12px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .example {
    font-size: 13px;
    padding: 8px 12px;
    background: rgba(124, 58, 237, 0.1);
    border-radius: 6px;
    margin-bottom: 6px;
    color: #c0c0c0;
    font-style: italic;
  }

  .example:last-child {
    margin-bottom: 0;
  }

  .thinking-message {
    display: flex;
    justify-content: flex-start;
    margin-bottom: 16px;
  }

  .thinking-dots {
    background: #1a1a1f;
    border: 1px solid rgba(124, 58, 237, 0.2);
    border-radius: 12px;
    padding: 12px 20px;
    display: flex;
    gap: 6px;
  }

  .thinking-dots span {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #7c3aed;
    animation: bounce 1.4s infinite ease-in-out;
  }

  .thinking-dots span:nth-child(1) {
    animation-delay: -0.32s;
  }

  .thinking-dots span:nth-child(2) {
    animation-delay: -0.16s;
  }

  @keyframes bounce {
    0%, 80%, 100% {
      transform: scale(0);
      opacity: 0.5;
    }
    40% {
      transform: scale(1);
      opacity: 1;
    }
  }

  .input-area {
    padding: 16px 20px;
    background: #1a1a1f;
    border-top: 1px solid rgba(124, 58, 237, 0.2);
  }

  .error-banner {
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    color: #ef4444;
    padding: 10px 14px;
    border-radius: 6px;
    font-size: 13px;
    margin-bottom: 12px;
  }

  .input-wrapper {
    display: flex;
    gap: 12px;
    align-items: flex-end;
  }

  textarea {
    flex: 1;
    background: #0f0f13;
    border: 1px solid rgba(124, 58, 237, 0.3);
    border-radius: 8px;
    padding: 12px 14px;
    color: #e0e0e0;
    font-size: 14px;
    font-family: inherit;
    resize: none;
    transition: border-color 0.2s ease;
  }

  textarea:focus {
    outline: none;
    border-color: #7c3aed;
  }

  textarea:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  textarea::placeholder {
    color: #666;
  }

  .send-btn {
    padding: 12px 24px;
    background: linear-gradient(135deg, #7c3aed 0%, #6d28d9 100%);
    border: none;
    border-radius: 8px;
    color: white;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
    min-width: 80px;
  }

  .send-btn:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(124, 58, 237, 0.4);
  }

  .send-btn:active:not(:disabled) {
    transform: translateY(0);
  }

  .send-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
