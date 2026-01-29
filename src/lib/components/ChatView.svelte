<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { metaAgentChat, metaAgentThinking, addChatMessage, agentsWithOutputs, dismissAgentResult } from "../stores/agents";
  import { metaAgentTodos } from "../stores/metaTodos";
  import { isSleeping, metaSleepStatus } from "../stores/metaAgentInteraction";
  import { voiceSidebarOpen } from "../stores/voice";
  import { syncCurrentConversationId, viewingConversation, closeConversationView } from "../stores/metaConversations";
  import type { ChatResponse, ConfigStatus, ImageAttachment } from "../types";
  import { useAsyncData } from '$lib/hooks/useAsyncData.svelte';

  // Import sub-components
  import { PageLayout } from "./ui/layout";
  import ChatHeader from "./chat/ChatHeader.svelte";
  import ChatInput from "./chat/ChatInput.svelte";
  import ChatEmptyState from "./chat/ChatEmptyState.svelte";
  import ChatLockedState from "./chat/ChatLockedState.svelte";
  import ChatMessageList from "./chat/ChatMessageList.svelte";
  import AgentResultsSection from "./chat/AgentResultsSection.svelte";
  import MetaTaskProgress from "./chat/MetaTaskProgress.svelte";
  import ConversationHistoryView from "./chat/ConversationHistoryView.svelte";
  import VoiceSidebar from "./voice/VoiceSidebar.svelte";

  // State
  let processingAgentId = $state<string | null>(null);
  let error = $state<string | null>(null);
  let showTodoPanel = $state(false);

  // Config async data
  const asyncConfig = useAsyncData(() => invoke<ConfigStatus>("get_config_status"));

  // Derived config values
  const configLoaded = $derived(!asyncConfig.loading || asyncConfig.data !== null || asyncConfig.error !== null);
  const hasApiKey = $derived(asyncConfig.data?.api_keys.some((key) => key.is_configured) ?? true); // Default to true to avoid flash
  const hasOpenAiKey = $derived(asyncConfig.data?.api_keys.some(
    (key) => key.provider.toLowerCase() === "openai" && key.is_configured
  ) ?? false);
  const configPath = $derived(asyncConfig.data?.config_path ?? "");

  onMount(() => {
    asyncConfig.fetch();
    // Sync current conversation ID from backend
    syncCurrentConversationId();
  });

  function handleClear() {
    if (confirm("Clear all chat history?")) {
      invoke("clear_chat_history");
      metaAgentChat.set([]);
    }
  }

  async function openConfigDir() {
    try {
      await invoke("open_config_directory");
    } catch (err) {
      console.error("Failed to open config directory:", err);
    }
  }

  async function handleSendMessage(message: string, image: ImageAttachment | null) {
    error = null;

    // Add user message to chat (with image if present)
    addChatMessage({
      role: "user",
      content: message || (image ? "[Image]" : ""),
      image: image || undefined,
      timestamp: Date.now(),
    });

    try {
      const response = await invoke<ChatResponse>("send_chat_message", {
        message,
        image,
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

  function handleVoiceClick() {
    voiceSidebarOpen.set(true);
  }

  function handleSendToChat(text: string) {
    // Close sidebar and send the text to chat
    voiceSidebarOpen.set(false);
    handleSendMessage(text, null);
  }

  async function handleProcessResults(agentId: string, resultsOnly: boolean = false) {
    if (processingAgentId || $metaAgentThinking) return;

    processingAgentId = agentId;
    error = null;

    try {
      const response = await invoke<ChatResponse>("process_agent_results", {
        agentId,
        resultsOnly,
      });

      // Add the response to chat
      addChatMessage(response.message);

      // Dismiss the notification after successfully processing
      dismissAgentResult(agentId);
    } catch (e) {
      console.error("Error processing agent results:", e);
      error = e instanceof Error ? e.message : String(e);
    } finally {
      processingAgentId = null;
    }
  }

  function handleDismissResult(agentId: string) {
    dismissAgentResult(agentId);
  }
</script>

<PageLayout>
  {#if $viewingConversation}
    <!-- Read-only conversation history view -->
    <ConversationHistoryView
      messages={$viewingConversation.messages}
      title={$viewingConversation.title}
      onClose={() => closeConversationView(false)}
      onResume={() => closeConversationView(true)}
    />
  {:else}
    <!-- Active chat view -->
    <ChatHeader
      isThinking={$metaAgentThinking}
      onClear={handleClear}
      {hasOpenAiKey}
      onVoiceClick={handleVoiceClick}
      {showTodoPanel}
      onToggleTodoPanel={() => showTodoPanel = !showTodoPanel}
      hasTodos={$metaAgentTodos.length > 0}
    />

    <div class="content-wrapper">
      <div class="messages-panel">
        <div class="messages-wrapper">
          {#if configLoaded && !hasApiKey}
            <ChatLockedState {configPath} onOpenConfigDir={openConfigDir} />
          {:else if $metaAgentChat.length === 0}
            <ChatEmptyState />
          {:else}
            <ChatMessageList
              messages={$metaAgentChat}
              isThinking={$metaAgentThinking}
              isSleeping={$isSleeping}
              sleepDuration={$metaSleepStatus?.duration_ms}
              sleepReason={$metaSleepStatus?.reason}
            />
          {/if}
        </div>

        {#if $agentsWithOutputs.length > 0}
          <AgentResultsSection
            agents={$agentsWithOutputs}
            {processingAgentId}
            disabled={$metaAgentThinking}
            onProcessResults={handleProcessResults}
            onDismiss={handleDismissResult}
          />
        {/if}

        <ChatInput
          disabled={$metaAgentThinking && !$isSleeping}
          {hasApiKey}
          onSend={handleSendMessage}
        />
      </div>

      {#if showTodoPanel}
        <div class="side-panel">
          <MetaTaskProgress />
        </div>
      {/if}
    </div>
  {/if}
</PageLayout>

<VoiceSidebar onSendToChat={handleSendToChat} />

<style>
  .content-wrapper {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .messages-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-width: 0;
  }

  .messages-wrapper {
    flex: 1;
    overflow: hidden;
    position: relative;
    display: flex;
    flex-direction: column;
  }

  .side-panel {
    width: min(350px, 40%);
    min-width: 250px;
    flex-shrink: 0;
    background-color: var(--bg-secondary);
    border-left: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    animation: slideLeft 0.2s ease;
  }

  @keyframes slideLeft {
    from {
      transform: translateX(20px);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }
</style>
