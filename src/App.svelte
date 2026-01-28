<script lang="ts">
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import { invoke } from "@tauri-apps/api/core";
  import "./app.css";
  import { initResizeTracking } from "./lib/stores/resize";
  import AgentList from "./lib/components/AgentList.svelte";
  import NewAgentDialog from "./lib/components/NewAgentDialog.svelte";
  import ViewRouter from "./lib/components/ViewRouter.svelte";
  import ToastNotifications, { showToast } from "./lib/components/ToastNotifications.svelte";
  import RateLimitModal from "./lib/components/RateLimitModal.svelte";
  import WelcomeModal from "./lib/components/WelcomeModal.svelte";
  import { checkAndSetRateLimit } from "./lib/stores/rateLimit";
  import {
    agents,
    agentOutputs,
    appendOutput,
    appendToolEvent,
    addAgent,
    updateAgentStatus,
    updateAgentActivity,
    updateAgentStats,
    layoutMode,
    metaAgentThinking,
    addMetaAgentToolCall,
    openAgent,
    openChat,
    addOrchestratorToolCall,
    completeOrchestratorToolCall,
    addOrchestratorStateChange,
    addOrchestratorDecision,
    clearOrchestratorActivity,
    incrementStepToolCount,
    selectedAgentId,
    setCurrentPipelineId,
  } from "./lib/stores/agents";
  import {
    addPipeline,
    updatePipelineStatus,
    updatePipelinePhase,
    updatePhaseProgress,
  } from "./lib/stores/pipelines";
  import { autoPipelines, selectAutoPipeline } from "./lib/stores/autoPipelines";
  import { updateActivity, updateActivityDetail } from "./lib/stores/activity";
  import {
    addSecurityAlert,
    markAgentTerminated,
    markAgentSuspended,
    addPendingReview,
    removePendingReview,
    showAlertDetail,
    addPendingElevatedCommand,
    updateElevatedCommandStatus,
    showElevatedCommand,
  } from "./lib/stores/security";
  import { setMetaTodos } from "./lib/stores/metaTodos";
  import {
    pendingMetaQuestion,
    setMetaQuestion,
    clearMetaQuestion,
    addMetaUserUpdate,
    setMetaSleepStatus,
  } from "./lib/stores/metaAgentInteraction";
  import MetaAgentQuestion from "./lib/components/chat/MetaAgentQuestion.svelte";
  import SecurityAlertDetail from "./lib/components/SecurityAlertDetail.svelte";
  import NotificationsModal from "./lib/components/NotificationsModal.svelte";
  import ElevatedCommandModal from "./lib/components/ElevatedCommandModal.svelte";
  import {
    setupEventListeners,
    setupKeyboardShortcuts,
    refreshAutoPipeline,
  } from "$lib/services";
  import { InteractiveTutorial } from "$lib/components/tutorial";
  import { ContextualHelpPanel } from "$lib/components/help";
  import { tutorialStore } from "$lib/stores/tutorial.svelte";
  import type { Agent, AutoPipeline } from "./lib/types";
  import AttentionOverlay from "$lib/components/voice/AttentionOverlay.svelte";
  import { startAttentionSession, initAttentionListeners, attentionEnabled } from "$lib/stores/voice";

  let showNewAgentDialog = $state(false);
  let showDatabaseStats = $state(false);
  let showSettings = $state(false);
  let showInstructionPanel = $state(false);
  let showCommanderSettings = $state(false);
  let showWelcomeModal = $state(false);
  let helpOpen = $state(false);

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'F1') {
      e.preventDefault();
      helpOpen = !helpOpen;
    }
  }

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
    // Only show toast if attention mode is disabled (otherwise voice notification handles it)
    const isAttentionEnabled = get(attentionEnabled);
    if (!isAttentionEnabled) {
      showToast({
        type: "warning",
        message: "Agent is waiting for input",
        action: {
          label: "View",
          onClick: () => {
            selectedAgentId.set(agentId);
          },
        },
      });
    }

    // Trigger attention mode with actual agent result
    const currentAgents = get(agents);
    const agent = currentAgents.get(agentId);
    const title = agent?.title || agentId;
    const result = getAgentResult(agentId);
    startAttentionSession(agentId, title, result);
  }

  // Get the agent's actual result output to read aloud
  function getAgentResult(agentId: string): string {
    // Outputs are stored in a separate store, not on the agent object
    const outputs = get(agentOutputs).get(agentId) || [];
    console.log("[getAgentResult] Agent:", agentId, "outputs count:", outputs.length);
    console.log("[getAgentResult] Output types:", outputs.map((o: any) => o.type));

    if (outputs.length === 0) return "Task completed successfully";

    // Find the most recent "result" type output
    const resultOutput = outputs.slice().reverse().find((o: any) => o.type === "result");
    if (resultOutput?.content) {
      console.log("[getAgentResult] Found result output:", resultOutput.content.slice(0, 100));
      // Truncate long results for voice reading
      return resultOutput.content.slice(0, 500);
    }
    // Fallback to last text output
    const textOutput = outputs.slice().reverse().find((o: any) => o.type === "text");
    if (textOutput?.content) {
      console.log("[getAgentResult] Found text output:", textOutput.content.slice(0, 100));
      return textOutput.content.slice(0, 500);
    }
    // Final fallback to last output
    const lastOutput = outputs.slice(-1)[0];
    console.log("[getAgentResult] Last output:", lastOutput);
    return lastOutput?.content?.slice(0, 500) || "Task completed successfully";
  }

  onMount(() => {
    // Check if this is first run and show welcome modal
    invoke("get_config_status")
      .then((status: any) => {
        if (status.is_first_run) {
          showWelcomeModal = true;
        }
      })
      .catch((e) => {
        console.error("Failed to get config status:", e);
      });

    // Initialize resize tracking to prevent layout thrashing during resize
    const cleanupResize = initResizeTracking();

    // Setup keyboard shortcuts
    const cleanupKeyboard = setupKeyboardShortcuts(
      {
        onNewAgent: () => { showNewAgentDialog = true; },
        onToggleLayout: () => {
          layoutMode.update((current) => {
            if (current === 'single') return 'split';
            if (current === 'split') return 'grid';
            return 'single';
          });
        },
        onOpenChat: () => openChat(),
        onToggleDatabaseStats: () => { showDatabaseStats = !showDatabaseStats; },
        onSelectAgent: (agentIndex) => {
          const agentList = Array.from($agents.values());
          if (agentIndex < agentList.length) {
            selectedAgentId.set(agentList[agentIndex].id);
          }
        },
        onEscape: () => { showNewAgentDialog = false; },
      },
      {
        getAgents: () => Array.from($agents.values()),
      }
    );

    // Setup event listeners
    const cleanupEventsPromise = setupEventListeners({
      // Agent callbacks
      onAgentOutput: (agentId, output) => {
        appendOutput(agentId, output);
        // Check for rate limit errors in output content
        if (output.content && (output.type === "error" || output.type === "text")) {
          checkAndSetRateLimit(output.content);
        }
      },
      onToolEvent: (agentId, event) => {
        appendToolEvent(agentId, event);

        // Check if this agent belongs to an auto-pipeline step and increment tool count
        // Only count tool results (completed tools), not tool starts
        if (event.hookEventName === 'tool-result') {
          const pipelines = get(autoPipelines);
          for (const [_pipelineId, pipeline] of pipelines) {
            // Check each step to see if this agent belongs to it
            for (const step of pipeline.steps) {
              if (step.agent_id === agentId && step.step_number > 0) {
                incrementStepToolCount(step.step_number);
                break;
              }
            }
          }
        }
      },
      onAgentStatus: (agentId, status, agent) => {
        // Check if this is a new agent
        if (agent && !$agents.has(agentId)) {
          addAgent(agent);
        }
        updateAgentStatus(agentId, status as Agent["status"]);
        handleStatusChange(agentId, status);
      },
      onInputRequired: (agentId) => {
        updateAgentActivity(agentId, {
          pendingInput: true,
          isProcessing: false,
        });
        updateActivity(agentId, {
          pendingInput: true,
          isProcessing: false,
          lastActivity: new Date(),
        });
        handleInputRequired(agentId);
      },
      onAgentActivity: (agentId, activity) => {
        updateAgentActivity(agentId, activity);
        updateActivity(agentId, activity);
      },
      onAgentStats: (agentId, stats) => {
        updateAgentStats(agentId, stats);
      },
      onAgentActivityDetail: (agentId, detail) => {
        updateActivityDetail(agentId, detail);
      },

      // Meta-agent callbacks
      onMetaAgentThinking: (isThinking) => {
        metaAgentThinking.set(isThinking);
      },
      onMetaAgentToolCall: (toolCall) => {
        addMetaAgentToolCall(toolCall);
      },
      onNavigate: (agentId) => {
        openAgent(agentId);
      },
      onMetaAgentTodos: (event) => {
        setMetaTodos(event.todos);
      },
      onMetaAgentUserUpdate: (event) => {
        addMetaUserUpdate(event);
        // Show as toast notification for visibility
        showToast({
          type: event.level === "warning" ? "warning" :
                event.level === "success" ? "success" : "info",
          message: event.message,
          duration: 5000,
        });
      },
      onMetaAgentQuestion: (event) => {
        setMetaQuestion(event);
      },
      onMetaAgentSleep: (event) => {
        setMetaSleepStatus(event);
        if (event.status === "sleeping") {
          showToast({
            type: "info",
            message: event.reason || "System Commander is sleeping...",
            duration: 3000,
          });
        }
      },

      // Pipeline callbacks
      onPipelineCreated: (pipeline) => {
        addPipeline(pipeline);
      },
      onPipelineStatus: (pipelineId, status) => {
        updatePipelineStatus(pipelineId, status);
      },
      onPipelinePhase: (pipelineId, phase) => {
        updatePipelinePhase(pipelineId, phase);
      },
      onPhaseProgress: (data) => {
        updatePhaseProgress(data);
      },

      // Auto-pipeline callbacks
      onAutoPipelineStarted: (pipeline) => {
        // Clear previous orchestrator activity when a NEW pipeline starts
        clearOrchestratorActivity();
        // Set the pipeline ID for persistence
        setCurrentPipelineId(pipeline.id);
        autoPipelines.update(m => {
          m.set(pipeline.id, pipeline);
          return new Map(m);
        });
        selectAutoPipeline(pipeline.id);
        showToast({
          type: "info",
          message: "Auto-pipeline started",
        });
      },
      onAutoPipelineStep: async (pipelineId, stepNumber) => {
        await refreshAutoPipeline(pipelineId, autoPipelines.update);
        showToast({
          type: "info",
          message: `Auto-pipeline step ${stepNumber} completed`,
        });
      },
      onAutoPipelineComplete: async (pipelineId) => {
        await refreshAutoPipeline(pipelineId, autoPipelines.update);
        showToast({
          type: "success",
          message: "Auto-pipeline completed successfully!",
        });

        // Trigger attention mode with orchestrator summary
        const currentPipelines = get(autoPipelines);
        const pipeline = currentPipelines.get(pipelineId);
        const summary = pipeline?.final_summary || "Pipeline completed successfully";
        startAttentionSession(pipelineId, "Pipeline", summary);
      },
      onStepStatus: async (pipelineId, _stepNumber, _status) => {
        await refreshAutoPipeline(pipelineId, autoPipelines.update);
      },

      // Orchestrator callbacks
      onOrchestratorToolStart: (toolCall) => {
        addOrchestratorToolCall({
          ...toolCall,
          completed: false,  // Mark as in-progress
        });
      },
      onOrchestratorToolComplete: (data) => {
        const { toolName, isError, stepNumber } = data;

        // Mark the tool as completed in the store
        completeOrchestratorToolCall(toolName, isError);

        // Increment step tool count for real-time tracking
        if (stepNumber && stepNumber > 0) {
          incrementStepToolCount(stepNumber);
        }
      },
      onOrchestratorStateChange: (stateChange) => {
        addOrchestratorStateChange(stateChange);
      },
      onOrchestratorDecision: (decision) => {
        addOrchestratorDecision(decision);
      },

      // Security callbacks
      onSecurityAlert: (payload) => {
        // Create alert with full threat details
        const alert = {
          agentId: payload.agent_id,
          alertId: payload.alert_id,
          severity: payload.risk_level,
          title: payload.title,
          description: payload.description,
          timestamp: new Date(payload.timestamp),
          threats: payload.threats || [],
          overallConfidence: payload.overall_confidence || 0,
        };

        // Add to store
        addSecurityAlert(alert);

        // Show toast notification
        const toastType = payload.risk_level === "critical" || payload.risk_level === "high"
          ? "error"
          : "warning";

        showToast({
          type: toastType,
          message: `Security: ${payload.title}`,
          action: {
            label: "View Agent",
            onClick: () => selectedAgentId.set(payload.agent_id),
          },
          secondaryAction: {
            label: "More Info",
            onClick: () => showAlertDetail(alert),
          },
          duration: payload.risk_level === "critical" ? 0 : 6000,
        });
      },

      onSecurityAgentTerminated: (payload) => {
        markAgentTerminated(payload.agent_id);
        updateAgentStatus(payload.agent_id, "stopped");

        showToast({
          type: "error",
          message: "Agent terminated: Critical security threat",
          action: {
            label: "View Agent",
            onClick: () => selectedAgentId.set(payload.agent_id),
          },
          duration: 0, // Don't auto-dismiss
        });
      },

      onSecurityAgentSuspended: (payload) => {
        markAgentSuspended(payload.agent_id);
        updateAgentStatus(payload.agent_id, "stopped");

        showToast({
          type: "warning",
          message: "Agent suspended: Awaiting security review",
          action: {
            label: "View Agent",
            onClick: () => selectedAgentId.set(payload.agent_id),
          },
          duration: 0, // Don't auto-dismiss
        });
      },

      onSecurityPendingReview: (payload) => {
        addPendingReview({
          id: payload.id,
          batchId: payload.batch_id,
          summary: payload.analysis_summary,
          riskLevel: payload.overall_risk_level,
          recommendedAction: payload.recommended_action,
          agentId: payload.agent_id,
          createdAt: new Date(payload.created_at),
        });

        showToast({
          type: "warning",
          message: "Security review required",
          action: {
            label: "Review",
            onClick: () => {
              if (payload.agent_id) {
                selectedAgentId.set(payload.agent_id);
              }
            },
          },
          duration: 0, // Don't auto-dismiss
        });
      },

      onSecurityReviewCompleted: (payload) => {
        removePendingReview(payload.review_id);

        showToast({
          type: payload.approved ? "success" : "info",
          message: payload.approved ? "Security review approved" : "Security review dismissed",
        });
      },

      // Toast callback - show toasts from backend
      onToast: (toast) => {
        showToast(toast);
      },

      // Elevated command callbacks
      onElevatedCommandRequest: (request) => {
        addPendingElevatedCommand(request);
        // Auto-show modal for the new request
        showElevatedCommand(request);
        // Also show a toast notification
        const riskLabel = request.riskLevel === "high" ? "HIGH RISK" :
                          request.riskLevel === "suspicious" ? "SUSPICIOUS" : "";
        showToast({
          type: request.riskLevel === "high" ? "error" : "warning",
          message: `${riskLabel} Sudo approval needed`.trim(),
          action: {
            label: "Review",
            onClick: () => showElevatedCommand(request),
          },
          duration: 0, // Don't auto-dismiss
        });
      },
      onElevatedCommandStatus: (requestId, status, error) => {
        updateElevatedCommandStatus(requestId, status as any);
        if (status === "expired" || status === "failed") {
          showToast({
            type: "warning",
            message: error || `Elevated command ${status}`,
          });
        }
      },
    });

    // Initialize attention mode listeners
    initAttentionListeners();

    return () => {
      cleanupResize();
      cleanupKeyboard();
      cleanupEventsPromise.then((cleanup) => cleanup());
    };
  });
</script>

<div class="app">
  <AgentList
    onNewAgent={() => (showNewAgentDialog = true)}
    onToggleDatabaseStats={() => (showDatabaseStats = !showDatabaseStats)}
    onOpenSettings={() => (showSettings = !showSettings)}
    onOpenInstructions={() => (showInstructionPanel = !showInstructionPanel)}
    onOpenCommanderSettings={() => (showCommanderSettings = !showCommanderSettings)}
  />
  <div class="main-content">
    <ViewRouter
      {showDatabaseStats}
      {showInstructionPanel}
      {showSettings}
      {showCommanderSettings}
      onCloseInstructions={() => (showInstructionPanel = false)}
      onCloseSettings={() => (showSettings = false)}
      onCloseCommanderSettings={() => (showCommanderSettings = false)}
    />
  </div>
</div>

<ToastNotifications />
<RateLimitModal />
<SecurityAlertDetail />
<NotificationsModal />
<ElevatedCommandModal />
<WelcomeModal show={showWelcomeModal} onClose={() => (showWelcomeModal = false)} on:startTutorial={() => tutorialStore.start()} />

{#if showNewAgentDialog}
  <NewAgentDialog onClose={() => (showNewAgentDialog = false)} />
{/if}

<!-- Global overlays -->
<InteractiveTutorial />
<ContextualHelpPanel bind:open={helpOpen} />
<AttentionOverlay />

<!-- Meta-agent question dialog -->
{#if $pendingMetaQuestion}
  <div class="meta-question-overlay">
    <MetaAgentQuestion
      question={$pendingMetaQuestion}
      onAnswered={() => clearMetaQuestion()}
    />
  </div>
{/if}

<svelte:window onkeydown={handleKeydown} />

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

  .meta-question-overlay {
    position: fixed;
    bottom: 80px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 1000;
    width: 90%;
    max-width: 600px;
  }
</style>
