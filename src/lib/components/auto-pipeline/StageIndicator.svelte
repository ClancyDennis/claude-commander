<script lang="ts">
  import type { AutoPipelineStep } from '$lib/types';
  import OrchestratorStatus from './OrchestratorStatus.svelte';
  import StageRow from './StageRow.svelte';

  type ToolCall = {
    tool_name?: string;
    tool_input?: Record<string, unknown> | string;
    completed?: boolean;
    is_error?: boolean;
  };

  let {
    steps,
    orchestratorCurrentState,
    orchestratorToolCalls,
    isOrchestratorActive,
    totalToolCount,
    currentTool,
    currentToolDisplay,
    isToolExecuting,
    currentToolStatus,
    orchestratorActiveStage
  }: {
    steps: AutoPipelineStep[];
    orchestratorCurrentState: string;
    orchestratorToolCalls: ToolCall[];
    isOrchestratorActive: boolean;
    totalToolCount: number;
    currentTool: ToolCall | null;
    currentToolDisplay: string;
    isToolExecuting: boolean;
    currentToolStatus: 'idle' | 'executing' | 'completed' | 'error';
    orchestratorActiveStage: number | null;
  } = $props();
</script>

<div class="stage-indicator">
  <!-- Orchestrator Status (Left) -->
  <OrchestratorStatus
    {orchestratorCurrentState}
    {isOrchestratorActive}
    {totalToolCount}
    {currentTool}
    {currentToolDisplay}
    {isToolExecuting}
    {currentToolStatus}
  />

  <!-- Vertical Separator -->
  <div class="stage-separator"></div>

  <!-- Stages (Right, compressed) -->
  <StageRow {steps} {orchestratorActiveStage} />
</div>

<style>
  .stage-indicator {
    display: flex;
    align-items: center;
    padding: var(--space-md) var(--space-lg);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    gap: var(--space-lg);
  }

  /* Vertical Separator */
  .stage-separator {
    width: 1px;
    height: 52px;
    background: var(--border);
    flex-shrink: 0;
  }
</style>
