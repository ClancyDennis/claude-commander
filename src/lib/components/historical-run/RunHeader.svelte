<script lang="ts">
  import type { AgentRun } from "$lib/types";
  import { getStatusColorHex } from '$lib/utils/status';

  interface Props {
    run: AgentRun;
  }

  let { run }: Props = $props();

  // Extract the directory name from the full path
  let directoryName = $derived(run.working_dir.split("/").pop() || run.working_dir);
</script>

<header>
  <div class="run-info">
    <div class="run-icon">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
        <path d="M3 3v5h5"/>
        <circle cx="12" cy="12" r="1"/>
      </svg>
    </div>
    <div class="run-details">
      <h2>{directoryName}</h2>
      <div class="path-and-status">
        <span class="full-path">{run.working_dir}</span>
        <span
          class="status-badge"
          style="background-color: {getStatusColorHex(run.status)}"
        >
          {run.status.toUpperCase()}
        </span>
      </div>
    </div>
  </div>
</header>

<style>
  header {
    padding: var(--space-lg);
    border-bottom: 1px solid var(--border);
    background: linear-gradient(180deg, var(--bg-tertiary) 0%, var(--bg-secondary) 100%);
    flex-shrink: 0;
  }

  .run-info {
    display: flex;
    align-items: center;
    gap: var(--space-md);
  }

  .run-icon {
    width: 48px;
    height: 48px;
    border-radius: 12px;
    background: linear-gradient(135deg, rgba(124, 58, 237, 0.2) 0%, rgba(147, 51, 234, 0.15) 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--accent);
  }

  .run-icon svg {
    width: 28px;
    height: 28px;
    color: var(--accent);
  }

  .run-details {
    flex: 1;
    min-width: 0;
  }

  h2 {
    font-size: 22px;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0 0 8px 0;
  }

  .path-and-status {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .full-path {
    font-size: 13px;
    color: var(--text-muted);
    font-family: 'SF Mono', Menlo, Monaco, Courier, monospace;
  }

  .status-badge {
    padding: 4px 10px;
    border-radius: 12px;
    font-size: 11px;
    font-weight: 600;
    color: white;
    letter-spacing: 0.5px;
  }
</style>
