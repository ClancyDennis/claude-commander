<script lang="ts">
  import type { Agent } from '$lib/types';
  import { formatPath, formatTimeRelative } from '$lib/utils/formatting';
  import { getStatusColor } from '$lib/utils/status';
  import { agentsWithAlerts } from '$lib/stores/security';
  import { selectedAgentIds } from '$lib/stores/agents';
  import { ChevronRight, AlertCircle, Shield } from "lucide-svelte";

  let {
    agent,
    isSelected = false,
    isMultiSelected = false,
    onSelect,
    onMultiSelect
  }: {
    agent: Agent;
    isSelected?: boolean;
    isMultiSelected?: boolean;
    onSelect: (id: string) => void;
    onMultiSelect?: (id: string) => void;
  } = $props();

  let hasSecurityAlert = $derived($agentsWithAlerts.has(agent.id));
  let inMultiSelection = $derived($selectedAgentIds.has(agent.id));

  function handleClick(e: MouseEvent) {
    if ((e.ctrlKey || e.metaKey) && onMultiSelect) {
      onMultiSelect(agent.id);
    } else {
      onSelect(agent.id);
    }
  }
</script>

<button
  class="list-item"
  class:selected={isSelected}
  class:multi-selected={inMultiSelection && !isSelected}
  onclick={handleClick}
>
  <div class="status-dot-container">
    <div class="status-dot" style="background-color: {getStatusColor(agent.status)}">
      {#if agent.status === "running" || agent.status === "waitingforinput"}
        <span class="pulse-ring"></span>
      {/if}
    </div>
    {#if agent.isProcessing}
      <div class="processing-spinner"></div>
    {/if}
  </div>

  <div class="item-content">
    <div class="title-row">
      <span class="title">{formatPath(agent.workingDir)}</span>
      {#if hasSecurityAlert}
        <Shield size={12} class="security-icon" />
      {/if}
      {#if agent.unreadOutputs && agent.unreadOutputs > 0}
        <span class="badge">{agent.unreadOutputs}</span>
      {/if}
    </div>
    <div class="subtitle-row">
      <span class="path">{agent.workingDir}</span>
      {#if agent.lastActivity}
        <span class="time">{formatTimeRelative(agent.lastActivity)}</span>
      {/if}
    </div>
  </div>

  {#if agent.pendingInput}
    <AlertCircle size={16} class="pending-icon" />
  {:else}
    <ChevronRight size={16} class="disclosure" />
  {/if}
</button>

<style>
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
    margin-bottom: 2px;
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

  .list-item.multi-selected {
    background: rgba(0, 122, 255, 0.12);
  }

  .list-item.multi-selected:hover {
    background: rgba(0, 122, 255, 0.18);
  }

  .status-dot-container {
    position: relative;
    width: 10px;
    height: 10px;
    flex-shrink: 0;
  }

  .status-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    position: relative;
  }

  .pulse-ring {
    position: absolute;
    inset: -3px;
    border-radius: 50%;
    background: inherit;
    opacity: 0.4;
    animation: pulse 2s ease-in-out infinite;
  }

  .processing-spinner {
    position: absolute;
    inset: -4px;
    border: 2px solid var(--accent-hex);
    border-radius: 50%;
    border-top-color: transparent;
    animation: spin 1s linear infinite;
  }

  .item-content {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .title {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .list-item :global(.security-icon) {
    color: var(--error);
    flex-shrink: 0;
  }

  .badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 16px;
    height: 16px;
    padding: 0 4px;
    background: var(--error);
    color: white;
    font-size: 10px;
    font-weight: var(--font-semibold);
    border-radius: var(--radius-full);
    flex-shrink: 0;
  }

  .subtitle-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
  }

  .path {
    font-size: var(--text-xs);
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
  }

  .time {
    font-size: var(--text-xs);
    color: var(--text-muted);
    flex-shrink: 0;
    opacity: 0.7;
  }

  .list-item :global(.pending-icon) {
    color: var(--warning-hex);
    flex-shrink: 0;
    animation: glow-pulse 2s ease-in-out infinite;
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

  @keyframes pulse {
    0%, 100% { transform: scale(1); opacity: 0.4; }
    50% { transform: scale(1.5); opacity: 0; }
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  @keyframes glow-pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
</style>
