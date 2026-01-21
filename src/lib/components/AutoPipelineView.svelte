<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { get } from 'svelte/store';
  import type { AutoPipeline } from '../types';
  import {
    stepToolCounts,
    orchestratorToolCalls,
    orchestratorCurrentState
  } from '../stores/agents';
  import { autoPipelines } from '../stores/autoPipelines';

  let { pipelineId }: { pipelineId: string } = $props();

  let pipeline: AutoPipeline | null = $state(null);
  let loading = $state(true);
  let error: string | null = $state(null);
  let unlistenFns: UnlistenFn[] = [];

  // Filter state
  let stageFilter = $state<'all' | 'Planning' | 'Building' | 'Verifying'>('all');
  let typeFilter = $state<'all' | 'text' | 'tool_use' | 'tool_result' | 'error'>('all');
  let searchQuery = $state('');

  // Load pipeline when pipelineId changes
  $effect(() => {
    if (pipelineId) {
      loadPipeline(pipelineId);
    }
  });

  // Subscribe to store updates for this pipeline
  $effect(() => {
    const storedPipeline = $autoPipelines.get(pipelineId);
    if (storedPipeline) {
      pipeline = storedPipeline;
      loading = false;
    }
  });

  // Combine all outputs from all steps
  let allOutputs = $derived.by(() => {
    if (!pipeline) return [];
    const outputs: Array<{ stage: string; output_type: string; content: string; timestamp?: number }> = [];
    for (const step of pipeline.steps) {
      if (step.output?.agent_outputs) {
        for (const output of step.output.agent_outputs) {
          outputs.push({ ...output, stage: step.role });
        }
      }
    }
    return outputs;
  });

  // Apply filters
  let filteredOutputs = $derived.by(() => {
    return allOutputs.filter(o => {
      if (stageFilter !== 'all' && o.stage !== stageFilter) return false;
      if (typeFilter !== 'all' && o.output_type !== typeFilter) return false;
      if (searchQuery && !o.content.toLowerCase().includes(searchQuery.toLowerCase())) return false;
      return true;
    });
  });

  // Count outputs per filter for badges
  let stageCounts = $derived.by(() => {
    const counts = { Planning: 0, Building: 0, Verifying: 0 };
    for (const o of allOutputs) {
      if (o.stage in counts) {
        counts[o.stage as keyof typeof counts]++;
      }
    }
    return counts;
  });

  let typeCounts = $derived.by(() => {
    const counts = { text: 0, tool_use: 0, tool_result: 0, error: 0 };
    for (const o of allOutputs) {
      if (o.output_type in counts) {
        counts[o.output_type as keyof typeof counts]++;
      }
    }
    return counts;
  });

  // Orchestrator status derived values
  let totalToolCount = $derived($orchestratorToolCalls.length);

  // Get the most recent tool call (the one currently running or last completed)
  let currentTool = $derived.by(() => {
    const tools = $orchestratorToolCalls;
    if (tools.length === 0) return null;
    return tools[tools.length - 1];
  });

  // Format the current tool for display
  let currentToolDisplay = $derived.by(() => {
    if (!currentTool) return 'Waiting...';
    const name = currentTool.tool_name || 'Unknown';
    // Extract first meaningful argument
    let display = name;
    if (currentTool.tool_input) {
      const input = currentTool.tool_input;
      if (typeof input === 'object') {
        // Try common keys: file_path, path, command, etc.
        const val = (input as Record<string, unknown>).file_path ||
                    (input as Record<string, unknown>).path ||
                    (input as Record<string, unknown>).command ||
                    (input as Record<string, unknown>).pattern ||
                    Object.values(input)[0];
        if (val && typeof val === 'string') {
          // Get just filename if it's a path
          const short = val.split('/').pop()?.slice(0, 25) || val.slice(0, 25);
          display = `${name}(${short}${val.length > 25 ? '...' : ''})`;
        }
      }
    }
    return display;
  });

  // Check if orchestrator is actively working (not idle/completed)
  let isOrchestratorActive = $derived.by(() => {
    const state = $orchestratorCurrentState;
    return state !== 'Idle' && state !== 'Completed' && !state.includes('Failed');
  });

  async function loadPipeline(id: string) {
    error = null;

    // First check if pipeline is already in the store
    const storedPipelines = get(autoPipelines);
    const storedPipeline = storedPipelines.get(id);

    if (storedPipeline) {
      pipeline = storedPipeline;
      loading = false;
      return;
    }

    // Not in store, fetch from backend
    loading = true;
    pipeline = null;

    try {
      pipeline = await invoke<AutoPipeline>('get_auto_pipeline', { pipelineId: id });
      loading = false;
    } catch (e) {
      error = String(e);
      loading = false;
    }
  }

  onMount(async () => {
    const unlistenStep = await listen('auto_pipeline:step_completed', (event: any) => {
      if (event.payload.pipeline_id === pipelineId) {
        refreshPipeline();
      }
    });
    unlistenFns.push(unlistenStep);

    const unlistenComplete = await listen('auto_pipeline:completed', (event: any) => {
      if (event.payload.pipeline_id === pipelineId) {
        refreshPipeline();
      }
    });
    unlistenFns.push(unlistenComplete);
  });

  onDestroy(() => {
    unlistenFns.forEach(fn => fn());
  });

  async function refreshPipeline() {
    try {
      pipeline = await invoke<AutoPipeline>('get_auto_pipeline', { pipelineId });
    } catch (e) {
      console.error('Failed to refresh pipeline:', e);
    }
  }

  function getStatusClass(status: string): string {
    switch (status) {
      case 'Completed': return 'status-completed';
      case 'Running': return 'status-running';
      case 'Failed': return 'status-failed';
      default: return 'status-default';
    }
  }

  function getRoleIcon(role: string): string {
    switch (role) {
      case 'Planning': return '📋';
      case 'Building': return '🔨';
      case 'Verifying': return '✅';
      default: return '⚙️';
    }
  }

  function getTypeLabel(type: string): string {
    switch (type) {
      case 'text': return 'Text';
      case 'tool_use': return 'Tool';
      case 'tool_result': return 'Result';
      case 'error': return 'Error';
      default: return type;
    }
  }

  function clearFilters() {
    stageFilter = 'all';
    typeFilter = 'all';
    searchQuery = '';
  }

  const hasActiveFilters = $derived(stageFilter !== 'all' || typeFilter !== 'all' || searchQuery !== '');
</script>

{#if loading}
  <div class="loading-container">
    <div class="loading-spinner"></div>
    <span>Loading pipeline...</span>
  </div>
{:else if error}
  <div class="error-container">
    <div class="error-box">
      <h3>Error loading pipeline</h3>
      <p>{error}</p>
    </div>
  </div>
{:else if pipeline}
  <div class="auto-pipeline-view">
    <!-- Header -->
    <header class="view-header">
      <div class="pipeline-info">
        <div class="pipeline-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
            <polyline points="22 4 12 14.01 9 11.01"></polyline>
          </svg>
        </div>
        <div class="pipeline-details">
          <h2 class="truncate" title={pipeline.user_request}>{pipeline.user_request}</h2>
          <div class="path-display">
            <span class="full-path">{pipeline.working_dir}</span>
          </div>
        </div>
        <div class="status-badge {getStatusClass(pipeline.status)}">
          {pipeline.status}
        </div>
      </div>
    </header>

    <!-- Stage Indicator with Orchestrator Status -->
    <div class="stage-indicator">
      <!-- Orchestrator Status (Left) -->
      <div class="orchestrator-status" class:active={isOrchestratorActive}>
        <div class="orchestrator-icon">🎛️</div>
        <div class="orchestrator-info">
          <div class="orchestrator-stats">
            <span class="tool-count">{totalToolCount} tools</span>
            <span class="state-label" class:active={isOrchestratorActive}>
              {$orchestratorCurrentState}
            </span>
          </div>
          <div class="current-tool" class:active={isOrchestratorActive}>
            {#if isOrchestratorActive}
              <span class="tool-spinner"></span>
            {:else}
              <span class="tool-icon">⚡</span>
            {/if}
            <span class="tool-name" title={currentToolDisplay}>{currentToolDisplay}</span>
          </div>
        </div>
      </div>

      <!-- Vertical Separator -->
      <div class="stage-separator"></div>

      <!-- Stages (Right, compressed) -->
      <div class="stages-row">
        {#each pipeline.steps as step, i}
          <div class="stage" class:completed={step.status === 'Completed'}
               class:active={step.status === 'Running'} class:pending={step.status !== 'Completed' && step.status !== 'Running'}>
            <div class="stage-icon-wrapper" class:completed={step.status === 'Completed'} class:active={step.status === 'Running'}>
              {#if step.status === 'Completed'}
                <svg class="check-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                  <polyline points="20 6 9 17 4 12"/>
                </svg>
              {:else if step.status === 'Running'}
                <span class="stage-emoji">{getRoleIcon(step.role)}</span>
              {:else}
                <span class="stage-emoji dimmed">{getRoleIcon(step.role)}</span>
              {/if}
            </div>
            <span class="stage-name">{step.role}</span>
          </div>
          {#if i < pipeline.steps.length - 1}
            <div class="stage-connector" class:filled={step.status === 'Completed'}></div>
          {/if}
        {/each}
      </div>
    </div>

    <!-- Filter Bar -->
    <div class="filter-bar">
      <div class="filter-group">
        <label class="filter-label">Stage</label>
        <select bind:value={stageFilter} class="filter-select">
          <option value="all">All ({allOutputs.length})</option>
          <option value="Planning">Planning ({stageCounts.Planning})</option>
          <option value="Building">Building ({stageCounts.Building})</option>
          <option value="Verifying">Verifying ({stageCounts.Verifying})</option>
        </select>
      </div>

      <div class="filter-group">
        <label class="filter-label">Type</label>
        <select bind:value={typeFilter} class="filter-select">
          <option value="all">All Types</option>
          <option value="text">Text ({typeCounts.text})</option>
          <option value="tool_use">Tool Use ({typeCounts.tool_use})</option>
          <option value="tool_result">Result ({typeCounts.tool_result})</option>
          <option value="error">Error ({typeCounts.error})</option>
        </select>
      </div>

      <div class="search-wrapper">
        <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8"/>
          <line x1="21" y1="21" x2="16.65" y2="16.65"/>
        </svg>
        <input
          type="text"
          placeholder="Search outputs..."
          bind:value={searchQuery}
          class="search-input"
        />
        {#if searchQuery}
          <button class="clear-search" onclick={() => searchQuery = ''}>×</button>
        {/if}
      </div>

      {#if hasActiveFilters}
        <button class="clear-filters-btn" onclick={clearFilters}>
          Clear Filters
        </button>
      {/if}

      <span class="results-count">
        {filteredOutputs.length} of {allOutputs.length}
      </span>
    </div>

    <!-- Unified Output List -->
    <div class="output-list">
      {#if filteredOutputs.length === 0}
        <div class="empty-state">
          {#if allOutputs.length === 0}
            <div class="empty-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
              </svg>
            </div>
            <p class="empty-title">Pipeline starting...</p>
            <p class="empty-hint">Outputs will appear here as the agent works</p>
          {:else}
            <div class="empty-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <circle cx="11" cy="11" r="8"/>
                <line x1="21" y1="21" x2="16.65" y2="16.65"/>
              </svg>
            </div>
            <p class="empty-title">No matching outputs</p>
            <p class="empty-hint">Try adjusting your filters</p>
            <button class="secondary small" onclick={clearFilters}>Clear Filters</button>
          {/if}
        </div>
      {:else}
        {#each filteredOutputs as output, i (i)}
          <div class="output-item {output.output_type}">
            <div class="output-badges">
              <span class="stage-badge {output.stage.toLowerCase()}">{output.stage}</span>
              <span class="type-badge {output.output_type}">{getTypeLabel(output.output_type)}</span>
            </div>
            <pre class="output-content">{output.content}</pre>
          </div>
        {/each}
      {/if}
    </div>
  </div>
{/if}

<style>
  .auto-pipeline-view {
    height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--bg-primary);
    color: var(--text-primary);
  }

  .loading-container, .error-container {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
  }

  .loading-spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: 16px;
  }

  .error-box {
    padding: 24px;
    background: var(--error-glow);
    border: 1px solid var(--error);
    border-radius: 12px;
    color: var(--error);
    text-align: center;
  }

  /* Header */
  .view-header {
    padding: var(--space-lg);
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--border);
    background: linear-gradient(180deg, var(--bg-secondary) 0%, var(--bg-primary) 100%);
    flex-shrink: 0;
  }

  .pipeline-info {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    flex: 1;
    min-width: 0;
  }

  .pipeline-icon {
    width: 48px;
    height: 48px;
    border-radius: 14px;
    background: linear-gradient(135deg, var(--accent) 0%, #9333ea 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    box-shadow: 0 4px 12px var(--accent-glow);
  }

  .pipeline-icon svg {
    width: 24px;
    height: 24px;
    color: white;
  }

  .pipeline-details {
    flex: 1;
    min-width: 0;
  }

  .pipeline-details h2 {
    font-size: 18px;
    font-weight: 700;
    margin-bottom: 2px;
    color: var(--text-primary);
  }

  .truncate {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .full-path {
    font-size: 13px;
    color: var(--text-muted);
  }

  .status-badge {
    font-size: 11px;
    font-weight: 600;
    padding: 4px 12px;
    border-radius: 6px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .status-completed { color: #fff; background: #22c55e; box-shadow: 0 0 10px rgba(34, 197, 94, 0.4); }
  .status-running { color: #fff; background: #3b82f6; box-shadow: 0 0 10px rgba(59, 130, 246, 0.4); }
  .status-failed { color: #fff; background: #ef4444; box-shadow: 0 0 10px rgba(239, 68, 68, 0.4); }
  .status-default { color: var(--text-muted); background: var(--bg-tertiary); }

  /* Stage Indicator */
  .stage-indicator {
    display: flex;
    align-items: center;
    padding: var(--space-md) var(--space-lg);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    gap: var(--space-lg);
  }

  /* Orchestrator Status - Left Side */
  .orchestrator-status {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-sm) var(--space-md);
    background: var(--bg-tertiary);
    border-radius: 10px;
    border: 1px solid var(--border);
    flex-shrink: 0;
    min-width: 200px;
    transition: all 0.3s ease;
  }

  .orchestrator-status.active {
    border-color: var(--accent);
    box-shadow: 0 0 12px var(--accent-glow);
  }

  .orchestrator-icon {
    font-size: 28px;
    width: 44px;
    height: 44px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, var(--accent-glow) 0%, rgba(124, 58, 237, 0.2) 100%);
    border-radius: 10px;
  }

  .orchestrator-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
    flex: 1;
  }

  .orchestrator-stats {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .orchestrator-stats .tool-count {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .state-label {
    font-size: 11px;
    font-weight: 700;
    padding: 2px 8px;
    border-radius: 4px;
    background: var(--bg-primary);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.3px;
  }

  .state-label.active {
    background: var(--accent-glow);
    color: var(--accent);
  }

  .current-tool {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    color: var(--text-muted);
  }

  .current-tool.active {
    color: var(--text-primary);
  }

  .tool-icon {
    font-size: 10px;
  }

  .tool-name {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 140px;
    font-family: var(--font-mono, monospace);
    font-size: 11px;
  }

  .tool-spinner {
    width: 10px;
    height: 10px;
    border: 2px solid var(--accent-glow);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    flex-shrink: 0;
  }

  /* Vertical Separator */
  .stage-separator {
    width: 1px;
    height: 52px;
    background: var(--border);
    flex-shrink: 0;
  }

  /* Stages Row - Right Side */
  .stages-row {
    display: flex;
    align-items: center;
    flex: 1;
    justify-content: center;
  }

  .stage {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    min-width: 80px;
  }

  .stage-icon-wrapper {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-tertiary);
    border: 2px solid var(--border);
    transition: all 0.3s ease;
  }

  .stage-icon-wrapper.completed {
    background: var(--success);
    border-color: var(--success);
  }

  .stage-icon-wrapper.active {
    background: var(--accent);
    border-color: var(--accent);
    box-shadow: 0 0 20px var(--accent-glow);
    animation: pulseGlow 2s ease-in-out infinite;
  }

  @keyframes pulseGlow {
    0%, 100% { box-shadow: 0 0 20px var(--accent-glow); }
    50% { box-shadow: 0 0 30px var(--accent-glow), 0 0 40px var(--accent-glow); }
  }

  .stage-emoji {
    font-size: 18px;
  }

  .stage-emoji.dimmed {
    opacity: 0.5;
  }

  .check-icon {
    width: 20px;
    height: 20px;
    color: white;
  }

  .stage-name {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .stage.pending .stage-name {
    color: var(--text-muted);
  }

  .stage-connector {
    width: 40px;
    height: 2px;
    background: var(--border);
    margin: 0 var(--space-xs);
    margin-bottom: 20px; /* Align with icon center */
    position: relative;
  }

  .stage-connector::after {
    content: '';
    position: absolute;
    left: 0;
    top: 0;
    height: 100%;
    width: 0;
    background: linear-gradient(90deg, var(--success), var(--accent));
    transition: width 0.5s ease;
  }

  .stage-connector.filled::after {
    width: 100%;
  }

  /* Filter Bar */
  .filter-bar {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-sm) var(--space-lg);
    background: var(--bg-tertiary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    flex-wrap: wrap;
  }

  .filter-group {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
  }

  .filter-label {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
  }

  .filter-select {
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 6px 10px;
    font-size: 12px;
    color: var(--text-primary);
    cursor: pointer;
  }

  .filter-select:hover {
    border-color: var(--accent);
  }

  .search-wrapper {
    flex: 1;
    min-width: 150px;
    max-width: 300px;
    position: relative;
    display: flex;
    align-items: center;
  }

  .search-icon {
    position: absolute;
    left: 10px;
    width: 16px;
    height: 16px;
    color: var(--text-muted);
    pointer-events: none;
  }

  .search-input {
    width: 100%;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 6px 30px 6px 32px;
    font-size: 12px;
    color: var(--text-primary);
  }

  .search-input:focus {
    outline: none;
    border-color: var(--accent);
  }

  .clear-search {
    position: absolute;
    right: 8px;
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: 16px;
    cursor: pointer;
    padding: 0;
    line-height: 1;
  }

  .clear-search:hover {
    color: var(--text-primary);
  }

  .clear-filters-btn {
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 6px 12px;
    font-size: 12px;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .clear-filters-btn:hover {
    border-color: var(--accent);
    color: var(--accent);
  }

  .results-count {
    font-size: 12px;
    color: var(--text-muted);
    margin-left: auto;
  }

  /* Output List */
  .output-list {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-md);
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .output-item {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: var(--space-sm) var(--space-md);
    animation: slideIn 0.2s ease;
  }

  @keyframes slideIn {
    from { opacity: 0; transform: translateY(8px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .output-item.error {
    background: rgba(239, 68, 68, 0.1);
    border-color: rgba(239, 68, 68, 0.3);
  }

  .output-item.tool_use {
    background: rgba(245, 158, 11, 0.05);
    border-color: rgba(245, 158, 11, 0.2);
  }

  .output-badges {
    display: flex;
    gap: var(--space-xs);
    margin-bottom: var(--space-xs);
  }

  .stage-badge, .type-badge {
    font-size: 10px;
    font-weight: 600;
    padding: 2px 6px;
    border-radius: 4px;
    text-transform: uppercase;
  }

  .stage-badge {
    background: var(--bg-tertiary);
    color: var(--text-secondary);
  }

  .stage-badge.planning { background: rgba(59, 130, 246, 0.2); color: #3b82f6; }
  .stage-badge.building { background: rgba(234, 179, 8, 0.2); color: #eab308; }
  .stage-badge.verifying { background: rgba(34, 197, 94, 0.2); color: #22c55e; }

  .type-badge {
    background: var(--bg-primary);
    color: var(--text-muted);
    border: 1px solid var(--border);
  }

  .type-badge.error { color: #ef4444; border-color: rgba(239, 68, 68, 0.3); }
  .type-badge.tool_use { color: #f59e0b; border-color: rgba(245, 158, 11, 0.3); }
  .type-badge.tool_result { color: #22c55e; border-color: rgba(34, 197, 94, 0.3); }

  .output-content {
    font-size: 12px;
    font-family: monospace;
    white-space: pre-wrap;
    word-break: break-word;
    margin: 0;
    color: var(--text-primary);
    line-height: 1.5;
    max-height: 200px;
    overflow-y: auto;
  }

  /* Empty State */
  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-xl);
    text-align: center;
  }

  .empty-icon {
    width: 64px;
    height: 64px;
    border-radius: 16px;
    background: linear-gradient(135deg, var(--bg-secondary) 0%, var(--bg-tertiary) 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: var(--space-md);
    border: 1px solid var(--border);
  }

  .empty-icon svg {
    width: 32px;
    height: 32px;
    color: var(--text-muted);
  }

  .empty-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0 0 var(--space-xs);
  }

  .empty-hint {
    font-size: 13px;
    color: var(--text-muted);
    margin: 0 0 var(--space-md);
  }

  button.secondary.small {
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 8px 16px;
    font-size: 12px;
    color: var(--text-secondary);
    cursor: pointer;
  }

  button.secondary.small:hover {
    border-color: var(--accent);
    color: var(--accent);
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
