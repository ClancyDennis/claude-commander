<script lang="ts">
  import type { Agent } from "$lib/types";
  import StatusBadge from "../StatusBadge.svelte";

  let { 
    agent, 
    activeSidePanel, 
    onToggleSidePanel, 
    onClear, 
    onStop 
  }: {
    agent: Agent;
    activeSidePanel: "none" | "tools" | "stats" | "files" | "progress";
    onToggleSidePanel: (panel: "tools" | "stats" | "files" | "progress") => void;
    onClear: () => void;
    onStop: () => void;
  } = $props();
</script>

<header>
  <div class="agent-info">
    <div class="agent-icon">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="4" y="4" width="16" height="16" rx="2"/>
        <path d="M9 9h6M9 13h6M9 17h4"/>
      </svg>
    </div>
    <div class="agent-details">
      <h2>{agent.workingDir.split("/").pop()}</h2>
      <div class="path-and-github">
        <span class="full-path">{agent.workingDir}</span>
        {#if agent.githubContext}
          <a
            href={agent.githubContext.repositoryUrl}
            target="_blank"
            rel="noopener noreferrer"
            class="github-badge"
            title="Open on GitHub"
          >
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"/>
            </svg>
            {agent.githubContext.owner}/{agent.githubContext.repo}
            <span class="branch">{agent.githubContext.branch}</span>
          </a>
        {/if}
      </div>
    </div>
    <StatusBadge status={agent.status} />
  </div>
  <div class="actions">
    <button
      class="secondary"
      class:active={activeSidePanel === 'stats'}
      onclick={() => onToggleSidePanel('stats')}
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="12" y1="20" x2="12" y2="10"/>
        <line x1="18" y1="20" x2="18" y2="4"/>
        <line x1="6" y1="20" x2="6" y2="16"/>
      </svg>
      Stats
    </button>
    <button
      class="secondary"
      class:active={activeSidePanel === 'tools'}
      onclick={() => onToggleSidePanel('tools')}
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/>
      </svg>
      Tools
    </button>
    <button
      class="secondary"
      class:active={activeSidePanel === 'files'}
      onclick={() => onToggleSidePanel('files')}
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/>
        <polyline points="13 2 13 9 20 9"/>
      </svg>
      Files
    </button>
    <button
      class="secondary"
      class:active={activeSidePanel === 'progress'}
      onclick={() => onToggleSidePanel('progress')}
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M9 11l3 3L22 4"/>
        <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/>
      </svg>
      Progress
    </button>
    <button class="secondary" onclick={onClear}>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="3,6 5,6 21,6"/>
        <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/>
        <path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
      </svg>
      Clear
    </button>
    {#if agent.status === "running"}
      <button class="danger" onclick={onStop}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="6" y="6" width="12" height="12" rx="2"/>
        </svg>
        Stop
      </button>
    {/if}
  </div>
</header>

<style>
  header {
    padding: var(--space-lg);
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--border);
    background: linear-gradient(180deg, var(--bg-secondary) 0%, var(--bg-primary) 100%);
    gap: var(--space-md);
    flex-wrap: wrap;
  }

  .agent-info {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    flex: 1;
    min-width: 0;
  }

  .agent-icon {
    width: 48px;
    height: 48px;
    border-radius: 14px;
    background: linear-gradient(135deg, var(--accent) 0%, #9333ea 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    box-shadow: 0 4px 12px var(--accent-glow);
  }

  .agent-icon svg {
    width: 24px;
    height: 24px;
    color: white;
  }

  .agent-details {
    flex: 1;
    min-width: 0;
  }

  .agent-details h2 {
    font-size: 20px;
    font-weight: 700;
    margin-bottom: 2px;
  }

  .path-and-github {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    flex-wrap: wrap;
  }

  .full-path {
    font-size: 13px;
    color: var(--text-muted);
    display: block;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .actions {
    display: flex;
    gap: var(--space-sm);
    flex-wrap: wrap;
  }

  .actions button {
    padding: 12px 18px;
  }

  .actions button svg {
    width: 18px;
    height: 18px;
  }

  .actions button.active {
    background-color: var(--accent);
    color: white;
    border-color: var(--accent);
  }

  .actions button.danger {
    background-color: var(--error-glow);
    color: var(--error);
    border: 1px solid var(--error);
  }

  .actions button.danger:hover {
    background-color: var(--error);
    color: white;
  }

  .github-badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 8px;
    background: linear-gradient(135deg, #24292f 0%, #1b1f23 100%);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    font-size: 11px;
    color: #fff;
    text-decoration: none;
    font-weight: 500;
    transition: all 0.2s ease;
  }

  .github-badge:hover {
    background: linear-gradient(135deg, #2d333a 0%, #24292f 100%);
    border-color: rgba(255, 255, 255, 0.2);
    transform: translateY(-1px);
  }

  .github-badge svg {
    width: 12px;
    height: 12px;
  }

  .github-badge .branch {
    padding: 1px 4px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
    font-family: 'SF Mono', 'Monaco', 'Menlo', monospace;
  }
</style>
