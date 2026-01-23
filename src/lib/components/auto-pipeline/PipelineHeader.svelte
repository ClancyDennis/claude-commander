<script lang="ts">
  import type { AutoPipeline } from '$lib/types';
  import { getPipelineStatusColor } from '$lib/utils/status';

  let { pipeline }: { pipeline: AutoPipeline } = $props();

  function getStatusClass(status: string): string {
    switch (status) {
      case 'Completed': return 'status-completed';
      case 'Running': return 'status-running';
      case 'Failed': return 'status-failed';
      default: return 'status-default';
    }
  }
</script>

<header class="view-header">
  <div class="pipeline-info">
    <div class="pipeline-icon">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
        <polyline points="22 4 12 14.01 9 11.01"></polyline>
      </svg>
    </div>
    <div class="pipeline-details">
      <h2 class="truncate" title={pipeline.user_request}>{pipeline.user_request}</h2>
      <div class="path-display">
        <span class="full-path">{pipeline.working_dir}</span>
      </div>
    </div>
    <div class="status-badge {getStatusClass(pipeline.status)}">
      {pipeline.status}
    </div>
  </div>
</header>

<style>
  .view-header {
    padding: var(--space-lg);
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--border);
    background: linear-gradient(180deg, var(--bg-secondary) 0%, var(--bg-primary) 100%);
    flex-shrink: 0;
  }

  .pipeline-info {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    flex: 1;
    min-width: 0;
  }

  .pipeline-icon {
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

  .pipeline-icon svg {
    width: 24px;
    height: 24px;
    color: white;
  }

  .pipeline-details {
    flex: 1;
    min-width: 0;
  }

  .pipeline-details h2 {
    font-size: 18px;
    font-weight: 700;
    margin-bottom: 2px;
    color: var(--text-primary);
  }

  .truncate {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .full-path {
    font-size: 13px;
    color: var(--text-muted);
  }

  .status-badge {
    font-size: 11px;
    font-weight: 600;
    padding: 4px 12px;
    border-radius: 6px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .status-completed { color: #fff; background: #22c55e; box-shadow: 0 0 10px rgba(34, 197, 94, 0.4); }
  .status-running { color: #fff; background: #3b82f6; box-shadow: 0 0 10px rgba(59, 130, 246, 0.4); }
  .status-failed { color: #fff; background: #ef4444; box-shadow: 0 0 10px rgba(239, 68, 68, 0.4); }
  .status-default { color: var(--text-muted); background: var(--bg-tertiary); }
</style>
