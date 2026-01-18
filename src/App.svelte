<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import "./app.css";
  import AgentList from "./lib/components/AgentList.svelte";
  import AgentView from "./lib/components/AgentView.svelte";
  import ChatView from "./lib/components/ChatView.svelte";
  import HistoricalRunView from "./lib/components/HistoricalRunView.svelte";
  import NewAgentDialog from "./lib/components/NewAgentDialog.svelte";
  import LayoutManager from "./lib/components/LayoutManager.svelte";
  import SplitView from "./lib/components/SplitView.svelte";
  import GridView from "./lib/components/GridView.svelte";
  import DatabaseStats from "./lib/components/DatabaseStats.svelte";
  import PhaseProgress from "./lib/components/PhaseProgress.svelte";
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
    sidebarMode,
    selectedHistoricalRun,
    addOrchestratorToolCall,
    addOrchestratorStateChange,
    addOrchestratorDecision,
    clearOrchestratorActivity,
    incrementStepToolCount,
  } from "./lib/stores/agents";
  import {
    pipelines,
    selectedPipelineId,
    addPipeline,
    updatePipelineStatus,
    updatePipelinePhase,
    updatePhaseProgress,
  } from "./lib/stores/pipelines";
  import { autoPipelines, selectedAutoPipelineId, selectAutoPipeline } from "./lib/stores/autoPipelines";
  import AutoPipelineView from "./lib/components/AutoPipelineView.svelte";
  import type { AutoPipeline } from "./lib/types";
  import type { Pipeline, PhaseProgressData } from "./lib/stores/pipelines";
  import { updateActivity } from "./lib/stores/activity";
  import type { Agent, AgentOutput, ToolEvent, AgentStatusEvent, AgentInputRequiredEvent, AgentActivityEvent, AgentStatsEvent, MetaAgentThinkingEvent, MetaAgentToolCallEvent } from "./lib/types";

  let showNewAgentDialog = $state(false);
  let showDatabaseStats = $state(false);

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

    // Cmd/Ctrl + Shift + D: Toggle database stats
    if (isMod && e.shiftKey && e.key === 'D') {
      e.preventDefault();
      showDatabaseStats = !showDatabaseStats;
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
      console.log("[Frontend] Received agent:tool event:", event.payload.tool_name, event.payload.hook_event_name);
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
      console.log("[Frontend] Received agent:status event:", event.payload.agent_id, event.payload.status);
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
        console.log("[Frontend] Received agent:activity event:", event.payload.agent_id);
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

    // Listen for pipeline creation events
    const unlistenPipelineCreated = listen<{
      pipeline_id: string;
      working_dir: string;
      user_request: string;
    }>("pipeline:created", (event) => {
      const pipeline: Pipeline = {
        id: event.payload.pipeline_id,
        workingDir: event.payload.working_dir,
        userRequest: event.payload.user_request,
        status: 'planning',
        createdAt: new Date(),
      };
      addPipeline(pipeline);
    });

    // Listen for pipeline status events
    const unlistenPipelineStatus = listen<{
      pipeline_id: string;
      status: Pipeline['status'];
    }>("pipeline:status", (event) => {
      updatePipelineStatus(event.payload.pipeline_id, event.payload.status);
    });

    // Listen for pipeline phase events
    const unlistenPipelinePhase = listen<{
      pipeline_id: string;
      phase: string;
    }>("pipeline:phase", (event) => {
      updatePipelinePhase(event.payload.pipeline_id, event.payload.phase);
    });

    // Listen for phase progress events
    const unlistenPhaseProgress = listen<PhaseProgressData>("pipeline:progress", (event) => {
      updatePhaseProgress(event.payload);
    });

    // Listen for auto-pipeline started event
    const unlistenAutoPipelineStarted = listen<AutoPipeline | { pipeline_id: string }>("auto_pipeline:started", (event) => {
      const payload = event.payload;
      const pipelineId = 'id' in payload ? payload.id : payload.pipeline_id;

      console.log("[Frontend] Auto-pipeline started:", pipelineId);

      // Clear previous orchestrator activity when a NEW pipeline starts
      clearOrchestratorActivity();

      // Check if we received the full pipeline object
      if ('id' in payload && 'status' in payload) {
        // We got the full object, update store immediately
        autoPipelines.update(m => {
          m.set(payload.id, payload as AutoPipeline);
          return new Map(m);
        });
        selectAutoPipeline(payload.id);
      } else {
        // Fallback to fetch if we only got the ID
        import("@tauri-apps/api/core").then(({ invoke }) => {
          invoke<AutoPipeline>("get_auto_pipeline", {
            pipelineId
          }).then(pipeline => {
            autoPipelines.update(m => {
              m.set(pipeline.id, pipeline);
              return new Map(m);
            });
            // Immediately select this pipeline to show orchestrator view
            selectAutoPipeline(pipelineId);
          }).catch(err => {
            console.error("[Frontend] Failed to fetch pipeline:", err);
            // Still try to select it
            selectAutoPipeline(pipelineId);
          });
        });
      }

      showToast({
        type: "info",
        message: "Auto-pipeline started",
      });
    });

    // Listen for auto-pipeline step completion
    const unlistenAutoPipelineStep = listen<{
      pipeline_id: string;
      step_number: number;
      output: any;
    }>("auto_pipeline:step_completed", (event) => {
      // Fetch updated pipeline data from backend
      import("@tauri-apps/api/core").then(({ invoke }) => {
        invoke<AutoPipeline>("get_auto_pipeline", {
          pipelineId: event.payload.pipeline_id
        }).then(pipeline => {
          autoPipelines.update(m => {
            m.set(pipeline.id, pipeline);
            return new Map(m);
          });
        });
      });

      showToast({
        type: "info",
        message: `Auto-pipeline step ${event.payload.step_number} completed`,
      });
    });

    // Listen for auto-pipeline completion
    const unlistenAutoPipelineComplete = listen<{
      pipeline_id: string;
      verification_report: any;
    }>("auto_pipeline:completed", (event) => {
      // Fetch updated pipeline data from backend
      import("@tauri-apps/api/core").then(({ invoke }) => {
        invoke<AutoPipeline>("get_auto_pipeline", {
          pipelineId: event.payload.pipeline_id
        }).then(pipeline => {
          autoPipelines.update(m => {
            m.set(pipeline.id, pipeline);
            return new Map(m);
          });
        });
      });

      showToast({
        type: "success",
        message: "Auto-pipeline completed successfully!",
      });
    });

    // Listen for step status changes (Running, Completed, etc.)
    const unlistenStepStatus = listen<{
      pipeline_id: string;
      step_number: number;
      status: string;
    }>("auto_pipeline:step_status", (event) => {
      console.log("[Frontend] Step status change:", event.payload.step_number, event.payload.status);
      // Fetch updated pipeline data from backend to get the full state
      import("@tauri-apps/api/core").then(({ invoke }) => {
        invoke<AutoPipeline>("get_auto_pipeline", {
          pipelineId: event.payload.pipeline_id
        }).then(pipeline => {
          autoPipelines.update(m => {
            m.set(pipeline.id, pipeline);
            return new Map(m);
          });
        });
      });
    });

    // Listen for orchestrator tool start events
    const unlistenOrchestratorToolStart = listen<{
      tool_name: string;
      tool_input: Record<string, unknown>;
      current_state: string;
      iteration: number;
    }>("orchestrator:tool_start", (event) => {
      console.log("[Frontend] Orchestrator tool start:", event.payload.tool_name);
      addOrchestratorToolCall({
        tool_name: event.payload.tool_name,
        tool_input: event.payload.tool_input,
        current_state: event.payload.current_state,
        iteration: event.payload.iteration,
        timestamp: Date.now(),
      });
    });

    // Listen for orchestrator tool complete events
    const unlistenOrchestratorToolComplete = listen<{
      tool_name: string;
      is_error: boolean;
      summary: string;
      current_state: string;
      step_number?: number;
    }>("orchestrator:tool_complete", (event) => {
      console.log("[Frontend] Orchestrator tool complete:", event.payload.tool_name, event.payload.is_error ? "(error)" : "", "step:", event.payload.step_number);
      // Increment step tool count for real-time tracking
      if (event.payload.step_number && event.payload.step_number > 0) {
        incrementStepToolCount(event.payload.step_number);
      }
    });

    // Listen for orchestrator state change events
    const unlistenOrchestratorStateChange = listen<{
      old_state: string;
      new_state: string;
      iteration: number;
      generated_skills: number;
      generated_subagents: number;
      claudemd_generated: boolean;
    }>("orchestrator:state_changed", (event) => {
      console.log("[Frontend] Orchestrator state change:", event.payload.old_state, "->", event.payload.new_state);
      addOrchestratorStateChange({
        old_state: event.payload.old_state,
        new_state: event.payload.new_state,
        iteration: event.payload.iteration,
        generated_skills: event.payload.generated_skills,
        generated_subagents: event.payload.generated_subagents,
        claudemd_generated: event.payload.claudemd_generated,
        timestamp: Date.now(),
      });
    });

    // Listen for orchestrator decision events
    const unlistenOrchestratorDecision = listen<{
      pipeline_id: string;
      decision: string;
      reasoning: string;
      issues: string[];
      suggestions: string[];
    }>("auto_pipeline:decision", (event) => {
      console.log("[Frontend] Orchestrator decision:", event.payload.decision);
      addOrchestratorDecision({
        pipeline_id: event.payload.pipeline_id,
        decision: event.payload.decision as any,
        reasoning: event.payload.reasoning,
        issues: event.payload.issues,
        suggestions: event.payload.suggestions,
        timestamp: Date.now(),
      });
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
      unlistenPipelineCreated.then((f) => f());
      unlistenPipelineStatus.then((f) => f());
      unlistenPipelinePhase.then((f) => f());
      unlistenPhaseProgress.then((f) => f());
      unlistenAutoPipelineStarted.then((f) => f());
      unlistenAutoPipelineStep.then((f) => f());
      unlistenAutoPipelineComplete.then((f) => f());
      unlistenStepStatus.then((f) => f());
      unlistenOrchestratorToolStart.then((f) => f());
      unlistenOrchestratorToolComplete.then((f) => f());
      unlistenOrchestratorStateChange.then((f) => f());
      unlistenOrchestratorDecision.then((f) => f());
    };
  });
</script>

<div class="app">
  <AgentList
    onNewAgent={() => (showNewAgentDialog = true)}
    onToggleDatabaseStats={() => (showDatabaseStats = !showDatabaseStats)}
  />
  <div class="main-content">
    {#if showDatabaseStats}
      <div class="database-stats-container">
        <DatabaseStats />
      </div>
    {/if}
    {#if $selectedAutoPipelineId}
      <!-- Show auto-pipeline view with orchestrator activity -->
      <div class="pipeline-view-container">
        <AutoPipelineView pipelineId={$selectedAutoPipelineId} />
      </div>
    {:else if $selectedPipelineId}
      <!-- Show pipeline view when a pipeline is selected -->
      <div class="pipeline-view-container">
        <PhaseProgress pipelineId={$selectedPipelineId} />
      </div>
    {:else if $sidebarMode === 'history' && $selectedHistoricalRun}
      <!-- Show historical run view when in history mode and a run is selected -->
      <HistoricalRunView />
    {:else if $viewMode === 'chat'}
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

  .database-stats-container {
    padding: var(--space-lg);
    border-bottom: 1px solid var(--border);
  }

  .pipeline-view-container {
    flex: 1;
    overflow: auto;
    padding: var(--space-lg);
  }
</style>
