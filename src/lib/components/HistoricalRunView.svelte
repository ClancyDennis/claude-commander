<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { selectedHistoricalRun, addAgent, openAgent, setSidebarMode, pendingAgentPrompt } from "../stores/agents";
  import { showToast } from "./ToastNotifications.svelte";

  // Type for the resume run result from backend
  interface ResumeRunResult {
    agent_id: string;
    context_prompt: string | null;
  }
  import type {
    AgentRun,
    OrchestratorToolCall,
    OrchestratorStateChange,
    OrchestratorDecision,
    AgentOutputRecord
  } from "../types";
  import { PageLayout } from './ui/layout';
  import { History } from './ui/icons';
  import {
    RunHeader,
    RunStats,
    RunTabs,
    OverviewTab,
    ActivityTab,
    OutputsTab,
    PromptsTab,
    loadPrompts,
    loadActivity,
    loadOutputs,
    type TabType,
    type PromptData
  } from './historical-run';

  // Active tab
  let activeTab = $state<TabType>('overview');

  // Scroll position for virtualized lists
  let scrollTop = $state(0);

  // Resume operation state
  let isResuming = $state(false);

  // Loading states
  let loadingPrompts = $state(false);
  let loadingActivity = $state(false);
  let loadingOutputs = $state(false);

  // Error states
  let promptsError = $state<string | null>(null);
  let activityError = $state<string | null>(null);
  let outputsError = $state<string | null>(null);

  // Data
  let prompts = $state<PromptData[]>([]);
  let toolCalls = $state<OrchestratorToolCall[]>([]);
  let stateChanges = $state<OrchestratorStateChange[]>([]);
  let decisions = $state<OrchestratorDecision[]>([]);
  let outputs = $state<AgentOutputRecord[]>([]);

  // Load data when historical run changes
  $effect(() => {
    if ($selectedHistoricalRun) {
      // Reset data
      prompts = [];
      toolCalls = [];
      stateChanges = [];
      decisions = [];
      outputs = [];

      // Load data for each tab
      loadAllData($selectedHistoricalRun);
    }
  });

  async function loadAllData(run: AgentRun) {
    // Load prompts
    loadPromptsData(run.agent_id);

    // Load activity if pipeline_id is present
    if (run.pipeline_id) {
      loadActivityData(run.pipeline_id);
    }

    // Load outputs
    loadOutputsData(run.agent_id, run.pipeline_id);
  }

  async function loadPromptsData(agentId: string) {
    loadingPrompts = true;
    promptsError = null;
    const result = await loadPrompts(agentId);
    prompts = result.data;
    promptsError = result.error;
    loadingPrompts = false;
  }

  async function loadActivityData(pipelineId: string) {
    loadingActivity = true;
    activityError = null;
    const result = await loadActivity(pipelineId);
    toolCalls = result.data.toolCalls;
    stateChanges = result.data.stateChanges;
    decisions = result.data.decisions;
    activityError = result.error;
    loadingActivity = false;
  }

  async function loadOutputsData(agentId: string, pipelineId?: string) {
    loadingOutputs = true;
    outputsError = null;
    const result = await loadOutputs(agentId, pipelineId);
    outputs = result.data;
    outputsError = result.error;
    loadingOutputs = false;
  }

  function handleScroll(event: Event) {
    const target = event.target as HTMLDivElement;
    scrollTop = target.scrollTop;
  }

  function handleTabChange(tab: TabType) {
    activeTab = tab;
  }

  async function handleResume(autoStart: boolean) {
    if (!$selectedHistoricalRun || isResuming) return;

    const runToResume = $selectedHistoricalRun;
    isResuming = true;

    try {
      const result = await invoke<ResumeRunResult>("resume_crashed_run", {
        agentId: runToResume.agent_id,
        autoStart: autoStart
      });

      // Add the new agent to the store
      addAgent({
        id: result.agent_id,
        workingDir: runToResume.working_dir,
        status: "running",
        isProcessing: autoStart,  // Only processing if auto-started
        pendingInput: !autoStart,  // Waiting for input if not auto-started
        lastActivity: new Date(),
      });

      // If not auto-starting, set pending prompt for input pre-fill
      if (!autoStart && result.context_prompt) {
        pendingAgentPrompt.set({
          agentId: result.agent_id,
          prompt: result.context_prompt
        });
      }

      // Switch to running agents view and select the new agent
      setSidebarMode('running');
      openAgent(result.agent_id);

      showToast({
        type: "success",
        message: autoStart
          ? "Run resumed successfully"
          : "Agent created - review the context and send when ready",
        duration: 3000
      });

    } catch (error) {
      console.error("Failed to resume run:", error);
      showToast({
        type: "error",
        message: `Failed to resume run: ${error}`,
        duration: 5000
      });
    } finally {
      isResuming = false;
    }
  }

  // Derived values for tab counts
  let activityCount = $derived(toolCalls.length + stateChanges.length + decisions.length);
  let hasPipeline = $derived(!!$selectedHistoricalRun?.pipeline_id);
</script>

{#if $selectedHistoricalRun}
  <PageLayout>
    <RunHeader run={$selectedHistoricalRun} />

    <RunStats run={$selectedHistoricalRun} />

    <RunTabs
      {activeTab}
      {hasPipeline}
      {activityCount}
      outputsCount={outputs.length}
      promptsCount={prompts.length}
      onTabChange={handleTabChange}
    />

    <div class="tab-content" onscroll={handleScroll}>
      {#if activeTab === 'overview'}
        <OverviewTab
          run={$selectedHistoricalRun}
          {prompts}
          {outputs}
          onResume={handleResume}
        />
      {:else if activeTab === 'activity'}
        <ActivityTab
          loading={loadingActivity}
          error={activityError}
          {hasPipeline}
          {toolCalls}
          {stateChanges}
          {decisions}
          {scrollTop}
        />
      {:else if activeTab === 'outputs'}
        <OutputsTab
          loading={loadingOutputs}
          error={outputsError}
          {outputs}
          {scrollTop}
        />
      {:else if activeTab === 'prompts'}
        <PromptsTab
          loading={loadingPrompts}
          error={promptsError}
          {prompts}
        />
      {/if}
    </div>
  </PageLayout>
{:else}
  <PageLayout variant="centered">
    <div class="empty-content">
      <History size={80} />
      <p>Select a historical run to view details</p>
    </div>
  </PageLayout>
{/if}

<style>
  /* Tab content area */
  .tab-content {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  /* Empty view content */
  .empty-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-4);
    color: var(--text-muted);
    opacity: 0.6;
  }

  .empty-content p {
    font-size: var(--text-base);
  }

  /* Scrollbar styles */
  .tab-content::-webkit-scrollbar {
    width: 6px;
  }

  .tab-content::-webkit-scrollbar-track {
    background: transparent;
  }

  .tab-content::-webkit-scrollbar-thumb {
    background: var(--border-hex);
    border-radius: 3px;
  }

  .tab-content::-webkit-scrollbar-thumb:hover {
    background: var(--accent-hex);
  }
</style>
