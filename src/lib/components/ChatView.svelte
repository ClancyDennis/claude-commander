<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { metaAgentChat, metaAgentThinking, addChatMessage, agentsWithOutputs } from "../stores/agents";
  import ChatMessage from "./ChatMessage.svelte";
  import type { ChatResponse } from "../types";
  import HelpTip from "./new-agent/HelpTip.svelte";
  import { VirtualScroll } from "svelte-virtual-scroll-list";
  import { isResizing } from "$lib/stores/resize";

  let input = $state("");
  let textarea: HTMLTextAreaElement | null = $state(null);
  let virtualScroll: VirtualScroll | null = $state(null);
  let error = $state<string | null>(null);
  let processingAgentId = $state<string | null>(null);

  function adjustInputHeight() {
    if (textarea) {
      textarea.style.height = "auto";
      textarea.style.height = Math.min(textarea.scrollHeight, 200) + "px";
    }
  }

  async function sendMessage() {
    if (!input.trim() || $metaAgentThinking) return;

    const userMessage = input.trim();
    input = "";
    if (textarea) textarea.style.height = "auto"; // Reset height
    error = null;

    // Add user message to chat
    addChatMessage({
      role: "user",
      content: userMessage,
      timestamp: Date.now(),
    });

    try {
      const response = await invoke<ChatResponse>("send_chat_message", {
        message: userMessage,
      });

      // Add assistant response
      addChatMessage(response.message);

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

  function handleClear() {
    if (confirm("Clear all chat history?")) {
      invoke("clear_chat_history");
      metaAgentChat.set([]);
    }
  }

  async function handleProcessResults(agentId: string) {
    if (processingAgentId || $metaAgentThinking) return;

    processingAgentId = agentId;
    error = null;

    try {
      const response = await invoke<ChatResponse>("process_agent_results", {
        agentId,
      });

      // Add the response to chat
      addChatMessage(response.message);

    } catch (e) {
      console.error("Error processing agent results:", e);
      error = e instanceof Error ? e.message : String(e);
    } finally {
      processingAgentId = null;
    }
  }

  // Track previous message count to only scroll on new messages
  // Using a closure to avoid reactive tracking issues in Svelte 5
  const scrollState = { prevLength: 0, timeoutId: null as ReturnType<typeof setTimeout> | null };

  $effect(() => {
    const currentLength = $metaAgentChat.length;
    const vs = virtualScroll; // capture current value
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

  $effect(() => {
    // Skip during resize to prevent layout thrashing
    if ($isResizing) return;
    if (input !== undefined) adjustInputHeight();
  });
</script>

<div class="chat-view">
  <div class="chat-header">
    <div class="header-left">
      <span class="header-icon">🎛️</span>
      <span class="header-title">System Commander <HelpTip text="Natural language control center. Create agents, send prompts, and manage your fleet." placement="bottom" /></span>
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

  <div class="messages-wrapper">
    {#if $metaAgentChat.length === 0}
      <div class="empty-state">
        <div class="empty-icon">🚀</div>
        <div class="empty-title">Mission Control</div>
        <div class="empty-description">
          Command your agents, deploy prompts, take control.
        </div>
        <div class="example-prompts">
          <div class="example-title">Try asking:</div>
          <div class="example">"List all running agents"</div>
          <div class="example">"Create a new agent in /tmp/test"</div>
          <div class="example">"Send a prompt to the most recent agent"</div>
        </div>
      </div>
    {:else}
      <div class="virtual-scroll-container">
        <VirtualScroll
          bind:this={virtualScroll}
          data={$metaAgentChat}
          key="timestamp"
          estimateSize={80}
          style="height: 100%; overflow-y: auto;"
          let:data
        >
          <ChatMessage {data} />
        </VirtualScroll>
        
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
    {/if}
  </div>

  {#if $agentsWithOutputs.length > 0}
    <div class="agent-results-section">
      <div class="section-title">Agents with results: <HelpTip text="Click to have the System Commander analyze and summarize agent outputs." placement="right" /></div>
      <div class="agent-results-buttons">
        {#each $agentsWithOutputs as agent (agent.id)}
          <button
            onclick={() => handleProcessResults(agent.id)}
            disabled={processingAgentId !== null || $metaAgentThinking}
            class="process-results-btn"
            class:processing={processingAgentId === agent.id}
          >
            📊 Process results from {agent.workingDir}
            <span class="output-count">({agent.outputCount} outputs)</span>
          </button>
        {/each}
      </div>
    </div>
  {/if}

  <div class="input-area">
    {#if error}
      <div class="error-banner">
        ⚠️ {error}
      </div>
    {/if}
    <div class="input-wrapper">
      <textarea
        bind:this={textarea}
        bind:value={input}
        onkeydown={handleKeydown}
        oninput={adjustInputHeight}
        placeholder="Type a message... (Enter to send, Shift+Enter for newline)"
        disabled={$metaAgentThinking}
        rows="1"
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
    display: flex;
    align-items: center;
    gap: 8px;
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

  .messages-wrapper {
    flex: 1;
    overflow: hidden;
    position: relative;
    display: flex;
    flex-direction: column;
  }

  .virtual-scroll-container {
    flex: 1;
    min-height: 0; /* Critical for flex children to shrink properly during resize */
    overflow: hidden;
    padding: 20px;
    display: flex;
    flex-direction: column;
  }

  /* Custom scrollbar for virtual container if needed, but VirtualScroll handles its own usually */
  
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
    padding-top: 10px;
    padding-bottom: 10px;
  }

  .thinking-dots {
    background: #1a1a1f;
    border: 1px solid rgba(124, 58, 237, 0.2);
    border-radius: 12px;
    padding: 12px 20px;
    display: flex;
    gap: 6px;
    width: fit-content;
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
    min-height: 44px;
    max-height: 200px;
    line-height: 1.5;
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
    height: 44px; /* Align with single line input */
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

  .agent-results-section {
    padding: 12px 20px;
    background: rgba(124, 58, 237, 0.05);
    border-top: 1px solid rgba(124, 58, 237, 0.2);
    border-bottom: 1px solid rgba(124, 58, 237, 0.2);
  }

  .section-title {
    font-size: 12px;
    font-weight: 600;
    color: #7c3aed;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 8px;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .agent-results-buttons {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .process-results-btn {
    padding: 10px 14px;
    background: linear-gradient(135deg, rgba(124, 58, 237, 0.1) 0%, rgba(109, 40, 217, 0.1) 100%);
    border: 1px solid rgba(124, 58, 237, 0.3);
    border-radius: 8px;
    color: #e0e0e0;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
    text-align: left;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .process-results-btn:hover:not(:disabled) {
    background: linear-gradient(135deg, rgba(124, 58, 237, 0.2) 0%, rgba(109, 40, 217, 0.2) 100%);
    border-color: rgba(124, 58, 237, 0.5);
    transform: translateX(2px);
  }

  .process-results-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .process-results-btn.processing {
    background: linear-gradient(135deg, rgba(124, 58, 237, 0.3) 0%, rgba(109, 40, 217, 0.3) 100%);
    animation: pulse 1.5s ease-in-out infinite;
  }

  .output-count {
    color: #999;
    font-size: 12px;
    margin-left: auto;
  }
</style>
