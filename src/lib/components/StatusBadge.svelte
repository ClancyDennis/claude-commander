<script lang="ts">
  import type { AgentStatus } from "../types";
  import { getAgentStatusColor, getAgentStatusLabel } from '$lib/utils/status';

  let { status, size = "medium", showLabel = true }: { status: AgentStatus; size?: "small" | "medium" | "large"; showLabel?: boolean } = $props();

  function getIconSize(): string {
    switch (size) {
      case "small":
        return "12px";
      case "large":
        return "16px";
      default:
        return "14px";
    }
  }
</script>

<div class="status-badge {size} {status}">
  <div class="status-indicator" style="background-color: {getAgentStatusColor(status)}">
    {#if status === "running" || status === "waitingforinput"}
      <span class="pulse"></span>
    {/if}
    {#if status === "processing"}
      <div class="spinner" style="width: {getIconSize()}; height: {getIconSize()};">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
          <circle cx="12" cy="12" r="10" opacity="0.25"/>
          <path d="M12 2 A10 10 0 0 1 22 12" stroke-linecap="round"/>
        </svg>
      </div>
    {/if}
    {#if status === "error"}
      <svg style="width: {getIconSize()}; height: {getIconSize()};" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 2L2 22h20L12 2zm0 6l5 10H7l5-10z"/>
      </svg>
    {/if}
  </div>
  {#if showLabel}
    <span class="label">{getAgentStatusLabel(status)}</span>
  {/if}
</div>

<style>
  .status-badge {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    border-radius: 16px;
    font-size: 13px;
    font-weight: 600;
    transition: all 0.2s ease;
  }

  .status-badge.small {
    padding: 4px 8px;
    gap: 6px;
    font-size: 11px;
  }

  .status-badge.large {
    padding: 8px 16px;
    gap: 10px;
    font-size: 14px;
  }

  .status-indicator {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    flex-shrink: 0;
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .small .status-indicator {
    width: 10px;
    height: 10px;
  }

  .large .status-indicator {
    width: 14px;
    height: 14px;
  }

  .pulse {
    position: absolute;
    inset: -3px;
    border-radius: 50%;
    background: inherit;
    opacity: 0.4;
    animation: pulse 2s ease-in-out infinite;
  }

  .spinner {
    animation: spinner-rotate 1s linear infinite;
    color: white;
  }

  .status-indicator svg {
    color: white;
  }

  /* Status-specific styles */
  .running {
    background-color: var(--success-glow);
    color: var(--success);
  }

  .waitingforinput {
    background-color: var(--warning-glow);
    color: var(--warning);
    animation: glow-pulse 2s ease-in-out infinite;
  }

  .idle {
    background-color: rgba(107, 107, 122, 0.15);
    color: var(--text-secondary);
  }

  .processing {
    background-color: var(--accent-glow);
    color: var(--accent);
  }

  .stopped {
    background-color: rgba(160, 160, 160, 0.15);
    color: var(--text-muted);
  }

  .error {
    background-color: var(--error-glow);
    color: var(--error);
  }

  .label {
    white-space: nowrap;
  }

  @keyframes pulse {
    0%, 100% { transform: scale(1); opacity: 0.4; }
    50% { transform: scale(1.5); opacity: 0; }
  }
</style>
