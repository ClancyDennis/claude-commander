<script lang="ts">
  import { getOutputTypeLabel, getOutputTypeClass } from '$lib/utils/outputTypes';
  import type { UnifiedOutput } from './types';

  let {
    filteredOutputs,
    totalOutputs,
    onClearFilters
  }: {
    filteredOutputs: UnifiedOutput[];
    totalOutputs: number;
    onClearFilters: () => void;
  } = $props();

  function getTypeLabel(type: string): string {
    // Use utility if available, fall back to local implementation
    try {
      return getOutputTypeLabel(type);
    } catch {
      switch (type) {
        case 'text': return 'Text';
        case 'tool_use': return 'Tool';
        case 'tool_result': return 'Result';
        case 'error': return 'Error';
        case 'orchestrator_tool': return 'Orchestrator Tool';
        case 'state_change': return 'State Change';
        case 'decision': return 'Decision';
        default: return type;
      }
    }
  }
</script>

<div class="output-list">
  {#if filteredOutputs.length === 0}
    <div class="empty-state">
      {#if totalOutputs === 0}
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
        <button class="secondary small" onclick={onClearFilters}>Clear Filters</button>
      {/if}
    </div>
  {:else}
    {#each filteredOutputs as output, i (i)}
      <div class="output-item {output.output_type}">
        <div class="output-badges">
          <span class="stage-badge {output.stage.toLowerCase()}">{output.stage}</span>
          <span class="type-badge {output.output_type}">{getTypeLabel(output.output_type)}</span>
          {#if output.summary}
            <span class="summary-badge">{output.summary}</span>
          {/if}
        </div>
        <pre class="output-content">{output.content}</pre>
      </div>
    {/each}
  {/if}
</div>

<style>
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

  .output-item.orchestrator_tool {
    background: rgba(240, 112, 90, 0.05);
    border-color: rgba(240, 112, 90, 0.2);
  }

  .output-item.state_change {
    background: rgba(59, 130, 246, 0.05);
    border-color: rgba(59, 130, 246, 0.2);
  }

  .output-item.decision {
    background: rgba(34, 197, 94, 0.05);
    border-color: rgba(34, 197, 94, 0.2);
  }

  .output-badges {
    display: flex;
    gap: var(--space-xs);
    margin-bottom: var(--space-xs);
    flex-wrap: wrap;
  }

  .stage-badge, .type-badge, .summary-badge {
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

  .stage-badge.orchestrator { background: rgba(240, 112, 90, 0.2); color: #f0705a; }
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
  .type-badge.orchestrator_tool { color: #f0705a; border-color: rgba(240, 112, 90, 0.3); }
  .type-badge.state_change { color: #3b82f6; border-color: rgba(59, 130, 246, 0.3); }
  .type-badge.decision { color: #22c55e; border-color: rgba(34, 197, 94, 0.3); }

  .summary-badge {
    background: var(--bg-primary);
    color: var(--text-secondary);
    border: 1px solid var(--border);
    font-style: italic;
  }

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
</style>
