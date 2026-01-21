<script lang="ts">
  import type { Agent, AutoPipeline } from '$lib/types';
  import PipelineListItem from './PipelineListItem.svelte';
  import AgentListItem from './AgentListItem.svelte';
  import HelpTip from "../new-agent/HelpTip.svelte";

  let {
    agents,
    pipelines,
    selectedAgentId,
    selectedPipelineId,
    viewMode,
    onOpenChat,
    onSelectAgent,
    onMultiSelectAgent,
    onSelectPipeline
  }: {
    agents: Map<string, Agent>;
    pipelines: Map<string, AutoPipeline>;
    selectedAgentId: string | null;
    selectedPipelineId: string | null;
    viewMode: string;
    onOpenChat: () => void;
    onSelectAgent: (id: string) => void;
    onMultiSelectAgent?: (id: string) => void;
    onSelectPipeline: (id: string) => void;
  } = $props();

  function isChatSelected(): boolean {
    return viewMode === 'chat' && !selectedPipelineId;
  }

  function isAgentSelected(id: string): boolean {
    return viewMode === 'agent' && selectedAgentId === id && !selectedPipelineId;
  }

  function isPipelineSelected(id: string): boolean {
    return selectedPipelineId === id;
  }
</script>

<!-- Chat Assistant entry -->
<ul>
  <li>
    <button
      class="agent-btn chat-assistant"
      class:selected={isChatSelected()}
      onclick={() => onOpenChat()}
    >
      <div class="chat-icon">🎯</div>
      <div class="info">
        <div class="name-row">
          <span class="name">System Commander</span>
        </div>
        <div class="meta-row">
          <span class="path">Mission control for Claude <HelpTip text="Chat interface to control agents via natural language. Ask questions or give commands." placement="right" /></span>
        </div>
      </div>
      <svg class="chevron" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="9,6 15,12 9,18"/>
      </svg>
    </button>
  </li>
</ul>

<!-- Auto Pipelines Section -->
{#if pipelines.size > 0}
  <div class="separator">
    <span>Auto Pipelines ({pipelines.size})</span>
  </div>
  <ul>
    {#each [...pipelines.values()] as pipeline (pipeline.id)}
      <PipelineListItem
        {pipeline}
        isSelected={isPipelineSelected(pipeline.id)}
        onSelect={onSelectPipeline}
      />
    {/each}
  </ul>
{/if}

<!-- Worker Agents Section -->
{#if agents.size > 0}
  <div class="separator">
    <span>Worker Agents</span>
  </div>
{/if}

{#if agents.size === 0}
  <div class="empty">
    <div class="empty-icon">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <rect x="3" y="3" width="18" height="18" rx="2"/>
        <circle cx="12" cy="10" r="3"/>
        <path d="M7 21v-2a4 4 0 0 1 4-4h2a4 4 0 0 1 4 4v2"/>
      </svg>
    </div>
    <p class="empty-title">No worker agents</p>
    <p class="empty-hint">Tap "New" or use Chat to create agents</p>
  </div>
{:else}
  <ul>
    {#each [...agents.values()] as agent (agent.id)}
      <AgentListItem
        {agent}
        isSelected={isAgentSelected(agent.id)}
        onSelect={onSelectAgent}
        onMultiSelect={onMultiSelectAgent}
      />
    {/each}
  </ul>
{/if}

<style>
  ul {
    list-style: none;
    padding: var(--space-sm);
  }

  li {
    padding: 0;
    margin-bottom: var(--space-sm);
  }

  .agent-btn {
    width: 100%;
    padding: var(--space-md) var(--space-lg);
    display: flex;
    align-items: center;
    gap: 16px;
    cursor: pointer;
    border-radius: 12px;
    transition: all 0.2s ease;
    background-color: var(--bg-tertiary);
    border: 1px solid transparent;
    text-align: left;
    font: inherit;
    color: inherit;
  }

  .agent-btn:hover {
    background-color: var(--bg-elevated);
    border-color: var(--border);
  }

  .agent-btn.selected {
    background: linear-gradient(135deg, rgba(124, 58, 237, 0.15) 0%, rgba(147, 51, 234, 0.1) 100%);
    border-color: var(--accent);
    box-shadow: 0 0 16px var(--accent-glow);
  }

  .chat-assistant {
    background: linear-gradient(135deg, rgba(124, 58, 237, 0.1) 0%, rgba(147, 51, 234, 0.05) 100%);
    border: 1px solid rgba(124, 58, 237, 0.3);
  }

  .chat-assistant.selected {
    background: linear-gradient(135deg, rgba(124, 58, 237, 0.2) 0%, rgba(147, 51, 234, 0.15) 100%);
    border-color: var(--accent);
    box-shadow: 0 0 16px var(--accent-glow);
  }

  .chat-icon {
    font-size: 24px;
    width: 28px;
    text-align: center;
  }

  .info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .name-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .name {
    font-weight: 600;
    font-size: 16px;
    color: var(--text-primary);
  }

  .meta-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .path {
    font-size: 13px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .chevron {
    width: 20px;
    height: 20px;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .agent-btn.selected .chevron {
    color: var(--accent);
  }

  .separator {
    padding: 12px var(--space-lg);
    font-size: 12px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-xl);
    text-align: center;
    height: 100%;
    min-height: 300px;
  }

  .empty-icon {
    width: 80px;
    height: 80px;
    border-radius: 24px;
    background: linear-gradient(135deg, var(--bg-tertiary) 0%, var(--bg-elevated) 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: var(--space-lg);
    border: 1px solid var(--border);
  }

  .empty-icon svg {
    width: 40px;
    height: 40px;
    color: var(--text-muted);
  }

  .empty-title {
    font-size: 18px;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: var(--space-sm);
  }

  .empty-hint {
    font-size: 14px;
    color: var(--text-muted);
  }
</style>
