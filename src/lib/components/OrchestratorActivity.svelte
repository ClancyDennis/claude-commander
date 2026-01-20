<script lang="ts">
  import {
    orchestratorToolCalls,
    orchestratorStateChanges,
    orchestratorDecisions,
    orchestratorCurrentState,
  } from "../stores/agents";
  import { formatTimeLocale } from '$lib/utils/formatting';

  let activeTab = $state<"tools" | "states" | "decisions">("tools");
  let expanded = $state(true);

  // State badge colors
  function getStateBadgeClass(state: string): string {
    if (state.includes("Completed")) return "badge-success";
    if (state.includes("Failed") || state.includes("GaveUp")) return "badge-error";
    if (state.includes("Running") || state.includes("Executing") || state.includes("Verifying")) return "badge-info";
    if (state.includes("Planning") || state.includes("Ready")) return "badge-warning";
    return "badge-neutral";
  }

  // Decision badge colors
  function getDecisionBadgeClass(decision: string): string {
    if (decision === "Complete") return "badge-success";
    if (decision === "Iterate") return "badge-warning";
    if (decision === "Replan") return "badge-info";
    if (decision === "GiveUp") return "badge-error";
    return "badge-neutral";
  }

  // Tool icon mapping
  function getToolIcon(toolName: string): string {
    const icons: Record<string, string> = {
      read_instruction_file: "📖",
      create_skill: "🎯",
      create_subagent: "🤖",
      generate_claudemd: "📝",
      start_planning: "📋",
      approve_plan: "✅",
      start_execution: "🔨",
      start_verification: "🔍",
      complete: "🎉",
      iterate: "🔄",
      replan: "📋",
      give_up: "❌",
    };
    return icons[toolName] || "⚙️";
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

    <div class="activity-content">
      {#if activeTab === "tools"}
        <div class="tool-list">
          {#if $orchestratorToolCalls.length === 0}
            <div class="empty-state">No tool calls yet</div>
          {:else}
            {#each $orchestratorToolCalls as call, i}
              <div class="tool-item" class:error={call.is_error}>
                <div class="tool-header">
                  <span class="tool-icon">{getToolIcon(call.tool_name)}</span>
                  <span class="tool-name">{call.tool_name}</span>
                  <span class="tool-time">{formatTimeLocale(call.timestamp)}</span>
                </div>
                {#if call.summary}
                  <div class="tool-summary">{call.summary}</div>
                {/if}
                {#if call.tool_input && Object.keys(call.tool_input).length > 0}
                  <details class="tool-details">
                    <summary>Input</summary>
                    <pre>{JSON.stringify(call.tool_input, null, 2)}</pre>
                  </details>
                {/if}
              </div>
            {/each}
          {/if}
        </div>
      {:else if activeTab === "states"}
        <div class="state-list">
          {#if $orchestratorStateChanges.length === 0}
            <div class="empty-state">No state changes yet</div>
          {:else}
            {#each $orchestratorStateChanges as change, i}
              <div class="state-item">
                <div class="state-transition">
                  <span class="state-badge {getStateBadgeClass(change.old_state)}">
                    {change.old_state}
                  </span>
                  <span class="arrow">→</span>
                  <span class="state-badge {getStateBadgeClass(change.new_state)}">
                    {change.new_state}
                  </span>
                </div>
                <div class="state-meta">
                  <span class="iteration">Iteration {change.iteration}</span>
                  <span class="time">{formatTimeLocale(change.timestamp)}</span>
                </div>
                {#if change.generated_skills > 0 || change.generated_subagents > 0 || change.claudemd_generated}
                  <div class="state-resources">
                    {#if change.generated_skills > 0}
                      <span class="resource">🎯 {change.generated_skills} skills</span>
                    {/if}
                    {#if change.generated_subagents > 0}
                      <span class="resource">🤖 {change.generated_subagents} subagents</span>
                    {/if}
                    {#if change.claudemd_generated}
                      <span class="resource">📝 CLAUDE.md</span>
                    {/if}
                  </div>
                {/if}
              </div>
            {/each}
          {/if}
        </div>
      {:else if activeTab === "decisions"}
        <div class="decision-list">
          {#if $orchestratorDecisions.length === 0}
            <div class="empty-state">No decisions yet</div>
          {:else}
            {#each $orchestratorDecisions as decision, i}
              <div class="decision-item">
                <div class="decision-header">
                  <span class="decision-badge {getDecisionBadgeClass(decision.decision)}">
                    {decision.decision}
                  </span>
                  <span class="decision-time">{formatTimeLocale(decision.timestamp)}</span>
                </div>
                <div class="decision-reasoning">{decision.reasoning}</div>
                {#if decision.issues.length > 0}
                  <div class="decision-issues">
                    <strong>Issues:</strong>
                    <ul>
                      {#each decision.issues as issue}
                        <li>{issue}</li>
                      {/each}
                    </ul>
                  </div>
                {/if}
                {#if decision.suggestions.length > 0}
                  <div class="decision-suggestions">
                    <strong>Suggestions:</strong>
                    <ul>
                      {#each decision.suggestions as suggestion}
                        <li>{suggestion}</li>
                      {/each}
                    </ul>
                  </div>
                {/if}
              </div>
            {/each}
          {/if}
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .orchestrator-activity {
    background: var(--bg-secondary, #1a1a1f);
    border: 1px solid var(--border, rgba(124, 58, 237, 0.2));
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
    border-bottom: 1px solid var(--border, rgba(124, 58, 237, 0.2));
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
    color: var(--accent, #7c3aed);
    border-bottom-color: var(--accent, #7c3aed);
  }

  .activity-content {
    max-height: 400px;
    overflow-y: auto;
    padding: 12px;
  }

  .empty-state {
    text-align: center;
    padding: 24px;
    color: var(--text-muted, #666);
    font-size: 13px;
  }

  /* Tool list styles */
  .tool-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .tool-item {
    background: var(--bg-primary, #0f0f13);
    border: 1px solid var(--border, rgba(124, 58, 237, 0.15));
    border-radius: 8px;
    padding: 10px 12px;
  }

  .tool-item.error {
    border-color: rgba(239, 68, 68, 0.4);
  }

  .tool-header {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .tool-icon {
    font-size: 14px;
  }

  .tool-name {
    font-weight: 600;
    font-size: 13px;
    color: var(--text-primary, #e0e0e0);
    flex: 1;
  }

  .tool-time {
    font-size: 11px;
    color: var(--text-muted, #666);
  }

  .tool-summary {
    margin-top: 6px;
    font-size: 12px;
    color: var(--text-secondary, #999);
    line-height: 1.4;
  }

  .tool-details {
    margin-top: 8px;
  }

  .tool-details summary {
    font-size: 11px;
    color: var(--accent, #7c3aed);
    cursor: pointer;
  }

  .tool-details pre {
    margin-top: 6px;
    padding: 8px;
    background: var(--bg-tertiary, #252530);
    border-radius: 6px;
    font-size: 11px;
    overflow-x: auto;
    color: var(--text-secondary, #999);
  }

  /* State list styles */
  .state-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .state-item {
    background: var(--bg-primary, #0f0f13);
    border: 1px solid var(--border, rgba(124, 58, 237, 0.15));
    border-radius: 8px;
    padding: 10px 12px;
  }

  .state-transition {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .arrow {
    color: var(--text-muted, #666);
    font-size: 14px;
  }

  .state-meta {
    display: flex;
    justify-content: space-between;
    margin-top: 8px;
    font-size: 11px;
    color: var(--text-muted, #666);
  }

  .state-resources {
    display: flex;
    gap: 12px;
    margin-top: 8px;
    font-size: 11px;
  }

  .resource {
    color: var(--text-secondary, #999);
  }

  /* Decision list styles */
  .decision-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .decision-item {
    background: var(--bg-primary, #0f0f13);
    border: 1px solid var(--border, rgba(124, 58, 237, 0.15));
    border-radius: 8px;
    padding: 12px;
  }

  .decision-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }

  .decision-badge {
    font-size: 12px;
    font-weight: 600;
    padding: 4px 12px;
    border-radius: 6px;
  }

  .decision-time {
    font-size: 11px;
    color: var(--text-muted, #666);
  }

  .decision-reasoning {
    font-size: 13px;
    color: var(--text-primary, #e0e0e0);
    line-height: 1.5;
    margin-bottom: 10px;
  }

  .decision-issues,
  .decision-suggestions {
    font-size: 12px;
    margin-top: 8px;
  }

  .decision-issues strong,
  .decision-suggestions strong {
    color: var(--text-secondary, #999);
  }

  .decision-issues ul,
  .decision-suggestions ul {
    margin: 4px 0 0 16px;
    padding: 0;
    list-style: disc;
  }

  .decision-issues li,
  .decision-suggestions li {
    color: var(--text-primary, #e0e0e0);
    margin-bottom: 2px;
  }

  /* Scrollbar styles */
  .activity-content::-webkit-scrollbar {
    width: 6px;
  }

  .activity-content::-webkit-scrollbar-track {
    background: transparent;
  }

  .activity-content::-webkit-scrollbar-thumb {
    background: var(--border, rgba(124, 58, 237, 0.3));
    border-radius: 3px;
  }

  .activity-content::-webkit-scrollbar-thumb:hover {
    background: var(--accent, rgba(124, 58, 237, 0.5));
  }
</style>
