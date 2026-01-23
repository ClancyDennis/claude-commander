<script lang="ts">
  import type { AgentRun } from '$lib/types';
  import { formatPath, formatDate, formatDuration } from '$lib/utils/formatting';
  import { getStatusColor, getStatusLabel } from '$lib/utils/status';

  let {
    runs,
    onSelectRun
  }: {
    runs: AgentRun[];
    onSelectRun: (run: AgentRun) => void;
  } = $props();

  function truncateText(text: string, maxLength: number): string {
    return text.length > maxLength ? text.substring(0, maxLength) + '...' : text;
  }
</script>

<div class="separator">
  <span>Historical Runs ({runs.length})</span>
</div>

{#if runs.length === 0}
  <div class="empty">
    <div class="empty-icon">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
        <path d="M3 3v5h5"/>
        <circle cx="12" cy="12" r="1"/>
      </svg>
    </div>
    <p class="empty-title">No history yet</p>
    <p class="empty-hint">Agent runs will appear here</p>
  </div>
{:else}
  <ul>
    {#each runs as run (run.agent_id)}
      <li>
        <button
          class="agent-btn"
          onclick={() => onSelectRun(run)}
        >
          <div class="status-indicator" style="background-color: {getStatusColor(run.status)}">
            {#if run.status === "running"}
              <span class="pulse"></span>
            {/if}
          </div>
          <div class="info">
            <div class="name-row">
              <span class="name">{formatPath(run.working_dir)}</span>
              <span class="status-badge" style="background-color: {getStatusColor(run.status)}">
                {getStatusLabel(run.status)}
              </span>
            </div>
            <div class="meta-row">
              <span class="path">{formatDate(run.started_at)}</span>
              <span class="activity-time">{formatDuration(run.started_at, run.ended_at)}</span>
            </div>
            {#if run.initial_prompt}
              <div class="run-prompt">
                {truncateText(run.initial_prompt, 60)}
              </div>
            {/if}
          </div>
          <svg class="chevron" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="9,6 15,12 9,18"/>
          </svg>
        </button>
      </li>
    {/each}
  </ul>
{/if}

<style>
  ul {
    list-style: none;
    padding: var(--space-2);
  }

  li {
    padding: 0;
    margin-bottom: var(--space-2);
  }

  .agent-btn {
    width: 100%;
    padding: var(--space-3) var(--space-4);
    display: flex;
    align-items: center;
    gap: var(--space-4);
    cursor: pointer;
    border-radius: var(--radius-md);
    transition: all var(--transition-fast);
    background-color: var(--bg-tertiary);
    border: 1px solid transparent;
    text-align: left;
    font: inherit;
    color: inherit;
  }

  .agent-btn:hover {
    background-color: var(--bg-elevated);
    border-color: var(--border-hex);
  }

  .status-indicator {
    width: 10px;
    height: 10px;
    border-radius: var(--radius-full);
    flex-shrink: 0;
    position: relative;
  }

  .pulse {
    position: absolute;
    inset: -3px;
    border-radius: var(--radius-full);
    background: inherit;
    opacity: 0.4;
    animation: pulse 2s ease-in-out infinite;
  }

  .info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .name-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .name {
    font-weight: var(--font-medium);
    font-size: var(--text-base);
    color: var(--text-primary);
  }

  .status-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 2px var(--space-2);
    color: white;
    font-size: var(--text-xs);
    font-weight: var(--font-semibold);
    border-radius: var(--radius-sm);
    flex-shrink: 0;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .meta-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
  }

  .path {
    font-size: var(--text-sm);
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
  }

  .activity-time {
    font-size: var(--text-xs);
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .run-prompt {
    font-size: var(--text-xs);
    color: var(--text-muted);
    margin-top: 2px;
    font-style: italic;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .chevron {
    width: 16px;
    height: 16px;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .separator {
    padding: var(--space-3) var(--space-4);
    font-size: var(--text-xs);
    font-weight: var(--font-semibold);
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
    padding: var(--space-8);
    text-align: center;
    height: 100%;
    min-height: 300px;
  }

  .empty-icon {
    width: 64px;
    height: 64px;
    border-radius: var(--radius-lg);
    background: var(--bg-tertiary);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: var(--space-4);
    border: 1px solid var(--border-hex);
  }

  .empty-icon svg {
    width: 32px;
    height: 32px;
    color: var(--text-muted);
  }

  .empty-title {
    font-size: var(--text-lg);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
    margin-bottom: var(--space-2);
  }

  .empty-hint {
    font-size: var(--text-sm);
    color: var(--text-muted);
  }

  @keyframes pulse {
    0%, 100% { transform: scale(1); opacity: 0.4; }
    50% { transform: scale(1.5); opacity: 0; }
  }
</style>
