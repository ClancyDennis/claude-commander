<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { selectedHistoricalRun } from "../stores/agents";
  import type {
    AgentRun,
    OrchestratorToolCall,
    OrchestratorStateChange,
    OrchestratorDecision,
    AgentOutputRecord,
    OrchestratorToolCallRecord,
    OrchestratorStateChangeRecord,
    OrchestratorDecisionRecord
  } from "../types";
  import { formatTimeAbsolute, formatDurationVerbose, formatBytes, formatCost } from '$lib/utils/formatting';
  import { getStatusColorHex } from '$lib/utils/status';
  import MarkdownRenderer from './MarkdownRenderer.svelte';
  import { VirtualScroll } from "svelte-virtual-scroll-list";
  import HistoricalPromptItem from './HistoricalPromptItem.svelte';
  import { ToolCallList, StateChangeList, DecisionList } from './orchestrator';
  import { HistoricalOutputView } from './history';

  // Active tab
  let activeTab = $state<'overview' | 'activity' | 'outputs' | 'prompts'>('overview');

  // Activity subtab
  let activitySubtab = $state<'tools' | 'states' | 'decisions'>('tools');

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
  let prompts = $state<Array<{ prompt: string; timestamp: number }>>([]);
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
    loadPrompts(run.agent_id);

    // Load activity if pipeline_id is present
    if (run.pipeline_id) {
      loadActivity(run.pipeline_id);
    }

    // Load outputs
    loadOutputs(run.agent_id, run.pipeline_id);
  }

  async function loadPrompts(agentId: string) {
    loadingPrompts = true;
    promptsError = null;
    try {
      const result = await invoke<Array<[string, number]>>("get_run_prompts", { agentId });
      prompts = result.map(([prompt, timestamp]) => ({ prompt, timestamp }));
    } catch (e) {
      console.error("Failed to load run prompts:", e);
      promptsError = "Failed to load conversation history";
    } finally {
      loadingPrompts = false;
    }
  }

  // Convert database record to display type
  function convertToolCallRecord(record: OrchestratorToolCallRecord): OrchestratorToolCall {
    let parsedInput: Record<string, unknown> = {};
    if (record.tool_input) {
      try {
        parsedInput = JSON.parse(record.tool_input);
      } catch { /* ignore parse errors */ }
    }
    return {
      tool_name: record.tool_name,
      tool_input: parsedInput,
      is_error: record.is_error,
      summary: record.summary,
      current_state: record.current_state,
      iteration: record.iteration,
      timestamp: record.timestamp
    };
  }

  function convertStateChangeRecord(record: OrchestratorStateChangeRecord): OrchestratorStateChange {
    return {
      old_state: record.old_state,
      new_state: record.new_state,
      iteration: record.iteration,
      generated_skills: record.generated_skills,
      generated_subagents: record.generated_subagents,
      claudemd_generated: record.claudemd_generated,
      timestamp: record.timestamp
    };
  }

  function convertDecisionRecord(record: OrchestratorDecisionRecord): OrchestratorDecision {
    return {
      pipeline_id: record.pipeline_id,
      decision: record.decision as OrchestratorDecision['decision'],
      reasoning: record.reasoning || '',
      issues: record.issues || [],
      suggestions: record.suggestions || [],
      timestamp: record.timestamp
    };
  }

  async function loadActivity(pipelineId: string) {
    loadingActivity = true;
    activityError = null;
    try {
      // Load all activity data in parallel
      const [toolCallsResult, stateChangesResult, decisionsResult] = await Promise.all([
        invoke<OrchestratorToolCallRecord[]>("get_orchestrator_tool_calls", {
          pipelineId,
          limit: 1000
        }),
        invoke<OrchestratorStateChangeRecord[]>("get_orchestrator_state_changes", {
          pipelineId,
          limit: 500
        }),
        invoke<OrchestratorDecisionRecord[]>("get_orchestrator_decisions", {
          pipelineId,
          limit: 100
        }),
      ]);

      // Convert records to display types
      toolCalls = (toolCallsResult || []).map(convertToolCallRecord);
      stateChanges = (stateChangesResult || []).map(convertStateChangeRecord);
      decisions = (decisionsResult || []).map(convertDecisionRecord);
    } catch (e) {
      console.error("Failed to load activity:", e);
      activityError = "Failed to load orchestrator activity";
    } finally {
      loadingActivity = false;
    }
  }

  async function loadOutputs(agentId: string, pipelineId?: string) {
    loadingOutputs = true;
    outputsError = null;
    try {
      const result = await invoke<AgentOutputRecord[]>("get_agent_output_history", {
        agentId,
        pipelineId,
        limit: 2000
      });
      outputs = result || [];
    } catch (e) {
      console.error("Failed to load outputs:", e);
      outputsError = "Failed to load agent outputs";
    } finally {
      loadingOutputs = false;
    }
  }

  function handleScroll(event: Event) {
    const target = event.target as HTMLDivElement;
    scrollTop = target.scrollTop;
  }

</script>

{#if $selectedHistoricalRun}
  <main class="historical-run-view">
    <header>
      <div class="run-info">
        <div class="run-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
            <path d="M3 3v5h5"/>
            <circle cx="12" cy="12" r="1"/>
          </svg>
        </div>
        <div class="run-details">
          <h2>{$selectedHistoricalRun.working_dir.split("/").pop()}</h2>
          <div class="path-and-status">
            <span class="full-path">{$selectedHistoricalRun.working_dir}</span>
            <span
              class="status-badge"
              style="background-color: {getStatusColorHex($selectedHistoricalRun.status)}"
            >
              {$selectedHistoricalRun.status.toUpperCase()}
            </span>
          </div>
        </div>
      </div>
    </header>

    <div class="stats-summary">
      <div class="stat-card">
        <div class="stat-label">Started</div>
        <div class="stat-value">{formatTimeAbsolute($selectedHistoricalRun.started_at)}</div>
      </div>
      <div class="stat-card">
        <div class="stat-label">Duration</div>
        <div class="stat-value">
          {formatDurationVerbose($selectedHistoricalRun.started_at, $selectedHistoricalRun.ended_at)}
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-label">Prompts</div>
        <div class="stat-value">{$selectedHistoricalRun.total_prompts}</div>
      </div>
      <div class="stat-card">
        <div class="stat-label">Tool Calls</div>
        <div class="stat-value">{$selectedHistoricalRun.total_tool_calls}</div>
      </div>
      <div class="stat-card">
        <div class="stat-label">Output Size</div>
        <div class="stat-value">{formatBytes($selectedHistoricalRun.total_output_bytes)}</div>
      </div>
      {#if $selectedHistoricalRun.total_cost_usd}
        <div class="stat-card">
          <div class="stat-label">Cost</div>
          <div class="stat-value">{formatCost($selectedHistoricalRun.total_cost_usd)}</div>
        </div>
      {/if}
    </div>

    <!-- Tabs -->
    <div class="main-tabs">
      <button
        class="main-tab"
        class:active={activeTab === 'overview'}
        onclick={() => activeTab = 'overview'}
      >
        Overview
      </button>
      <button
        class="main-tab"
        class:active={activeTab === 'activity'}
        onclick={() => activeTab = 'activity'}
      >
        Activity ({toolCalls.length + stateChanges.length + decisions.length})
      </button>
      <button
        class="main-tab"
        class:active={activeTab === 'outputs'}
        onclick={() => activeTab = 'outputs'}
      >
        Outputs ({outputs.length})
      </button>
      <button
        class="main-tab"
        class:active={activeTab === 'prompts'}
        onclick={() => activeTab = 'prompts'}
      >
        Prompts ({prompts.length})
      </button>
    </div>

    <div class="tab-content" onscroll={handleScroll}>
      <!-- Overview Tab -->
      {#if activeTab === 'overview'}
        <div class="overview-content">
          {#if $selectedHistoricalRun.initial_prompt}
            <div class="section">
              <h3>Initial Prompt</h3>
              <div class="prompt-content">
                <MarkdownRenderer content={$selectedHistoricalRun.initial_prompt} />
              </div>
            </div>
          {/if}

          {#if $selectedHistoricalRun.error_message}
            <div class="section">
              <h3>Error</h3>
              <div class="error-content">
                <MarkdownRenderer content={$selectedHistoricalRun.error_message} />
              </div>
            </div>
          {/if}

          {#if $selectedHistoricalRun.can_resume}
            <div class="resume-section">
              <button class="primary resume-btn">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polygon points="5 3 19 12 5 21 5 3"/>
                </svg>
                Resume This Run
              </button>
              <p class="resume-hint">This run can be resumed from where it left off</p>
            </div>
          {/if}

          {#if !$selectedHistoricalRun.initial_prompt && !$selectedHistoricalRun.error_message && !$selectedHistoricalRun.can_resume}
            <div class="empty-state">
              <p>No additional information available for this run.</p>
              <p class="hint">Check the Activity, Outputs, or Prompts tabs for more details.</p>
            </div>
          {/if}
        </div>

      <!-- Activity Tab -->
      {:else if activeTab === 'activity'}
        <div class="activity-content">
          {#if loadingActivity}
            <div class="loading">
              <div class="spinner"></div>
              <p>Loading activity...</p>
            </div>
          {:else if activityError}
            <div class="error-message">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"/>
                <line x1="12" y1="8" x2="12" y2="12"/>
                <line x1="12" y1="16" x2="12.01" y2="16"/>
              </svg>
              <p>{activityError}</p>
            </div>
          {:else if !$selectedHistoricalRun.pipeline_id}
            <div class="empty-state">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M9 17H5a2 2 0 0 0-2 2 2 2 0 0 0 2 2h2a2 2 0 0 0 2-2zm12-2h-4a2 2 0 0 0-2 2 2 2 0 0 0 2 2h2a2 2 0 0 0 2-2z"/>
                <circle cx="9" cy="7" r="4"/>
                <path d="M9 11a4 4 0 0 0 4 4"/>
              </svg>
              <p>No orchestrator activity available</p>
              <p class="hint">This run was not created by an orchestrator pipeline</p>
            </div>
          {:else if toolCalls.length === 0 && stateChanges.length === 0 && decisions.length === 0}
            <div class="empty-state">
              <p>No activity recorded for this pipeline</p>
            </div>
          {:else}
            <div class="activity-subtabs">
              <button
                class="subtab"
                class:active={activitySubtab === 'tools'}
                onclick={() => activitySubtab = 'tools'}
              >
                Tools ({toolCalls.length})
              </button>
              <button
                class="subtab"
                class:active={activitySubtab === 'states'}
                onclick={() => activitySubtab = 'states'}
              >
                States ({stateChanges.length})
              </button>
              <button
                class="subtab"
                class:active={activitySubtab === 'decisions'}
                onclick={() => activitySubtab = 'decisions'}
              >
                Decisions ({decisions.length})
              </button>
            </div>

            <div class="activity-list">
              {#if activitySubtab === 'tools'}
                <ToolCallList {toolCalls} {scrollTop} />
              {:else if activitySubtab === 'states'}
                <StateChangeList {stateChanges} {scrollTop} />
              {:else if activitySubtab === 'decisions'}
                <DecisionList {decisions} {scrollTop} />
              {/if}
            </div>
          {/if}
        </div>

      <!-- Outputs Tab -->
      {:else if activeTab === 'outputs'}
        <div class="outputs-content">
          {#if loadingOutputs}
            <div class="loading">
              <div class="spinner"></div>
              <p>Loading outputs...</p>
            </div>
          {:else if outputsError}
            <div class="error-message">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"/>
                <line x1="12" y1="8" x2="12" y2="12"/>
                <line x1="12" y1="16" x2="12.01" y2="16"/>
              </svg>
              <p>{outputsError}</p>
            </div>
          {:else}
            <HistoricalOutputView {outputs} {scrollTop} />
          {/if}
        </div>

      <!-- Prompts Tab -->
      {:else if activeTab === 'prompts'}
        <div class="prompts-content">
          {#if loadingPrompts}
            <div class="loading">
              <div class="spinner"></div>
              <p>Loading conversation history...</p>
            </div>
          {:else if promptsError}
            <div class="error-message">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"/>
                <line x1="12" y1="8" x2="12" y2="12"/>
                <line x1="12" y1="16" x2="12.01" y2="16"/>
              </svg>
              <p>{promptsError}</p>
            </div>
          {:else if prompts.length === 0}
            <div class="empty-state">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
              </svg>
              <p>No conversation history available</p>
            </div>
          {:else}
            <div class="prompts-list-wrapper">
               <VirtualScroll
                data={prompts}
                key="timestamp"
                let:data
                let:index
               >
                <HistoricalPromptItem {data} {index} />
               </VirtualScroll>
            </div>
          {/if}
        </div>
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

  header {
    padding: var(--space-lg);
    border-bottom: 1px solid var(--border);
    background: linear-gradient(180deg, var(--bg-tertiary) 0%, var(--bg-secondary) 100%);
    flex-shrink: 0;
  }

  .run-info {
    display: flex;
    align-items: center;
    gap: var(--space-md);
  }

  .run-icon {
    width: 48px;
    height: 48px;
    border-radius: 12px;
    background: linear-gradient(135deg, rgba(124, 58, 237, 0.2) 0%, rgba(147, 51, 234, 0.15) 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--accent);
  }

  .run-icon svg {
    width: 28px;
    height: 28px;
    color: var(--accent);
  }

  .run-details {
    flex: 1;
    min-width: 0;
  }

  h2 {
    font-size: 22px;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0 0 8px 0;
  }

  .path-and-status {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .full-path {
    font-size: 13px;
    color: var(--text-muted);
    font-family: 'SF Mono', Menlo, Monaco, Courier, monospace;
  }

  .status-badge {
    padding: 4px 10px;
    border-radius: 12px;
    font-size: 11px;
    font-weight: 600;
    color: white;
    letter-spacing: 0.5px;
  }

  .stats-summary {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(130px, 1fr));
    gap: var(--space-sm);
    padding: var(--space-md);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .stat-card {
    background-color: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: var(--space-sm) var(--space-md);
  }

  .stat-label {
    font-size: 11px;
    color: var(--text-muted);
    font-weight: 500;
    margin-bottom: 4px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .stat-value {
    font-size: 16px;
    font-weight: 700;
    color: var(--text-primary);
  }

  /* Main tabs */
  .main-tabs {
    display: flex;
    border-bottom: 1px solid var(--border);
    background: var(--bg-secondary);
    flex-shrink: 0;
  }

  .main-tab {
    flex: 1;
    padding: 12px 16px;
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--text-secondary);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .main-tab:hover {
    color: var(--text-primary);
    background: var(--bg-tertiary);
  }

  .main-tab.active {
    color: var(--accent);
    border-bottom-color: var(--accent);
  }

  /* Tab content area */
  .tab-content {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  /* Overview tab */
  .overview-content {
    padding: var(--space-md);
  }

  .section {
    margin-bottom: var(--space-lg);
  }

  h3 {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0 0 var(--space-sm) 0;
  }

  .prompt-content,
  .error-content {
    background-color: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: var(--space-md);
    color: var(--text-primary);
  }

  .error-content {
    border-color: var(--error);
    background-color: rgba(239, 68, 68, 0.1);
  }

  .resume-section {
    padding: var(--space-lg);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-sm);
  }

  .resume-btn {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: 12px 24px;
    font-size: 14px;
  }

  .resume-btn svg {
    width: 16px;
    height: 16px;
  }

  .resume-hint {
    font-size: 12px;
    color: var(--text-muted);
    margin: 0;
  }

  /* Activity tab */
  .activity-content {
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .activity-subtabs {
    display: flex;
    padding: var(--space-sm);
    gap: var(--space-sm);
    border-bottom: 1px solid var(--border);
    background: var(--bg-tertiary);
  }

  .subtab {
    padding: 8px 14px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: transparent;
    color: var(--text-secondary);
    font-size: 12px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .subtab:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .subtab.active {
    background: var(--accent);
    border-color: var(--accent);
    color: white;
  }

  .activity-list {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-md);
  }

  /* Outputs tab */
  .outputs-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  /* Prompts tab */
  .prompts-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: var(--space-md);
  }

  .prompts-list-wrapper {
    flex: 1;
    overflow-y: hidden;
    height: 100%;
    min-height: 200px;
  }

  /* Loading and empty states */
  .loading,
  .error-message,
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-xl);
    gap: var(--space-md);
    color: var(--text-muted);
    flex: 1;
    text-align: center;
  }

  .empty-state .hint {
    font-size: 13px;
    opacity: 0.7;
  }

  .error-message svg,
  .empty-state svg {
    width: 48px;
    height: 48px;
    opacity: 0.5;
  }

  .spinner {
    width: 36px;
    height: 36px;
    border: 3px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
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
  .tab-content::-webkit-scrollbar,
  .activity-list::-webkit-scrollbar {
    width: 6px;
  }

  .tab-content::-webkit-scrollbar-track,
  .activity-list::-webkit-scrollbar-track {
    background: transparent;
  }

  .tab-content::-webkit-scrollbar-thumb,
  .activity-list::-webkit-scrollbar-thumb {
    background: var(--border);
    border-radius: 3px;
  }

  .tab-content::-webkit-scrollbar-thumb:hover,
  .activity-list::-webkit-scrollbar-thumb:hover {
    background: var(--accent);
  }
</style>
