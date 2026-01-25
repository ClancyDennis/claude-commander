<script lang="ts">
  import { metaAgentTodos, metaTodoSummary, currentMetaTask } from "../../stores/metaTodos";
</script>

<div class="task-progress">
  <div class="panel-header">
    <h3>Commander Tasks</h3>
    {#if $metaAgentTodos.length > 0}
      <span class="progress-badge">{$metaTodoSummary.completed}/{$metaTodoSummary.total}</span>
    {/if}
  </div>

  {#if $metaAgentTodos.length === 0}
    <div class="empty-state">
      <div class="empty-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M9 11l3 3L22 4"/>
          <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/>
        </svg>
      </div>
      <p>No orchestration tasks</p>
      <span class="hint">Tasks will appear when the commander plans multi-step operations</span>
    </div>
  {:else}
    <!-- Progress bar -->
    <div class="progress-bar-container">
      <div class="progress-bar" style="width: {$metaTodoSummary.progress}%"></div>
    </div>

    <!-- Current task highlight -->
    {#if $currentMetaTask}
      <div class="current-task">
        <span class="current-label">Currently:</span>
        <span class="current-content">{$currentMetaTask.activeForm || $currentMetaTask.content}</span>
      </div>
    {/if}

    <ul class="steps-list">
      {#each $metaAgentTodos as step, i (step.content + i)}
        <li class="step {step.status}">
          <span class="indicator">
            {#if step.status === 'completed'}
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                <polyline points="20 6 9 17 4 12"/>
              </svg>
            {:else if step.status === 'in_progress'}
              <span class="pulse-dot"></span>
            {:else}
              <span class="empty-dot"></span>
            {/if}
          </span>
          <span class="content">{step.content}</span>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .task-progress {
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-md) var(--space-lg);
    border-bottom: 1px solid var(--border);
    background-color: var(--bg-tertiary);
  }

  .panel-header h3 {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .progress-badge {
    font-size: 12px;
    background-color: var(--accent);
    color: white;
    padding: 2px 8px;
    border-radius: 10px;
    font-weight: 600;
  }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-xl);
    text-align: center;
  }

  .empty-icon {
    width: 64px;
    height: 64px;
    border-radius: 16px;
    background: linear-gradient(135deg, var(--bg-secondary) 0%, var(--bg-tertiary) 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: var(--space-md);
    border: 1px solid var(--border);
  }

  .empty-icon svg {
    width: 32px;
    height: 32px;
    color: var(--text-muted);
  }

  .empty-state p {
    font-size: 14px;
    color: var(--text-secondary);
    margin: 0 0 var(--space-xs);
  }

  .empty-state .hint {
    font-size: 12px;
    color: var(--text-muted);
  }

  .progress-bar-container {
    height: 4px;
    background-color: var(--bg-tertiary);
    overflow: hidden;
  }

  .progress-bar {
    height: 100%;
    background: linear-gradient(90deg, var(--accent-hex) 0%, #e85a45 100%);
    transition: width 0.3s ease;
  }

  .current-task {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-sm) var(--space-lg);
    background-color: rgba(240, 112, 90, 0.1);
    border-bottom: 1px solid var(--border);
  }

  .current-label {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--accent);
  }

  .current-content {
    font-size: 13px;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .steps-list {
    flex: 1;
    overflow-y: auto;
    list-style: none;
    margin: 0;
    padding: var(--space-sm) var(--space-md);
  }

  .step {
    display: flex;
    align-items: flex-start;
    gap: var(--space-sm);
    padding: var(--space-sm) var(--space-sm);
    border-radius: 6px;
    margin-bottom: 2px;
    transition: background-color 0.15s ease;
  }

  .step:hover {
    background-color: var(--bg-tertiary);
  }

  .indicator {
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    margin-top: 1px;
  }

  .indicator svg {
    width: 14px;
    height: 14px;
  }

  .step.completed .indicator svg {
    color: var(--success);
  }

  .step.completed .content {
    color: var(--text-secondary);
    text-decoration: line-through;
    text-decoration-color: var(--text-muted);
  }

  .step.in_progress {
    background-color: rgba(240, 112, 90, 0.08);
  }

  .step.in_progress .content {
    color: var(--accent);
    font-weight: 500;
  }

  .step.pending .content {
    color: var(--text-muted);
  }

  .pulse-dot {
    width: 10px;
    height: 10px;
    background-color: var(--accent);
    border-radius: 50%;
    animation: pulse 1.5s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
      transform: scale(1);
    }
    50% {
      opacity: 0.5;
      transform: scale(1.2);
    }
  }

  .empty-dot {
    width: 10px;
    height: 10px;
    border: 2px solid var(--text-muted);
    border-radius: 50%;
  }

  .content {
    flex: 1;
    font-size: 13px;
    line-height: 1.4;
  }
</style>
