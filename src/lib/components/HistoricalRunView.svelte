<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { selectedHistoricalRun } from "../stores/agents";
  import type { AgentRun } from "../types";
  import { onMount } from "svelte";
  import { formatTimeAbsolute, formatDurationVerbose, formatBytes, formatCost } from '$lib/utils/formatting';
  import { getStatusColorHex } from '$lib/utils/status';
  import MarkdownRenderer from './MarkdownRenderer.svelte';
  import { VirtualScroll } from "svelte-virtual-scroll-list";
  import HistoricalPromptItem from './HistoricalPromptItem.svelte';

  let prompts = $state<Array<{ prompt: string; timestamp: number }>>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  // Load prompts when historical run changes
  $effect(() => {
    if ($selectedHistoricalRun) {
      loadRunPrompts($selectedHistoricalRun.agent_id);
    }
  });

  async function loadRunPrompts(agentId: string) {
    loading = true;
    error = null;
    try {
      const result = await invoke<Array<[string, number]>>("get_run_prompts", { agentId });
      prompts = result.map(([prompt, timestamp]) => ({ prompt, timestamp }));
    } catch (e) {
      console.error("Failed to load run prompts:", e);
      error = "Failed to load conversation history";
    } finally {
      loading = false;
    }
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

    <div class="scrollable-content">
      {#if $selectedHistoricalRun.initial_prompt}
        <div class="initial-prompt-section">
          <h3>Initial Prompt</h3>
          <div class="prompt-content">
            <MarkdownRenderer content={$selectedHistoricalRun.initial_prompt} />
          </div>
        </div>
      {/if}

      {#if $selectedHistoricalRun.error_message}
        <div class="error-section">
          <h3>Error</h3>
          <div class="error-content">
            <MarkdownRenderer content={$selectedHistoricalRun.error_message} />
          </div>
        </div>
      {/if}

      <div class="conversation-section">
        <h3>Conversation History ({prompts.length} messages)</h3>

        {#if loading}
          <div class="loading">
            <div class="spinner"></div>
            <p>Loading conversation history...</p>
          </div>
        {:else if error}
          <div class="error-message">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"/>
              <line x1="12" y1="8" x2="12" y2="12"/>
              <line x1="12" y1="16" x2="12.01" y2="16"/>
            </svg>
            <p>{error}</p>
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
    overflow: hidden; /* Changed from auto to hidden to let inner parts scroll */
  }

  header {
    padding: var(--space-lg);
    border-bottom: 1px solid var(--border);
    background: linear-gradient(180deg, var(--bg-tertiary) 0%, var(--bg-secondary) 100%);
    flex-shrink: 0;
  }

  .scrollable-content {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
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
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: var(--space-md);
    padding: var(--space-lg);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .stat-card {
    background-color: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: var(--space-md);
  }

  .stat-label {
    font-size: 12px;
    color: var(--text-muted);
    font-weight: 500;
    margin-bottom: 6px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .stat-value {
    font-size: 18px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .initial-prompt-section,
  .error-section {
    padding: var(--space-lg);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  
  .conversation-section {
    padding: var(--space-lg);
    border-bottom: 1px solid var(--border);
    flex: 1; /* Let this grow */
    display: flex;
    flex-direction: column;
    min-height: 400px; /* Ensure some height for virtual scroll */
  }

  h3 {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0 0 var(--space-md) 0;
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
  }

  .error-message svg,
  .empty-state svg {
    width: 48px;
    height: 48px;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .prompts-list-wrapper {
    flex: 1;
    overflow-y: hidden; /* VirtualScroll handles scroll internally, but we need to give it height */
    /* Wait, VirtualScroll usually takes height of container. 
       If we want the whole page to scroll, we put it in window mode. 
       But here we have a split layout. 
       Ideally we want the conversation section to take remaining space and scroll internally 
       OR we let the whole scrollable-content scroll. 
       VirtualScroll works best with a fixed height container OR page scroll.
       Given the structure, I'll set a min-height and let it fill.
    */
    height: 100%;
    min-height: 200px;
  }

  .resume-section {
    padding: var(--space-lg);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-sm);
    flex-shrink: 0;
  }

  .resume-btn {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: 14px 28px;
    font-size: 15px;
  }

  .resume-btn svg {
    width: 18px;
    height: 18px;
  }

  .resume-hint {
    font-size: 13px;
    color: var(--text-muted);
    margin: 0;
  }

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
  }

  .empty-view p {
    font-size: 16px;
  }
</style>
