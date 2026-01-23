<script lang="ts">
  import type { AgentStatus } from "../types";
  import { getAgentStatusColor, getAgentStatusLabel } from '$lib/utils/status';
  import { Loader2 } from "lucide-svelte";

  let { status, size = "medium", showLabel = true }: {
    status: AgentStatus;
    size?: "small" | "medium" | "large";
    showLabel?: boolean;
  } = $props();

  function getDotSize(): number {
    switch (size) {
      case "small": return 6;
      case "large": return 10;
      default: return 8;
    }
  }

  function getSpinnerSize(): number {
    switch (size) {
      case "small": return 10;
      case "large": return 14;
      default: return 12;
    }
  }
</script>

<div class="status-badge size-{size} status-{status}" data-tutorial="status-badge">
  <div class="status-dot" style="--dot-color: {getAgentStatusColor(status)}; --dot-size: {getDotSize()}px">
    {#if status === "running" || status === "waitingforinput"}
      <span class="pulse"></span>
    {/if}
  </div>
  {#if status === "processing"}
    <Loader2 size={getSpinnerSize()} class="spinner" />
  {/if}
  {#if showLabel}
    <span class="label">{getAgentStatusLabel(status)}</span>
  {/if}
</div>

<style>
  .status-badge {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    padding: 4px 10px;
    border-radius: var(--radius-full);
    font-size: var(--text-xs);
    font-weight: var(--font-medium);
    background: var(--bg-tertiary);
    color: var(--text-secondary);
  }

  .size-small {
    padding: 2px 8px;
    gap: 4px;
    font-size: 10px;
  }

  .size-large {
    padding: 6px 12px;
    gap: var(--space-2);
    font-size: var(--text-sm);
  }

  /* Status-specific background tints */
  .status-running {
    background: rgba(52, 199, 89, 0.12);
    color: var(--success-hex);
  }

  .status-waitingforinput {
    background: rgba(255, 149, 0, 0.12);
    color: var(--warning-hex);
  }

  .status-processing {
    background: rgba(232, 102, 77, 0.12);
    color: var(--accent-hex);
  }

  .status-error {
    background: rgba(255, 59, 48, 0.12);
    color: var(--error);
  }

  .status-idle,
  .status-stopped {
    background: var(--bg-tertiary);
    color: var(--text-muted);
  }

  .status-dot {
    width: var(--dot-size);
    height: var(--dot-size);
    border-radius: 50%;
    background-color: var(--dot-color);
    position: relative;
    flex-shrink: 0;
  }

  .pulse {
    position: absolute;
    inset: -2px;
    border-radius: 50%;
    background: var(--dot-color);
    opacity: 0.4;
    animation: pulse 2s ease-in-out infinite;
  }

  .status-badge :global(.spinner) {
    animation: spin 1s linear infinite;
    margin-left: -2px;
  }

  .label {
    white-space: nowrap;
    line-height: 1;
  }

  @keyframes pulse {
    0%, 100% { transform: scale(1); opacity: 0.4; }
    50% { transform: scale(1.8); opacity: 0; }
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
