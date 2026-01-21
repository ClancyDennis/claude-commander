<script lang="ts">
  import { agents, selectedAgentIds, gridSize } from "../stores/agents";
  import AgentCard from "./AgentCard.svelte";

  const selectedAgents = $derived(
    Array.from($selectedAgentIds)
      .map(id => $agents.get(id))
      .filter(Boolean)
  );

  const gridColumns = $derived($gridSize);
  const totalSlots = $derived($gridSize * $gridSize);
</script>

<div class="grid-view" style="grid-template-columns: repeat({gridColumns}, 1fr);">
  {#each selectedAgents as agent (agent?.id)}
    {#if agent}
      <div class="grid-cell animate-grid-expand">
        <AgentCard {agent} />
      </div>
    {/if}
  {/each}

  {#if selectedAgents.length === 0}
    <div class="empty-grid" style="grid-column: 1 / -1;">
      <div class="empty-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <rect x="3" y="3" width="7" height="7" rx="1"/>
          <rect x="14" y="3" width="7" height="7" rx="1"/>
          <rect x="3" y="14" width="7" height="7" rx="1"/>
          <rect x="14" y="14" width="7" height="7" rx="1"/>
        </svg>
      </div>
      <h3>No Agents Selected</h3>
      <p>Select multiple agents from the sidebar to view them in a grid</p>
    </div>
  {/if}
</div>

<style>
  .grid-view {
    display: grid;
    gap: var(--space-md);
    padding: var(--space-md);
    height: 100%;
    overflow-y: auto;
    background-color: var(--bg-primary);
  }

  .grid-cell {
    min-height: 300px;
    border-radius: 12px;
    overflow: hidden;
    border: 1px solid var(--border);
    background-color: var(--bg-secondary);
    transition: all 0.2s ease;
  }

  .grid-cell:hover {
    border-color: var(--accent);
    box-shadow: 0 0 16px var(--accent-glow);
  }

  .empty-grid {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 400px;
    text-align: center;
    color: var(--text-muted);
  }

  .empty-icon {
    width: 100px;
    height: 100px;
    border-radius: 32px;
    background: linear-gradient(135deg, var(--bg-secondary) 0%, var(--bg-tertiary) 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: var(--space-lg);
    border: 1px solid var(--border);
  }

  .empty-icon svg {
    width: 50px;
    height: 50px;
  }

  .empty-grid h3 {
    font-size: 24px;
    margin-bottom: var(--space-sm);
    color: var(--text-primary);
  }

  .empty-grid p {
    font-size: 16px;
    color: var(--text-muted);
  }
</style>
