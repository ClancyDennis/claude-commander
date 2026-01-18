<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { get } from 'svelte/store';
  import type { AutoPipeline, AgentOutputEvent, OrchestratorStateChange, OrchestratorDecision, OrchestratorToolCall } from '../types';
  import {
    orchestratorStateChanges,
    orchestratorDecisions,
    orchestratorToolCalls,
    stepToolCounts
  } from '../stores/agents';
  import { autoPipelines, selectAutoPipeline } from '../stores/autoPipelines';

  let { pipelineId }: { pipelineId: string } = $props();

  let pipeline: AutoPipeline | null = $state(null);
  let loading = $state(true);
  let error: string | null = $state(null);
  let unlistenFns: UnlistenFn[] = [];

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

  async function loadPipeline(id: string) {
    error = null;

    // First check if pipeline is already in the store (from auto_pipeline:started event)
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
    // Listen for pipeline updates
    const unlistenStep = await listen('auto_pipeline:step_completed', (event: any) => {
      if (event.payload.pipeline_id === pipelineId) {
        refreshPipeline();
      }
    });
    unlistenFns.push(unlistenStep);

    // Also listen for final completion
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

  type TimelineEvent = 
    | (OrchestratorStateChange & { type: 'state' })
    | (OrchestratorDecision & { type: 'decision' })
    | (OrchestratorToolCall & { type: 'tool' });

  // Combine events for timeline
  // We want to interleave State Changes, Decisions, and Tool Calls
  function getTimelineEvents(): TimelineEvent[] {
    const changes = $orchestratorStateChanges.map(e => ({ ...e, type: 'state' as const }));
    const decisions = $orchestratorDecisions.map(e => ({ ...e, type: 'decision' as const }));
    const tools = $orchestratorToolCalls.map(e => ({ ...e, type: 'tool' as const }));
    
    return [...changes, ...decisions, ...tools].sort((a, b) => a.timestamp - b.timestamp);
  }

  function formatTime(timestamp: number): string {
    return new Date(timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
  }
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
    <!-- Header (Matching AgentView) -->
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
      <!-- Actions could go here if needed -->
    </header>

    <div class="main-content-scroll">
      <div class="content-container">
        
        <!-- Timeline Section -->
        <section class="timeline-section">
          <h3 class="section-heading">Execution Timeline</h3>
          <div class="timeline">
            {#if getTimelineEvents().length === 0}
              <div class="empty-timeline">Pipeline initializing...</div>
            {/if}
            
            {#each getTimelineEvents() as event}
              {#if event.type === 'state'}
                <div class="timeline-item state-change">
                  <div class="timeline-marker"></div>
                  <div class="timeline-content">
                    <div class="event-time">{formatTime(event.timestamp)}</div>
                    <div class="event-title">
                      State changed to <span class="highlight">{(event as any).new_state}</span>
                    </div>
                    {#if (event as any).old_state}
                      <div class="event-detail">from {(event as any).old_state}</div>
                    {/if}
                  </div>
                </div>
              {:else if event.type === 'decision'}
                <div class="timeline-item decision {(event as any).decision === 'Complete' ? 'success' : (event as any).decision === 'GiveUp' || (event as any).decision === 'Replan' ? 'warning' : 'info'}">
                  <div class="timeline-marker icon">
                    {#if (event as any).decision === 'Complete'}🎉{:else if (event as any).decision === 'Replan'}🔄{:else}⚠️{/if}
                  </div>
                  <div class="timeline-content">
                    <div class="event-time">{formatTime(event.timestamp)}</div>
                    <div class="event-title">Decision: {(event as any).decision}</div>
                    <div class="decision-reasoning">{(event as any).reasoning}</div>
                    {#if (event as any).issues && (event as any).issues.length > 0}
                      <div class="issues-list">
                        <strong>Issues:</strong>
                        <ul>
                          {#each (event as any).issues as issue}
                            <li>{issue}</li>
                          {/each}
                        </ul>
                      </div>
                    {/if}
                  </div>
                </div>
              {:else if event.type === 'tool'}
                <div class="timeline-item tool">
                  <div class="timeline-marker icon">🛠️</div>
                  <div class="timeline-content">
                    <div class="event-time">{formatTime(event.timestamp)}</div>
                    <div class="event-title">Tool: {(event as any).tool_name}</div>
                    {#if (event as any).summary}
                      <div class="event-detail">{(event as any).summary}</div>
                    {/if}
                    {#if (event as any).tool_input}
                      <details class="mt-2 text-xs">
                        <summary class="cursor-pointer text-blue-500 hover:underline">View Input</summary>
                        <pre class="mt-1 p-2 bg-gray-900 rounded overflow-x-auto text-gray-300">
                          {JSON.stringify((event as any).tool_input, null, 2)}
                        </pre>
                      </details>
                    {/if}
                  </div>
                </div>
              {/if}
            {/each}
          </div>
        </section>

        <!-- Current Output Section -->
        <section class="steps-section">
          <h3 class="section-heading">Current Iteration Output</h3>
          <div class="steps-list">
            {#each pipeline.steps as step, i}
              <div class="step-card {step.status.toLowerCase()}">
                <div class="step-card-header">
                  <div class="step-info">
                    <span class="role-icon">{getRoleIcon(step.role)}</span>
                    <div class="step-details">
                      <h4 class="step-title">{step.role}</h4>
                      <div class="step-meta">
                        <span class="status-text {getStatusClass(step.status)}">{step.status}</span>
                        {#if ($stepToolCounts.get(i + 1) || 0) > 0 || step.status === 'Running'}
                          <span class="tool-count">
                            {$stepToolCounts.get(i + 1) || 0} tools
                          </span>
                        {/if}
                      </div>
                    </div>
                  </div>
                </div>

                {#if step.output}
                  <div class="step-output">
                    <div class="output-container">
                      {#if step.output.agent_outputs && step.output.agent_outputs.length > 0}
                        <div class="agent-outputs">
                          {#each step.output.agent_outputs as output}
                            <div class="output-item {output.output_type} animate-slide-up">
                              <div class="output-header">
                                <span class="output-type-badge">{output.output_type}</span>
                              </div>
                              <pre class="output-content">{output.content}</pre>
                            </div>
                          {/each}
                        </div>
                      {:else if step.output.structured_data}
                        <pre class="code-block">{JSON.stringify(step.output.structured_data, null, 2)}</pre>
                      {:else}
                        <pre class="code-block">{step.output.raw_text}</pre>
                      {/if}
                    </div>
                  </div>
                {:else if step.status === 'Running'}
                  <div class="running-placeholder">
                    <div class="mini-spinner"></div>
                    Agent is working...
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        </section>

      </div>
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

  /* Header - Matches AgentView */
  .view-header {
    padding: var(--space-lg);
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--border);
    background: linear-gradient(180deg, var(--bg-secondary) 0%, var(--bg-primary) 100%);
    gap: var(--space-md);
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
    font-size: 20px;
    font-weight: 700;
    margin-bottom: 2px;
    color: var(--text-primary);
  }

  .path-display {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .full-path {
    font-size: 13px;
    color: var(--text-muted);
    display: block;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .status-badge {
    font-size: 11px;
    font-weight: 600;
    padding: 4px 12px;
    border-radius: 6px;
    border: 1px solid transparent;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .status-completed { color: #fff; background: #22c55e; box-shadow: 0 0 10px rgba(34, 197, 94, 0.4); }
  .status-running { color: #fff; background: #3b82f6; box-shadow: 0 0 10px rgba(59, 130, 246, 0.4); }
  .status-failed { color: #fff; background: #ef4444; box-shadow: 0 0 10px rgba(239, 68, 68, 0.4); }
  .status-default { color: var(--text-muted); background: var(--bg-tertiary); border-color: var(--border); }

  /* Main Content Scroll */
  .main-content-scroll {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
  }

  .content-container {
    max-width: 800px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 32px;
  }

  .section-heading {
    font-size: 14px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
    margin-bottom: 16px;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--border);
  }

  /* Timeline */
  .timeline {
    position: relative;
    padding-left: 24px;
    border-left: 2px solid var(--border);
  }

  .timeline-item {
    position: relative;
    margin-bottom: 24px;
  }

  .timeline-marker {
    position: absolute;
    left: -31px;
    top: 4px;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--bg-secondary);
    border: 2px solid var(--text-muted);
  }

  .timeline-item.state-change .timeline-marker {
    border-color: #3b82f6;
    background: var(--bg-primary);
  }

  .timeline-marker.icon {
    width: 24px;
    height: 24px;
    left: -37px;
    top: -2px;
    border: none;
    background: transparent;
    font-size: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .timeline-content {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 12px 16px;
  }

  .timeline-item.decision.warning .timeline-content {
    border-left: 3px solid #eab308;
  }
  .timeline-item.decision.success .timeline-content {
    border-left: 3px solid #22c55e;
  }
  .timeline-item.decision.info .timeline-content {
    border-left: 3px solid #3b82f6;
  }

  /* Tool items */
  .timeline-item.tool .timeline-content {
    border-left: 3px solid var(--text-muted);
  }

  .event-time {
    font-size: 11px;
    color: var(--text-muted);
    margin-bottom: 4px;
  }

  .event-title {
    font-weight: 600;
    font-size: 14px;
    color: var(--text-primary);
  }

  .highlight { color: #3b82f6; }

  .event-detail {
    font-size: 13px;
    color: var(--text-secondary);
    margin-top: 4px;
  }

  .decision-reasoning {
    margin-top: 8px;
    font-size: 13px;
    color: var(--text-primary);
    line-height: 1.5;
  }

  .issues-list {
    margin-top: 12px;
    padding-top: 12px;
    border-top: 1px solid var(--border);
    font-size: 13px;
  }
  .issues-list ul {
    margin: 4px 0 0 20px;
    padding: 0;
    list-style: disc;
    color: var(--text-secondary);
  }

  .empty-timeline {
    color: var(--text-muted);
    font-style: italic;
    padding: 12px;
  }

  /* Steps Section */
  .steps-list {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .step-card {
    background: var(--bg-elevated);
    border-radius: 12px;
    border: 1px solid var(--border);
    overflow: hidden;
  }

  .step-card.running {
    border-color: #3b82f6;
    box-shadow: 0 0 0 1px #3b82f6;
  }

  .step-card-header {
    padding: 16px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--bg-secondary);
    background: var(--bg-secondary);
  }

  .step-info { display: flex; align-items: center; gap: 12px; }
  
  .role-icon {
    font-size: 24px;
    background: var(--bg-primary);
    width: 40px;
    height: 40px;
    border-radius: 10px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--border);
  }

  .step-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0 0 2px 0;
  }

  .step-meta {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .status-text {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
  }
  .status-text.status-completed { color: #22c55e; }
  .status-text.status-running { color: #3b82f6; }
  .status-text.status-failed { color: #ef4444; }
  .status-text.status-default { color: var(--text-muted); }

  .tool-count {
    font-size: 11px;
    font-weight: 500;
    color: var(--text-secondary);
    background: var(--bg-primary);
    padding: 2px 8px;
    border-radius: 4px;
    border: 1px solid var(--border);
  }

  .step-output {
    padding: 16px;
    background: var(--bg-primary);
  }

  .running-placeholder {
    padding: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--text-secondary);
    font-style: italic;
  }

  .mini-spinner {
    width: 16px;
    height: 16px;
    border: 2px solid var(--text-secondary);
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  /* Reused Output Styles */
  .agent-outputs { display: flex; flex-direction: column; gap: 8px; }
  
  .output-item {
    padding: 12px;
    border-radius: 8px;
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
  }
  .output-item.error { background-color: rgba(239, 68, 68, 0.1); border-color: rgba(239, 68, 68, 0.2); }
  .output-item.tool_use { background-color: rgba(234, 179, 8, 0.1); border-color: rgba(234, 179, 8, 0.2); }

  .output-header { margin-bottom: 6px; }
  .output-type-badge {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--text-secondary);
    background-color: var(--bg-primary);
    padding: 2px 6px;
    border-radius: 4px;
    border: 1px solid var(--border);
  }
  .output-item.error .output-type-badge { color: #ef4444; background-color: rgba(239, 68, 68, 0.2); border-color: rgba(239, 68, 68, 0.3); }
  .output-item.tool_use .output-type-badge { color: #eab308; background-color: rgba(234, 179, 8, 0.2); border-color: rgba(234, 179, 8, 0.3); }
  
  .output-content {
    font-size: 12px;
    white-space: pre-wrap;
    overflow-x: auto;
    margin: 0;
    color: var(--text-primary);
    font-family: monospace;
  }

  .code-block {
    background: #0f0f13;
    color: #e0e0e0;
    padding: 12px;
    border-radius: 8px;
    font-size: 12px;
    overflow-x: auto;
    margin: 0;
    border: 1px solid var(--border);
    font-family: monospace;
  }
</style>
