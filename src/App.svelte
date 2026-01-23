<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import "./app.css";
  import { initResizeTracking } from "./lib/stores/resize";
  import AgentList from "./lib/components/AgentList.svelte";
  import AgentView from "./lib/components/AgentView.svelte";
  import ChatView from "./lib/components/ChatView.svelte";
  import HistoricalRunView from "./lib/components/HistoricalRunView.svelte";
  import NewAgentDialog from "./lib/components/NewAgentDialog.svelte";
  import LayoutManager from "./lib/components/LayoutManager.svelte";
  import SplitView from "./lib/components/SplitView.svelte";
  import GridView from "./lib/components/GridView.svelte";
  import DatabaseStats from "./lib/components/DatabaseStats.svelte";
  import Settings from "./lib/components/Settings.svelte";
  import PhaseProgress from "./lib/components/PhaseProgress.svelte";
  import ToastNotifications, { showToast } from "./lib/components/ToastNotifications.svelte";
  import RateLimitModal from "./lib/components/RateLimitModal.svelte";
  import WelcomeModal from "./lib/components/WelcomeModal.svelte";
  import { checkAndSetRateLimit } from "./lib/stores/rateLimit";
  import AutoPipelineView from "./lib/components/AutoPipelineView.svelte";
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
    completeOrchestratorToolCall,
    addOrchestratorStateChange,
    addOrchestratorDecision,
    clearOrchestratorActivity,
    incrementStepToolCount,
    selectedAgentId,
    setCurrentPipelineId,
  } from "./lib/stores/agents";
  import {
    selectedPipelineId,
    addPipeline,
    updatePipelineStatus,
    updatePipelinePhase,
    updatePhaseProgress,
  } from "./lib/stores/pipelines";
  import { autoPipelines, selectedAutoPipelineId, selectAutoPipeline } from "./lib/stores/autoPipelines";
  import { updateActivity } from "./lib/stores/activity";
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
    pendingElevatedCount,
  } from "./lib/stores/security";
  import SecurityAlertDetail from "./lib/components/SecurityAlertDetail.svelte";
  import NotificationsModal from "./lib/components/NotificationsModal.svelte";
  import ElevatedCommandModal from "./lib/components/ElevatedCommandModal.svelte";
  import {
    setupEventListeners,
    setupKeyboardShortcuts,
    refreshAutoPipeline,
  } from "$lib/services";
  import type { Agent, AutoPipeline } from "./lib/types";

  let showNewAgentDialog = $state(false);
  let showDatabaseStats = $state(false);
  let showSettings = $state(false);
  let showWelcomeModal = $state(false);

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
          selectedAgentId.set(agentId);
        },
      },
    });
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
  />
  <div class="main-content">
    {#if showDatabaseStats}
      <div class="database-stats-container">
        <DatabaseStats />
      </div>
    {/if}
    {#if showSettings}
      <div class="settings-container">
        <Settings onClose={() => (showSettings = false)} />
      </div>
    {:else if $selectedAutoPipelineId}
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
<RateLimitModal />
<SecurityAlertDetail />
<NotificationsModal />
<ElevatedCommandModal />
<WelcomeModal show={showWelcomeModal} onClose={() => (showWelcomeModal = false)} />

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

  .settings-container {
    flex: 1;
    overflow: auto;
    background: var(--bg-primary);
  }
</style>
