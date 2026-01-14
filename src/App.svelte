<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import "./app.css";
  import AgentList from "./lib/components/AgentList.svelte";
  import AgentView from "./lib/components/AgentView.svelte";
  import ChatView from "./lib/components/ChatView.svelte";
  import NewAgentDialog from "./lib/components/NewAgentDialog.svelte";
  import LayoutManager from "./lib/components/LayoutManager.svelte";
  import SplitView from "./lib/components/SplitView.svelte";
  import GridView from "./lib/components/GridView.svelte";
  import ToastNotifications, { showToast } from "./lib/components/ToastNotifications.svelte";
  import {
    agents,
    appendOutput,
    appendToolEvent,
    addAgent,
    updateAgentStatus,
    updateAgentActivity,
    updateAgentStats,
    layoutMode,
    viewMode,
    metaAgentThinking,
    addMetaAgentToolCall,
    openAgent,
    openChat,
  } from "./lib/stores/agents";
  import { updateActivity } from "./lib/stores/activity";
  import type { Agent, AgentOutput, ToolEvent, AgentStatusEvent, AgentInputRequiredEvent, AgentActivityEvent, AgentStatsEvent, MetaAgentThinkingEvent, MetaAgentToolCallEvent } from "./lib/types";

  let showNewAgentDialog = $state(false);

  // Show toast notifications for key events
  function handleStatusChange(agentId: string, status: string) {
    const statusMessages: Record<string, { type: "info" | "success" | "warning" | "error"; message: string }> = {
      stopped: { type: "info", message: "Agent stopped" },
      error: { type: "error", message: "Agent encountered an error" },
    };

    const toast = statusMessages[status];
    if (toast) {
      showToast(toast);
    }
  }

  function handleInputRequired(agentId: string) {
    showToast({
      type: "warning",
      message: "Agent is waiting for input",
      action: {
        label: "View",
        onClick: () => {
          // Switch to the agent that needs input
          import("./lib/stores/agents").then(({ selectedAgentId }) => {
            selectedAgentId.set(agentId);
          });
        },
      },
    });
  }

  // Keyboard shortcuts
  function handleKeyboardShortcuts(e: KeyboardEvent) {
    const isMod = e.ctrlKey || e.metaKey;

    // Cmd/Ctrl + N: New agent
    if (isMod && e.key === 'n') {
      e.preventDefault();
      showNewAgentDialog = true;
      return;
    }

    // Cmd/Ctrl + /: Toggle layout mode
    if (isMod && e.key === '/') {
      e.preventDefault();
      layoutMode.update((current) => {
        if (current === 'single') return 'split';
        if (current === 'split') return 'grid';
        return 'single';
      });
      return;
    }

    // Cmd/Ctrl + Shift + C: Open chat
    if (isMod && e.shiftKey && e.key === 'C') {
      e.preventDefault();
      openChat();
      return;
    }

    // Cmd/Ctrl + 1-9: Select agent by number
    if (isMod && e.key >= '1' && e.key <= '9') {
      e.preventDefault();
      const agentIndex = parseInt(e.key) - 1;
      const agentList = Array.from($agents.values());
      if (agentIndex < agentList.length) {
        import("./lib/stores/agents").then(({ selectedAgentId }) => {
          selectedAgentId.set(agentList[agentIndex].id);
        });
      }
      return;
    }

    // Escape: Close dialogs
    if (e.key === 'Escape') {
      showNewAgentDialog = false;
      return;
    }
  }

  onMount(() => {
    // Add keyboard event listener
    window.addEventListener('keydown', handleKeyboardShortcuts);

    // Listen for agent output events
    const unlistenOutput = listen<{
      agent_id: string;
      output_type: string;
      content: string;
    }>("agent:output", (event) => {
      console.log("[Frontend] Received agent:output event:", event.payload);
      const output: AgentOutput = {
        agentId: event.payload.agent_id,
        type: event.payload.output_type as AgentOutput["type"],
        content: event.payload.content,
        timestamp: new Date(),
      };
      console.log("[Frontend] Appending output to agent:", event.payload.agent_id);
      appendOutput(event.payload.agent_id, output);
    });

    // Listen for tool events from hooks
    const unlistenTool = listen<{
      agent_id: string;
      session_id: string;
      hook_event_name: string;
      tool_name: string;
      tool_input: Record<string, unknown>;
      tool_response?: Record<string, unknown>;
      tool_call_id: string;
      status?: string;
      error_message?: string;
      execution_time_ms?: number;
      timestamp: number;
    }>("agent:tool", (event) => {
      const toolEvent: ToolEvent = {
        agentId: event.payload.agent_id,
        sessionId: event.payload.session_id,
        hookEventName: event.payload.hook_event_name,
        toolName: event.payload.tool_name,
        toolInput: event.payload.tool_input,
        toolResponse: event.payload.tool_response,
        timestamp: new Date(event.payload.timestamp),
        toolCallId: event.payload.tool_call_id,
        status: event.payload.status,
        errorMessage: event.payload.error_message,
        executionTimeMs: event.payload.execution_time_ms,
      };
      appendToolEvent(event.payload.agent_id, toolEvent);
    });

    // Listen for status changes and new agents
    const unlistenStatus = listen<AgentStatusEvent>("agent:status", (event) => {
      // Check if this is a new agent (has info property)
      if (event.payload.info) {
        const info = event.payload.info;
        // Create agent object from info
        const agent: Agent = {
          id: info.id,
          workingDir: info.working_dir,
          status: event.payload.status,
          createdAt: new Date(),
          lastActivity: info.last_activity ? new Date(info.last_activity) : undefined,
          isProcessing: info.is_processing,
          pendingInput: info.pending_input,
          githubContext: info.github_context ? {
            repositoryUrl: info.github_context.repository_url,
            owner: info.github_context.owner,
            repo: info.github_context.repo,
            branch: info.github_context.branch,
            commitSha: info.github_context.commit_sha,
            lastSynced: info.github_context.last_synced,
          } : undefined,
        };

        // Check if agent already exists
        if (!$agents.has(event.payload.agent_id)) {
          addAgent(agent);
        }
      }

      // Update status (works for both new and existing agents)
      updateAgentStatus(
        event.payload.agent_id,
        event.payload.status
      );
      handleStatusChange(event.payload.agent_id, event.payload.status);
    });

    // Listen for input required events
    const unlistenInputRequired = listen<AgentInputRequiredEvent>(
      "agent:input_required",
      (event) => {
        updateAgentActivity(event.payload.agent_id, {
          pendingInput: true,
          isProcessing: false,
        });

        updateActivity(event.payload.agent_id, {
          pendingInput: true,
          isProcessing: false,
          lastActivity: new Date(),
        });

        handleInputRequired(event.payload.agent_id);
      }
    );

    // Listen for activity events
    const unlistenActivity = listen<AgentActivityEvent>(
      "agent:activity",
      (event) => {
        const lastActivity = event.payload.last_activity
          ? new Date(event.payload.last_activity)
          : new Date();

        updateAgentActivity(event.payload.agent_id, {
          isProcessing: event.payload.is_processing,
          pendingInput: event.payload.pending_input,
          lastActivity,
        });

        updateActivity(event.payload.agent_id, {
          isProcessing: event.payload.is_processing,
          pendingInput: event.payload.pending_input,
          lastActivity,
        });
      }
    );

    // Listen for statistics events
    const unlistenStats = listen<AgentStatsEvent>("agent:stats", (event) => {
      const stats = {
        agentId: event.payload.stats.agent_id,
        totalPrompts: event.payload.stats.total_prompts,
        totalToolCalls: event.payload.stats.total_tool_calls,
        totalOutputBytes: event.payload.stats.total_output_bytes,
        sessionStart: event.payload.stats.session_start,
        lastActivity: event.payload.stats.last_activity,
        totalTokensUsed: event.payload.stats.total_tokens_used,
        totalCostUsd: event.payload.stats.total_cost_usd,
      };
      updateAgentStats(event.payload.agent_id, stats);
    });

    // Listen for meta-agent thinking events
    const unlistenThinking = listen<MetaAgentThinkingEvent>("meta-agent:thinking", (event) => {
      metaAgentThinking.set(event.payload.is_thinking);
    });

    // Listen for meta-agent tool call events
    const unlistenToolCall = listen<MetaAgentToolCallEvent>("meta-agent:tool-call", (event) => {
      addMetaAgentToolCall(event.payload);
    });

    // Listen for navigation events from meta-agent
    const unlistenNavigate = listen<{ agent_id: string }>("agent:navigate", (event) => {
      openAgent(event.payload.agent_id);
    });

    return () => {
      window.removeEventListener('keydown', handleKeyboardShortcuts);
      unlistenOutput.then((f) => f());
      unlistenTool.then((f) => f());
      unlistenStatus.then((f) => f());
      unlistenInputRequired.then((f) => f());
      unlistenActivity.then((f) => f());
      unlistenStats.then((f) => f());
      unlistenThinking.then((f) => f());
      unlistenToolCall.then((f) => f());
      unlistenNavigate.then((f) => f());
    };
  });
</script>

<div class="app">
  <AgentList onNewAgent={() => (showNewAgentDialog = true)} />
  <div class="main-content">
    {#if $viewMode === 'chat'}
      <ChatView />
    {:else}
      <LayoutManager />
      {#if $layoutMode === 'single'}
        <AgentView />
      {:else if $layoutMode === 'split'}
        <SplitView direction="horizontal" />
      {:else if $layoutMode === 'grid'}
        <GridView />
      {/if}
    {/if}
  </div>
</div>

<ToastNotifications />

{#if showNewAgentDialog}
  <NewAgentDialog onClose={() => (showNewAgentDialog = false)} />
{/if}

<style>
  .app {
    height: 100%;
    display: flex;
  }

  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
</style>
