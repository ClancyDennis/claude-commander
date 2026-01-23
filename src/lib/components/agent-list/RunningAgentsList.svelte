<script lang="ts">
  import type { Agent, AutoPipeline } from '$lib/types';
  import PipelineListItem from './PipelineListItem.svelte';
  import AgentListItem from './AgentListItem.svelte';

  let {
    agents,
    pipelines,
    selectedAgentId,
    selectedPipelineId,
    viewMode,
    onSelectAgent,
    onMultiSelectAgent,
    onSelectPipeline
  }: {
    agents: Map<string, Agent>;
    pipelines: Map<string, AutoPipeline>;
    selectedAgentId: string | null;
    selectedPipelineId: string | null;
    viewMode: string;
    onSelectAgent: (id: string) => void;
    onMultiSelectAgent?: (id: string) => void;
    onSelectPipeline: (id: string) => void;
  } = $props();

  function isAgentSelected(id: string): boolean {
    return viewMode === 'agent' && selectedAgentId === id && !selectedPipelineId;
  }

  function isPipelineSelected(id: string): boolean {
    return selectedPipelineId === id;
  }
</script>

<!-- Auto Pipelines Section -->
{#if pipelines.size > 0}
  <div class="section-header">
    <span>Pipelines</span>
    <span class="count">{pipelines.size}</span>
  </div>
  <div class="list-section">
    {#each [...pipelines.values()] as pipeline (pipeline.id)}
      <PipelineListItem
        {pipeline}
        isSelected={isPipelineSelected(pipeline.id)}
        onSelect={onSelectPipeline}
      />
    {/each}
  </div>
{/if}

<!-- Worker Agents Section -->
{#if agents.size > 0}
  <div class="section-header">
    <span>Agents</span>
    <span class="count">{agents.size}</span>
  </div>
  <div class="list-section">
    {#each [...agents.values()] as agent (agent.id)}
      <AgentListItem
        {agent}
        isSelected={isAgentSelected(agent.id)}
        onSelect={onSelectAgent}
        onMultiSelect={onMultiSelectAgent}
      />
    {/each}
  </div>
{:else}
  <div class="empty-state">
    <div class="empty-icon">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <rect x="3" y="3" width="18" height="18" rx="2"/>
        <circle cx="12" cy="10" r="3"/>
        <path d="M7 21v-2a4 4 0 0 1 4-4h2a4 4 0 0 1 4 4v2"/>
      </svg>
    </div>
    <p class="empty-title">No agents running</p>
    <p class="empty-hint">Tap "New" to create one</p>
  </div>
{/if}

<style>
  .list-section {
    padding: var(--space-2) var(--space-3);
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-3) var(--space-4) var(--space-2);
    font-size: var(--text-xs);
    font-weight: var(--font-semibold);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .count {
    font-size: var(--text-xs);
    font-weight: var(--font-medium);
    color: var(--text-muted);
    background: var(--bg-tertiary);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
  }

  .list-item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3);
    background: transparent;
    border: none;
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: background var(--transition-fast);
    text-align: left;
    color: inherit;
  }

  .list-item:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .list-item:active {
    background: rgba(255, 255, 255, 0.08);
  }

  .list-item.selected {
    background: rgba(232, 102, 77, 0.12);
  }

  .list-item.selected:hover {
    background: rgba(232, 102, 77, 0.18);
  }

  .item-icon {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-tertiary);
    border-radius: var(--radius-sm);
    flex-shrink: 0;
  }

  .item-content {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .item-title {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .item-subtitle {
    font-size: var(--text-xs);
    color: var(--text-muted);
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .list-item :global(.disclosure) {
    color: var(--text-muted);
    flex-shrink: 0;
    opacity: 0.5;
    transition: opacity var(--transition-fast);
  }

  .list-item:hover :global(.disclosure) {
    opacity: 0.8;
  }

  .list-item.selected :global(.disclosure) {
    color: var(--accent-hex);
    opacity: 1;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-8) var(--space-4);
    text-align: center;
  }

  .empty-icon {
    width: 56px;
    height: 56px;
    border-radius: var(--radius-lg);
    background: var(--bg-tertiary);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: var(--space-4);
  }

  .empty-icon svg {
    width: 28px;
    height: 28px;
    color: var(--text-muted);
  }

  .empty-title {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--text-primary);
    margin-bottom: var(--space-1);
  }

  .empty-hint {
    font-size: var(--text-xs);
    color: var(--text-muted);
  }
</style>
