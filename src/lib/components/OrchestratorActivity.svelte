<script lang="ts">
  import {
    orchestratorToolCalls,
    orchestratorStateChanges,
    orchestratorDecisions,
    orchestratorCurrentState,
  } from "../stores/agents";
  import { ToolCallList, StateChangeList, DecisionList } from './orchestrator';

  let activeTab = $state<"tools" | "states" | "decisions">("tools");
  let expanded = $state(true);

  // Track scroll position for virtualization (passed to subcomponents)
  let scrollTop = $state(0);

  function handleScroll(event: Event) {
    const target = event.target as HTMLDivElement;
    scrollTop = target.scrollTop;
  }

  // State badge colors (used for header)
  function getStateBadgeClass(state: string): string {
    if (state.includes("Completed")) return "badge-success";
    if (state.includes("Failed") || state.includes("GaveUp")) return "badge-error";
    if (state.includes("Running") || state.includes("Executing") || state.includes("Verifying")) return "badge-info";
    if (state.includes("Planning") || state.includes("Ready")) return "badge-warning";
    return "badge-neutral";
  }
</script>

<div class="orchestrator-activity">
  <div class="activity-header" onclick={() => (expanded = !expanded)}>
    <div class="header-left">
      <span class="header-icon">🎛️</span>
      <span class="header-title">Orchestrator Activity</span>
      <span class="state-badge {getStateBadgeClass($orchestratorCurrentState)}">
        {$orchestratorCurrentState}
      </span>
    </div>
    <div class="header-right">
      <span class="count-badge">{$orchestratorToolCalls.length} tools</span>
      <span class="expand-icon">{expanded ? "▼" : "▶"}</span>
    </div>
  </div>

  {#if expanded}
    <div class="activity-tabs">
      <button
        class="tab"
        class:active={activeTab === "tools"}
        onclick={() => (activeTab = "tools")}
      >
        Tools ({$orchestratorToolCalls.length})
      </button>
      <button
        class="tab"
        class:active={activeTab === "states"}
        onclick={() => (activeTab = "states")}
      >
        States ({$orchestratorStateChanges.length})
      </button>
      <button
        class="tab"
        class:active={activeTab === "decisions"}
        onclick={() => (activeTab = "decisions")}
      >
        Decisions ({$orchestratorDecisions.length})
      </button>
    </div>

    <div class="activity-content" onscroll={handleScroll}>
      {#if activeTab === "tools"}
        <ToolCallList toolCalls={$orchestratorToolCalls} {scrollTop} />
      {:else if activeTab === "states"}
        <StateChangeList stateChanges={$orchestratorStateChanges} {scrollTop} />
      {:else if activeTab === "decisions"}
        <DecisionList decisions={$orchestratorDecisions} {scrollTop} />
      {/if}
    </div>
  {/if}
</div>

<style>
  .orchestrator-activity {
    background: var(--bg-secondary, #1a1a1f);
    border: 1px solid var(--border, rgba(240, 112, 90, 0.2));
    border-radius: 12px;
    overflow: hidden;
  }

  .activity-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: var(--bg-tertiary, #252530);
    cursor: pointer;
    user-select: none;
  }

  .activity-header:hover {
    background: var(--bg-hover, #2a2a35);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .header-icon {
    font-size: 18px;
  }

  .header-title {
    font-weight: 600;
    font-size: 14px;
    color: var(--text-primary, #e0e0e0);
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .count-badge {
    font-size: 12px;
    color: var(--text-muted, #999);
    background: var(--bg-primary, #0f0f13);
    padding: 2px 8px;
    border-radius: 10px;
  }

  .expand-icon {
    font-size: 10px;
    color: var(--text-muted, #999);
  }

  .state-badge {
    font-size: 11px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 6px;
    text-transform: uppercase;
    letter-spacing: 0.3px;
  }

  .badge-success {
    background: rgba(34, 197, 94, 0.2);
    color: #22c55e;
  }

  .badge-error {
    background: rgba(239, 68, 68, 0.2);
    color: #ef4444;
  }

  .badge-info {
    background: rgba(59, 130, 246, 0.2);
    color: #3b82f6;
  }

  .badge-warning {
    background: rgba(234, 179, 8, 0.2);
    color: #eab308;
  }

  .badge-neutral {
    background: rgba(156, 163, 175, 0.2);
    color: #9ca3af;
  }

  .activity-tabs {
    display: flex;
    border-bottom: 1px solid var(--border, rgba(240, 112, 90, 0.2));
  }

  .tab {
    flex: 1;
    padding: 10px 16px;
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--text-secondary, #999);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .tab:hover {
    color: var(--text-primary, #e0e0e0);
    background: var(--bg-tertiary, #252530);
  }

  .tab.active {
    color: var(--accent, #f0705a);
    border-bottom-color: var(--accent, #f0705a);
  }

  .activity-content {
    max-height: 400px;
    overflow-y: auto;
    padding: 12px;
  }

  /* Scrollbar styles */
  .activity-content::-webkit-scrollbar {
    width: 6px;
  }

  .activity-content::-webkit-scrollbar-track {
    background: transparent;
  }

  .activity-content::-webkit-scrollbar-thumb {
    background: var(--border, rgba(240, 112, 90, 0.3));
    border-radius: 3px;
  }

  .activity-content::-webkit-scrollbar-thumb:hover {
    background: var(--accent, rgba(240, 112, 90, 0.5));
  }
</style>
