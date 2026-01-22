<script lang="ts">
  import type { AutoPipeline } from '$lib/types';
  import { getPipelineStatusColor } from '$lib/utils/status';

  let {
    pipeline,
    isSelected = false,
    onSelect
  }: {
    pipeline: AutoPipeline;
    isSelected?: boolean;
    onSelect: (id: string) => void;
  } = $props();

  function truncateText(text: string, maxLength: number): string {
    return text.length > maxLength ? text.substring(0, maxLength) + '...' : text;
  }
</script>

<li>
  <button
    class="pipeline-btn"
    class:selected={isSelected}
    onclick={() => onSelect(pipeline.id)}
  >
    <div class="status-indicator" style="background-color: {getPipelineStatusColor(pipeline.status)}">
      {#if pipeline.status === 'running'}
        <span class="pulse"></span>
      {/if}
    </div>
    <div class="info">
      <div class="name-row">
        <span class="name">Pipeline</span>
        <span class="pipeline-status-badge" style="background-color: {getPipelineStatusColor(pipeline.status)}">
          {pipeline.status}
        </span>
      </div>
      <div class="meta-row">
        <span class="path pipeline-request">{truncateText(pipeline.user_request, 40)}</span>
      </div>
    </div>
    <svg class="chevron" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <polyline points="9,6 15,12 9,18"/>
    </svg>
  </button>
</li>

<style>
  li {
    padding: 0;
    margin-bottom: var(--space-sm);
  }

  .pipeline-btn {
    width: 100%;
    padding: var(--space-md) var(--space-lg);
    display: flex;
    align-items: center;
    gap: 16px;
    cursor: pointer;
    border-radius: 12px;
    transition: all 0.2s ease;
    background: linear-gradient(135deg, rgba(16, 185, 129, 0.1) 0%, rgba(5, 150, 105, 0.05) 100%);
    border: 1px solid rgba(16, 185, 129, 0.3);
    text-align: left;
    font: inherit;
    color: inherit;
  }

  .pipeline-btn:hover {
    background-color: var(--bg-elevated);
    border-color: var(--border);
  }

  .pipeline-btn.selected {
    background: linear-gradient(135deg, rgba(16, 185, 129, 0.2) 0%, rgba(5, 150, 105, 0.15) 100%);
    border-color: #10b981;
    box-shadow: 0 0 16px rgba(16, 185, 129, 0.3);
  }

  .status-indicator {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    flex-shrink: 0;
    position: relative;
  }

  .pulse {
    position: absolute;
    inset: -3px;
    border-radius: 50%;
    background: inherit;
    opacity: 0.4;
    animation: pulse 2s ease-in-out infinite;
  }

  .info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .name-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .name {
    font-weight: 600;
    font-size: 16px;
    color: var(--text-primary);
  }

  .pipeline-status-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 2px 8px;
    color: white;
    font-size: 10px;
    font-weight: 600;
    border-radius: 10px;
    flex-shrink: 0;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .meta-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .path {
    font-size: 13px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
  }

  .pipeline-request {
    font-style: italic;
  }

  .chevron {
    width: 20px;
    height: 20px;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .pipeline-btn.selected .chevron {
    color: #10b981;
  }

  @keyframes pulse {
    0%, 100% { transform: scale(1); opacity: 0.4; }
    50% { transform: scale(1.5); opacity: 0; }
  }
</style>
