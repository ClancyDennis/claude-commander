<script lang="ts">
  import { agents, selectedAgentId, viewMode, openChat, openAgent, toggleAgentInSelection, sidebarMode, historicalRuns, toggleSidebarMode, setHistoricalRuns, selectHistoricalRun } from "$lib/stores/agents";
  import { autoPipelines, selectedAutoPipelineId, selectAutoPipeline } from "$lib/stores/autoPipelines";
  import type { AgentRun } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";
  import RunningAgentsList from './agent-list/RunningAgentsList.svelte';
  import HistoricalRunsList from './agent-list/HistoricalRunsList.svelte';
  import HelpTip from "./new-agent/HelpTip.svelte";
  import NotificationBell from "./NotificationBell.svelte";
  import { SegmentedControl } from "./ui/segmented-control";
  import { Plus, Settings, Database, Radio } from "lucide-svelte";

  let { onNewAgent, onToggleDatabaseStats, onOpenSettings }: {
    onNewAgent: () => void;
    onToggleDatabaseStats?: () => void;
    onOpenSettings?: () => void;
  } = $props();

  const sidebarSegments = [
    { id: 'running', label: 'Running' },
    { id: 'history', label: 'History' }
  ];

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

  function handleSidebarModeChange(mode: string) {
    if (mode !== $sidebarMode) {
      toggleSidebarMode();
    }
  }

  function handleSelectHistoricalRun(run: AgentRun) {
    selectHistoricalRun(run);
  }

  function handleSelectAgent(id: string) {
    openAgent(id);
  }

  function handleMultiSelectAgent(id: string) {
    toggleAgentInSelection(id);
  }

  function handleOpenChat() {
    openChat();
  }

  function handleSelectPipeline(id: string) {
    selectAutoPipeline(id);
  }
</script>

<aside class="sidebar" data-tutorial="agent-list">
  <header class="sidebar-header">
    <div class="app-title">
      <div class="app-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"/>
          <circle cx="12" cy="12" r="3"/>
          <line x1="12" y1="2" x2="12" y2="6"/>
          <line x1="12" y1="18" x2="12" y2="22"/>
          <line x1="2" y1="12" x2="6" y2="12"/>
          <line x1="18" y1="12" x2="22" y2="12"/>
        </svg>
      </div>
      <span>Claude Commander</span>
    </div>
    <button class="new-btn" data-tutorial="new-button" onclick={onNewAgent}>
      <Plus size={16} strokeWidth={2.5} />
      <span>New</span>
    </button>
  </header>

  <div class="system-control-section">
    <button
      class="system-control-btn"
      class:active={$viewMode === 'chat'}
      onclick={handleOpenChat}
    >
      <Radio size={16} />
      <span>System Control</span>
    </button>
  </div>

  <div class="sidebar-controls">
    <SegmentedControl
      segments={sidebarSegments}
      selected={$sidebarMode}
      onSelect={handleSidebarModeChange}
      size="sm"
      class="mode-toggle"
    />
    <HelpTip text="Running shows active agents. History shows completed sessions." placement="right" />
  </div>

  <div class="list-container">
    {#if $sidebarMode === 'running'}
      <RunningAgentsList
        agents={$agents}
        pipelines={$autoPipelines}
        selectedAgentId={$selectedAgentId}
        selectedPipelineId={$selectedAutoPipelineId}
        viewMode={$viewMode}
        onSelectAgent={handleSelectAgent}
        onMultiSelectAgent={handleMultiSelectAgent}
        onSelectPipeline={handleSelectPipeline}
      />
    {:else}
      <HistoricalRunsList
        runs={$historicalRuns}
        onSelectRun={handleSelectHistoricalRun}
      />
    {/if}
  </div>

  {#if onToggleDatabaseStats || onOpenSettings}
    <footer class="sidebar-footer">
      <NotificationBell />
      {#if onOpenSettings}
        <button class="footer-btn" onclick={onOpenSettings} title="Settings">
          <Settings size={16} />
        </button>
      {/if}
      {#if onToggleDatabaseStats}
        <button class="footer-btn" onclick={onToggleDatabaseStats} title="Database Stats">
          <Database size={16} />
        </button>
      {/if}
    </footer>
  {/if}
</aside>

<style>
  .sidebar {
    width: 280px;
    min-width: 280px;
    height: 100%;
    background: rgba(26, 26, 26, 0.95);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border-right: 1px solid rgba(255, 255, 255, 0.06);
    display: flex;
    flex-direction: column;
  }

  .sidebar-header {
    padding: var(--space-4);
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }

  .app-title {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-lg);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
  }

  .app-icon {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .app-icon svg {
    width: 20px;
    height: 20px;
    color: var(--accent-hex);
  }

  .new-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    background: var(--accent-hex);
    color: white;
    border: none;
    border-radius: var(--radius-sm);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    cursor: pointer;
    transition: all var(--transition-normal);
  }

  .new-btn:hover {
    background: var(--accent-hover);
  }

  .new-btn:active {
    transform: scale(0.98);
  }

  .system-control-section {
    padding: var(--space-3) var(--space-4);
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }

  .system-control-btn {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .system-control-btn:hover {
    background: var(--accent-glow);
    border-color: var(--accent-hex);
    color: var(--accent-hex);
  }

  .system-control-btn.active {
    background: var(--accent-hex);
    border-color: var(--accent-hex);
    color: white;
  }

  .sidebar-controls {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-4);
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }

  .sidebar-controls :global(.mode-toggle) {
    flex: 1;
  }

  /* Force equal width segments */
  .sidebar-controls :global(.mode-toggle .segment) {
    flex: 1;
  }

  .list-container {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .sidebar-footer {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-4);
    border-top: 1px solid rgba(255, 255, 255, 0.06);
    background: rgba(15, 15, 15, 0.5);
  }

  .footer-btn {
    width: 32px;
    height: 32px;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    cursor: pointer;
    transition: all var(--transition-normal);
  }

  .footer-btn:hover {
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-primary);
  }

  .footer-btn:active {
    background: rgba(255, 255, 255, 0.12);
  }
</style>
