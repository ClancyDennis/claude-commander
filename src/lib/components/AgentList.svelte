<script lang="ts">
  import { agents, selectedAgentId, viewMode, openChat, openAgent, sidebarMode, historicalRuns, toggleSidebarMode, setHistoricalRuns, selectHistoricalRun } from "$lib/stores/agents";
  import { autoPipelines, selectedAutoPipelineId, selectAutoPipeline } from "$lib/stores/autoPipelines";
  import type { AgentRun } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";
  import RunningAgentsList from './agent-list/RunningAgentsList.svelte';
  import HistoricalRunsList from './agent-list/HistoricalRunsList.svelte';

  let { onNewAgent, onToggleDatabaseStats }: {
    onNewAgent: () => void;
    onToggleDatabaseStats?: () => void;
  } = $props();

  // Load historical runs when sidebar mode changes to history
  $effect(() => {
    if ($sidebarMode === 'history') {
      loadHistoricalRuns();
    }
  });

  async function loadHistoricalRuns() {
    try {
      const runs = await invoke<AgentRun[]>("get_all_runs");
      setHistoricalRuns(runs);
    } catch (error) {
      console.error("Failed to load historical runs:", error);
    }
  }

  function handleToggleSidebarMode() {
    toggleSidebarMode();
  }

  function handleSelectHistoricalRun(run: AgentRun) {
    selectHistoricalRun(run);
  }

  function handleSelectAgent(id: string) {
    openAgent(id);
  }

  function handleOpenChat() {
    openChat();
  }

  function handleSelectPipeline(id: string) {
    selectAutoPipeline(id);
  }
</script>

<aside class="agent-list">
  <header>
    <div class="logo">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <circle cx="12" cy="12" r="3"/>
        <line x1="12" y1="2" x2="12" y2="6"/>
        <line x1="12" y1="18" x2="12" y2="22"/>
        <line x1="2" y1="12" x2="6" y2="12"/>
        <line x1="18" y1="12" x2="22" y2="12"/>
      </svg>
      <span>Claude Commander</span>
    </div>
    <button class="primary new-btn" onclick={onNewAgent}>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
        <line x1="12" y1="5" x2="12" y2="19"/>
        <line x1="5" y1="12" x2="19" y2="12"/>
      </svg>
      New
    </button>
  </header>

  <!-- Toggle between Running and History -->
  <div class="sidebar-toggle">
    <button
      class="toggle-btn"
      class:active={$sidebarMode === 'running'}
      onclick={() => handleToggleSidebarMode()}
      disabled={$sidebarMode === 'running'}
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <polyline points="12 6 12 12 16 14"/>
      </svg>
      Running
    </button>
    <button
      class="toggle-btn"
      class:active={$sidebarMode === 'history'}
      onclick={() => handleToggleSidebarMode()}
      disabled={$sidebarMode === 'history'}
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
        <path d="M3 3v5h5"/>
        <polyline points="12 7 12 12 15 15"/>
      </svg>
      History
    </button>
  </div>

  <div class="list-container">
    {#if $sidebarMode === 'running'}
      <RunningAgentsList
        agents={$agents}
        pipelines={$autoPipelines}
        selectedAgentId={$selectedAgentId}
        selectedPipelineId={$selectedAutoPipelineId}
        viewMode={$viewMode}
        onOpenChat={handleOpenChat}
        onSelectAgent={handleSelectAgent}
        onSelectPipeline={handleSelectPipeline}
      />
    {:else}
      <HistoricalRunsList
        runs={$historicalRuns}
        onSelectRun={handleSelectHistoricalRun}
      />
    {/if}
  </div>

  {#if onToggleDatabaseStats}
    <footer class="agent-list-footer">
      <button class="footer-btn" onclick={onToggleDatabaseStats} title="Toggle Database Stats (Ctrl+Shift+D)">
        DB Stats
      </button>
    </footer>
  {/if}
</aside>

<style>
  .agent-list {
    width: 320px;
    min-width: 320px;
    height: 100%;
    background-color: var(--bg-secondary);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
  }

  header {
    padding: var(--space-lg);
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--border);
    background: linear-gradient(180deg, var(--bg-tertiary) 0%, var(--bg-secondary) 100%);
  }

  .logo {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 18px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .logo svg {
    width: 28px;
    height: 28px;
    color: var(--accent);
  }

  .new-btn {
    padding: 12px 20px;
  }

  .new-btn svg {
    width: 18px;
    height: 18px;
  }

  .sidebar-toggle {
    display: flex;
    gap: 8px;
    padding: var(--space-md) var(--space-lg);
    border-bottom: 1px solid var(--border);
    background-color: var(--bg-primary);
  }

  .toggle-btn {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 10px 16px;
    border: 1px solid var(--border);
    background-color: var(--bg-tertiary);
    color: var(--text-muted);
    border-radius: 8px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .toggle-btn svg {
    width: 16px;
    height: 16px;
  }

  .toggle-btn:hover:not(:disabled) {
    background-color: var(--bg-elevated);
    border-color: var(--accent);
    color: var(--text-primary);
  }

  .toggle-btn.active {
    background: linear-gradient(135deg, rgba(124, 58, 237, 0.15) 0%, rgba(147, 51, 234, 0.1) 100%);
    border-color: var(--accent);
    color: var(--accent);
    box-shadow: 0 0 8px var(--accent-glow);
  }

  .toggle-btn:disabled {
    cursor: default;
  }

  .list-container {
    flex: 1;
    overflow-y: auto;
  }

  .agent-list-footer {
    display: flex;
    gap: 0.5rem;
    padding: 1rem;
    border-top: 1px solid var(--border);
    background: var(--bg-primary);
  }

  .footer-btn {
    flex: 1;
    padding: 0.75rem;
    border: 1px solid var(--border);
    background: var(--bg-secondary);
    color: var(--text);
    border-radius: 6px;
    font-size: 0.875rem;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.25rem;
  }

  .footer-btn:hover {
    background: var(--bg-tertiary);
    border-color: var(--accent);
  }
</style>
