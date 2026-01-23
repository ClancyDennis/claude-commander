<script lang="ts">
  import { selectedHistoricalRun } from "../stores/agents";
  import type {
    AgentRun,
    OrchestratorToolCall,
    OrchestratorStateChange,
    OrchestratorDecision,
    AgentOutputRecord
  } from "../types";
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

  // Derived values for tab counts
  let activityCount = $derived(toolCalls.length + stateChanges.length + decisions.length);
  let hasPipeline = $derived(!!$selectedHistoricalRun?.pipeline_id);
</script>

{#if $selectedHistoricalRun}
  <main class="historical-run-view">
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
  </main>
{:else}
  <div class="empty-view">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
      <path d="M3 3v5h5"/>
      <circle cx="12" cy="12" r="1"/>
    </svg>
    <p>Select a historical run to view details</p>
  </div>
{/if}

<style>
  .historical-run-view {
    height: 100%;
    display: flex;
    flex-direction: column;
    background-color: var(--bg-primary);
    overflow: hidden;
  }

  /* Tab content area */
  .tab-content {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  /* Empty view (no run selected) */
  .empty-view {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-md);
    color: var(--text-muted);
  }

  .empty-view svg {
    width: 80px;
    height: 80px;
    opacity: 0.4;
  }

  .empty-view p {
    font-size: 15px;
  }

  /* Scrollbar styles */
  .tab-content::-webkit-scrollbar {
    width: 6px;
  }

  .tab-content::-webkit-scrollbar-track {
    background: transparent;
  }

  .tab-content::-webkit-scrollbar-thumb {
    background: var(--border);
    border-radius: 3px;
  }

  .tab-content::-webkit-scrollbar-thumb:hover {
    background: var(--accent);
  }
</style>
