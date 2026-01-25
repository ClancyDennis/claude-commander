<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { metaAgentChat, metaAgentThinking, addChatMessage, agentsWithOutputs } from "../stores/agents";
  import { voiceSidebarOpen } from "../stores/voice";
  import type { ChatResponse, ConfigStatus, ImageAttachment } from "../types";

  // Import sub-components
  import { PageLayout } from "./ui/layout";
  import ChatHeader from "./chat/ChatHeader.svelte";
  import ChatInput from "./chat/ChatInput.svelte";
  import ChatEmptyState from "./chat/ChatEmptyState.svelte";
  import ChatLockedState from "./chat/ChatLockedState.svelte";
  import ChatMessageList from "./chat/ChatMessageList.svelte";
  import AgentResultsSection from "./chat/AgentResultsSection.svelte";
  import VoiceSidebar from "./voice/VoiceSidebar.svelte";

  // State
  let hasApiKey = $state(true); // Default to true to avoid flash
  let hasOpenAiKey = $state(false);
  let configLoaded = $state(false);
  let configPath = $state("");
  let processingAgentId = $state<string | null>(null);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      const config = await invoke<ConfigStatus>("get_config_status");
      hasApiKey = config.api_keys.some((key) => key.is_configured);
      hasOpenAiKey = config.api_keys.some(
        (key) => key.provider.toLowerCase() === "openai" && key.is_configured
      );
      configPath = config.config_path;
      configLoaded = true;
    } catch (err) {
      console.error("Failed to fetch config:", err);
      configLoaded = true;
    }
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
    } catch (e) {
      console.error("Error processing agent results:", e);
      error = e instanceof Error ? e.message : String(e);
    } finally {
      processingAgentId = null;
    }
  }
</script>

<PageLayout>
  <ChatHeader
    isThinking={$metaAgentThinking}
    onClear={handleClear}
    {hasOpenAiKey}
    onVoiceClick={handleVoiceClick}
  />

  <div class="messages-wrapper">
    {#if configLoaded && !hasApiKey}
      <ChatLockedState {configPath} onOpenConfigDir={openConfigDir} />
    {:else if $metaAgentChat.length === 0}
      <ChatEmptyState />
    {:else}
      <ChatMessageList messages={$metaAgentChat} isThinking={$metaAgentThinking} />
    {/if}
  </div>

  {#if $agentsWithOutputs.length > 0}
    <AgentResultsSection
      agents={$agentsWithOutputs}
      {processingAgentId}
      disabled={$metaAgentThinking}
      onProcessResults={handleProcessResults}
    />
  {/if}

  <ChatInput
    disabled={$metaAgentThinking}
    {hasApiKey}
    onSend={handleSendMessage}
  />
</PageLayout>

<VoiceSidebar onSendToChat={handleSendToChat} />

<style>
  .messages-wrapper {
    flex: 1;
    overflow: hidden;
    position: relative;
    display: flex;
    flex-direction: column;
  }
</style>
