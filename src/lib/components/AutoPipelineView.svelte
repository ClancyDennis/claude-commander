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
  import { useAsyncData } from '$lib/hooks/useAsyncData.svelte';

  // Import sub-components and utilities
  import { PageLayout } from './ui/layout';
  import { PipelineHeader, StageIndicator, FilterBar, OutputList } from './auto-pipeline';
  import {
    processOutputs,
    countOutputs,
    filterOutputs,
    formatToolDisplay,
    getOrchestratorActiveStage,
    isOrchestratorActiveState
  } from './auto-pipeline/utils';

  let { pipelineId }: { pipelineId: string } = $props();

  let unlistenFns: UnlistenFn[] = [];

  // Track the pipeline ID for fetch closures
  let fetchPipelineId = $state<string | null>(null);

  // Async data for pipeline
  const asyncPipeline = useAsyncData(async () => {
    if (!fetchPipelineId) return null;

    // Check store first
    const storedPipelines = get(autoPipelines);
    const storedPipeline = storedPipelines.get(fetchPipelineId);
    if (storedPipeline) {
      return storedPipeline;
    }

    return await invoke<AutoPipeline>('get_auto_pipeline', { pipelineId: fetchPipelineId });
  });

  // Async data for pipeline history
  const asyncHistory = useAsyncData(async () => {
    if (!fetchPipelineId) return null;
    return await invoke<PipelineHistoryBundle>('get_pipeline_history', { pipelineId: fetchPipelineId });
  });

  // Derived for convenience (maintains backward compatibility in template)
  const pipeline = $derived(asyncPipeline.data);
  const pipelineHistory = $derived(asyncHistory.data);
  const loading = $derived(asyncPipeline.loading);
  const error = $derived(asyncPipeline.error);

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
      asyncPipeline.setData(storedPipeline);
    }
  });

  // Derived values using utility functions
  let allOutputs = $derived(processOutputs(pipeline, pipelineHistory));
  let filteredOutputs = $derived(filterOutputs(allOutputs, stageFilter, typeFilter, searchQuery));
  // Single-pass count calculation for both stage and type counts
  let outputCounts = $derived(countOutputs(allOutputs));
  let stageCounts = $derived(outputCounts.stageCounts);
  let typeCounts = $derived(outputCounts.typeCounts);

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
    fetchPipelineId = id;
    await asyncPipeline.fetch();
    await asyncHistory.fetch();
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
      fetchPipelineId = pipelineId;
      // Fetch fresh data from backend (bypass store check)
      const freshPipeline = await invoke<AutoPipeline>('get_auto_pipeline', { pipelineId });
      asyncPipeline.setData(freshPipeline);
      await asyncHistory.fetch();
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
