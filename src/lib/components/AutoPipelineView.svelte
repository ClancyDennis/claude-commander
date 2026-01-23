<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { get } from 'svelte/store';
  import type { AutoPipeline, PipelineHistoryBundle } from '../types';
  import {
    orchestratorToolCalls,
    orchestratorCurrentState
  } from '../stores/agents';
  import { autoPipelines } from '../stores/autoPipelines';

  // Import sub-components and utilities
  import { PageLayout } from './ui/layout';
  import { PipelineHeader, StageIndicator, FilterBar, OutputList } from './auto-pipeline';
  import {
    processOutputs,
    countByStage,
    countByType,
    filterOutputs,
    formatToolDisplay,
    getOrchestratorActiveStage,
    isOrchestratorActiveState
  } from './auto-pipeline/utils';

  let { pipelineId }: { pipelineId: string } = $props();

  let pipeline: AutoPipeline | null = $state(null);
  let pipelineHistory: PipelineHistoryBundle | null = $state(null);
  let loading = $state(true);
  let error: string | null = $state(null);
  let unlistenFns: UnlistenFn[] = [];

  // Filter state
  let stageFilter = $state<'all' | 'Orchestrator' | 'Planning' | 'Building' | 'Verifying'>('all');
  let typeFilter = $state<'all' | 'text' | 'tool_use' | 'tool_result' | 'error' | 'orchestrator_tool' | 'state_change' | 'decision'>('all');
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

  // Derived values using utility functions
  let allOutputs = $derived(processOutputs(pipeline, pipelineHistory));
  let filteredOutputs = $derived(filterOutputs(allOutputs, stageFilter, typeFilter, searchQuery));
  let stageCounts = $derived(countByStage(allOutputs));
  let typeCounts = $derived(countByType(allOutputs));

  // Orchestrator status derived values
  let totalToolCount = $derived($orchestratorToolCalls.length);
  let currentTool = $derived($orchestratorToolCalls.length > 0 ? $orchestratorToolCalls[$orchestratorToolCalls.length - 1] : null);
  let currentToolDisplay = $derived(formatToolDisplay(currentTool));
  let isOrchestratorActive = $derived(isOrchestratorActiveState($orchestratorCurrentState));
  let orchestratorActiveStage = $derived(getOrchestratorActiveStage($orchestratorCurrentState));

  let isToolExecuting = $derived.by(() => {
    if ($orchestratorToolCalls.length === 0) return false;
    const lastTool = $orchestratorToolCalls[$orchestratorToolCalls.length - 1];
    return !lastTool.completed && isOrchestratorActive;
  });

  let currentToolStatus = $derived.by(() => {
    if (!currentTool) return 'idle' as const;
    if (!currentTool.completed) return 'executing' as const;
    if (currentTool.is_error) return 'error' as const;
    return 'completed' as const;
  });

  async function loadPipeline(id: string) {
    error = null;
    const storedPipelines = get(autoPipelines);
    const storedPipeline = storedPipelines.get(id);

    if (storedPipeline) {
      pipeline = storedPipeline;
      loading = false;
      await loadPipelineHistory(id);
      return;
    }

    loading = true;
    pipeline = null;

    try {
      pipeline = await invoke<AutoPipeline>('get_auto_pipeline', { pipelineId: id });
      await loadPipelineHistory(id);
      loading = false;
    } catch (e) {
      error = String(e);
      loading = false;
    }
  }

  async function loadPipelineHistory(id: string) {
    try {
      pipelineHistory = await invoke<PipelineHistoryBundle>('get_pipeline_history', { pipelineId: id });
    } catch (e) {
      console.error('[AutoPipelineView] Failed to load pipeline history:', e);
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
      await loadPipelineHistory(pipelineId);
    } catch (e) {
      console.error('Failed to refresh pipeline:', e);
    }
  }

  function clearFilters() {
    stageFilter = 'all';
    typeFilter = 'all';
    searchQuery = '';
  }
</script>

{#if loading}
  <PageLayout variant="centered">
    <div class="loading-content">
      <div class="loading-spinner"></div>
      <span>Loading pipeline...</span>
    </div>
  </PageLayout>
{:else if error}
  <PageLayout variant="centered">
    <div class="error-box">
      <h3>Error loading pipeline</h3>
      <p>{error}</p>
    </div>
  </PageLayout>
{:else if pipeline}
  <PageLayout>
    <PipelineHeader {pipeline} />

    <StageIndicator
      steps={pipeline.steps}
      orchestratorCurrentState={$orchestratorCurrentState}
      orchestratorToolCalls={$orchestratorToolCalls}
      {isOrchestratorActive}
      {totalToolCount}
      {currentTool}
      {currentToolDisplay}
      {isToolExecuting}
      {currentToolStatus}
      {orchestratorActiveStage}
    />

    <FilterBar
      bind:stageFilter
      bind:typeFilter
      bind:searchQuery
      {stageCounts}
      {typeCounts}
      totalOutputs={allOutputs.length}
      filteredCount={filteredOutputs.length}
    />

    <OutputList
      {filteredOutputs}
      totalOutputs={allOutputs.length}
      onClearFilters={clearFilters}
    />
  </PageLayout>
{/if}

<style>
  .loading-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    color: var(--text-muted);
  }

  .loading-spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--border-hex);
    border-top-color: var(--accent-hex);
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

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
