<script lang="ts">
  import type { ToolEvent } from "../../types";
  import { formatTimeDuration } from "$lib/utils/formatting";
  import { formatToolInput } from "$lib/utils/toolStats";
  import ToolIcon from "./ToolIcon.svelte";

  let { event }: { event: ToolEvent } = $props();
</script>

<div class="event" class:pending={event.status === "pending"}>
  <div class="event-header">
    <div class="tool-info">
      <ToolIcon toolName={event.toolName} />
      <div class="tool-details">
        <span class="tool-name">{event.toolName}</span>
        <span class="tool-meta">
          {#if event.executionTimeMs !== undefined}
            <span class="execution-time">{formatTimeDuration(event.executionTimeMs)}</span>
            <span class="separator">.</span>
          {/if}
          <span class="timestamp">{event.timestamp.toLocaleTimeString()}</span>
        </span>
      </div>
    </div>

    <!-- Status indicator -->
    {#if event.status}
      <div class="status-indicator {event.status}">
        {#if event.status === "pending"}
          <div class="spinner"></div>
        {:else if event.status === "success"}
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="20 6 9 17 4 12"/>
          </svg>
        {:else if event.status === "failed"}
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        {/if}
      </div>
    {/if}
  </div>

  <div class="event-content">
    <code>{formatToolInput(event.toolInput)}</code>
  </div>

  {#if event.errorMessage}
    <div class="error-message">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <circle cx="12" cy="12" r="10"/>
        <line x1="12" y1="8" x2="12" y2="12"/>
        <line x1="12" y1="16" x2="12.01" y2="16"/>
      </svg>
      <span>{event.errorMessage}</span>
    </div>
  {/if}
</div>

<style>
  .event {
    padding: var(--space-3);
    margin-bottom: var(--space-2);
    background-color: var(--bg-tertiary);
    border-radius: var(--radius-md);
    transition: background var(--transition-fast);
  }

  .event:hover {
    background-color: var(--bg-elevated);
  }

  .event.pending {
    background-color: rgba(255, 255, 255, 0.02);
  }

  .event-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-3);
    margin-bottom: var(--space-2);
  }

  .tool-info {
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
    min-width: 0;
  }

  .tool-details {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .tool-name {
    font-weight: var(--font-medium);
    font-size: var(--text-sm);
    color: var(--text-primary);
  }

  .tool-meta {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    font-size: var(--text-xs);
    color: var(--text-muted);
  }

  .separator {
    opacity: 0.5;
  }

  .execution-time {
    font-variant-numeric: tabular-nums;
  }

  .timestamp {
    font-variant-numeric: tabular-nums;
  }

  /* Status Indicator */
  .status-indicator {
    width: 20px;
    height: 20px;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .status-indicator svg {
    width: 12px;
    height: 12px;
  }

  .status-indicator.pending {
    background: transparent;
  }

  .status-indicator.success {
    color: var(--success-hex);
  }

  .status-indicator.failed {
    color: var(--error);
  }

  .spinner {
    width: 12px;
    height: 12px;
    border: 1.5px solid var(--border-hex);
    border-top-color: var(--text-muted);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  /* Event Content */
  .event-content {
    padding: var(--space-2) var(--space-3);
    background-color: var(--bg-primary);
    border-radius: var(--radius-sm);
  }

  code {
    font-family: 'SF Mono', 'Monaco', 'Menlo', monospace;
    font-size: var(--text-xs);
    color: var(--text-secondary);
    display: block;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* Error Message */
  .error-message {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    background-color: rgba(255, 59, 48, 0.1);
    border-radius: var(--radius-sm);
    margin-top: var(--space-2);
    font-size: var(--text-xs);
    color: var(--error);
    line-height: var(--leading-normal);
  }

  .error-message svg {
    width: 14px;
    height: 14px;
    flex-shrink: 0;
    margin-top: 1px;
  }

  .error-message span {
    min-width: 0;
    word-break: break-word;
  }
</style>
