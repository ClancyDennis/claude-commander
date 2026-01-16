<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import type { AutoPipeline } from '$lib/types';

  export let pipelineId: string;

  let pipeline: AutoPipeline | null = null;
  let loading = true;
  let error: string | null = null;
  let unlisten: UnlistenFn | null = null;

  onMount(async () => {
    try {
      pipeline = await invoke<AutoPipeline>('get_auto_pipeline', { pipelineId });
      loading = false;

      // Listen for pipeline updates
      unlisten = await listen('auto_pipeline:step_completed', (event: any) => {
        if (event.payload.pipeline_id === pipelineId) {
          refreshPipeline();
        }
      });

      // Also listen for final completion
      const unlistenComplete = await listen('auto_pipeline:completed', (event: any) => {
        if (event.payload.pipeline_id === pipelineId) {
          refreshPipeline();
        }
      });

    } catch (e) {
      error = String(e);
      loading = false;
    }
  });

  onDestroy(() => {
    if (unlisten) {
      unlisten();
    }
  });

  async function refreshPipeline() {
    try {
      pipeline = await invoke<AutoPipeline>('get_auto_pipeline', { pipelineId });
    } catch (e) {
      console.error('Failed to refresh pipeline:', e);
    }
  }

  function getStatusColor(status: string): string {
    switch (status) {
      case 'Completed': return 'text-green-600 bg-green-50';
      case 'Running': return 'text-blue-600 bg-blue-50';
      case 'Failed': return 'text-red-600 bg-red-50';
      default: return 'text-gray-600 bg-gray-50';
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
</script>

{#if loading}
  <div class="flex items-center justify-center p-8">
    <div class="text-gray-600">Loading pipeline...</div>
  </div>
{:else if error}
  <div class="p-4 bg-red-50 text-red-700 rounded">
    Error loading pipeline: {error}
  </div>
{:else if pipeline}
  <div class="auto-pipeline-view p-6 space-y-6">
    <!-- Header -->
    <div class="border-b pb-4">
      <h2 class="text-2xl font-bold mb-2">Automated Pipeline</h2>
      <p class="text-gray-700">{pipeline.user_request}</p>
      <div class="flex items-center gap-2 mt-2">
        <span class="text-sm text-gray-500">Status:</span>
        <span class="px-2 py-1 rounded text-sm font-medium {getStatusColor(pipeline.status)}">
          {pipeline.status}
        </span>
      </div>
    </div>

    <!-- Steps -->
    <div class="steps space-y-4">
      {#each pipeline.steps as step}
        <div class="step border rounded-lg p-4 {step.status === 'Running' ? 'border-blue-500 bg-blue-50' : 'border-gray-300'}">
          <div class="step-header flex items-center justify-between mb-3">
            <div class="flex items-center gap-2">
              <span class="text-2xl">{getRoleIcon(step.role)}</span>
              <span class="font-semibold text-lg">Step {step.step_number}: {step.role}</span>
            </div>
            <span class="px-3 py-1 rounded text-sm font-medium {getStatusColor(step.status)}">
              {step.status}
            </span>
          </div>

          {#if step.agent_id}
            <div class="text-sm text-gray-600 mb-2">
              Agent ID: <code class="bg-gray-100 px-1 rounded">{step.agent_id}</code>
            </div>
          {/if}

          {#if step.started_at}
            <div class="text-xs text-gray-500 mb-2">
              Started: {new Date(step.started_at).toLocaleString()}
            </div>
          {/if}

          {#if step.output}
            <div class="mt-3">
              <details class="cursor-pointer">
                <summary class="font-medium text-sm text-gray-700 hover:text-gray-900">
                  View Output
                </summary>
                <div class="mt-2 bg-gray-50 p-3 rounded border border-gray-200">
                  {#if step.output.structured_data}
                    <pre class="text-xs overflow-x-auto whitespace-pre-wrap">{JSON.stringify(step.output.structured_data, null, 2)}</pre>
                  {:else}
                    <pre class="text-xs overflow-x-auto whitespace-pre-wrap">{step.output.raw_text}</pre>
                  {/if}
                </div>
              </details>
            </div>
          {/if}

          {#if step.completed_at}
            <div class="text-xs text-gray-500 mt-2">
              Completed: {new Date(step.completed_at).toLocaleString()}
            </div>
          {/if}
        </div>
      {/each}
    </div>

    <!-- Q&A Section -->
    {#if pipeline.questions.length > 0}
      <div class="qna border-t pt-4">
        <h3 class="text-xl font-semibold mb-3">Automated Q&A</h3>
        <div class="space-y-3">
          {#each pipeline.questions as question, i}
            <div class="qa-pair bg-gray-50 p-3 rounded border border-gray-200">
              <p class="mb-2">
                <strong class="text-blue-700">Q{i + 1}:</strong>
                <span class="text-gray-800">{question}</span>
              </p>
              <p>
                <strong class="text-green-700">A{i + 1}:</strong>
                <span class="text-gray-800">{pipeline.answers[i] || 'Pending...'}</span>
              </p>
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Metadata -->
    <div class="metadata text-xs text-gray-500 border-t pt-4">
      <div>Pipeline ID: {pipeline.id}</div>
      <div>Working Directory: {pipeline.working_dir}</div>
      <div>Created: {new Date(pipeline.created_at).toLocaleString()}</div>
      {#if pipeline.completed_at}
        <div>Completed: {new Date(pipeline.completed_at).toLocaleString()}</div>
      {/if}
    </div>
  </div>
{/if}
