<script lang="ts">
  type ToolCall = {
    tool_name?: string;
    tool_input?: Record<string, unknown> | string;
    completed?: boolean;
    is_error?: boolean;
  };

  let {
    orchestratorCurrentState,
    isOrchestratorActive,
    totalToolCount,
    currentTool,
    currentToolDisplay,
    isToolExecuting,
    currentToolStatus
  }: {
    orchestratorCurrentState: string;
    isOrchestratorActive: boolean;
    totalToolCount: number;
    currentTool: ToolCall | null;
    currentToolDisplay: string;
    isToolExecuting: boolean;
    currentToolStatus: 'idle' | 'executing' | 'completed' | 'error';
  } = $props();
</script>

<div class="orchestrator-status" class:active={isOrchestratorActive}>
  <div class="orchestrator-icon">🎛️</div>
  <div class="orchestrator-info">
    <div class="orchestrator-stats">
      <span class="tool-count">{totalToolCount} tools</span>
      <span class="state-label" class:active={isOrchestratorActive}>
        {orchestratorCurrentState}
      </span>
    </div>
    <div class="current-tool"
         class:active={isOrchestratorActive}
         class:executing={currentToolStatus === 'executing'}
         class:completed={currentToolStatus === 'completed'}
         class:error={currentToolStatus === 'error'}
         class:idle={currentToolStatus === 'idle'}>
      {#if isToolExecuting}
        <span class="tool-spinner"></span>
      {:else if currentTool && currentToolStatus === 'error'}
        <span class="tool-icon error">x</span>
      {:else if currentTool}
        <span class="tool-icon success">v</span>
      {:else}
        <span class="tool-icon">*</span>
      {/if}
      <span class="tool-name" title={currentToolDisplay}>{currentToolDisplay}</span>
    </div>
  </div>
</div>

<style>
  .orchestrator-status {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-sm) var(--space-md);
    background: var(--bg-tertiary);
    border-radius: 10px;
    border: 1px solid var(--border);
    flex-shrink: 0;
    min-width: 200px;
    transition: all 0.3s ease;
  }

  .orchestrator-status.active {
    border-color: var(--accent);
    box-shadow: 0 0 12px var(--accent-glow);
  }

  .orchestrator-icon {
    font-size: 28px;
    width: 44px;
    height: 44px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, var(--accent-glow) 0%, rgba(124, 58, 237, 0.2) 100%);
    border-radius: 10px;
  }

  .orchestrator-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
    flex: 1;
  }

  .orchestrator-stats {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .orchestrator-stats .tool-count {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .state-label {
    font-size: 11px;
    font-weight: 700;
    padding: 2px 8px;
    border-radius: 4px;
    background: var(--bg-primary);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.3px;
  }

  .state-label.active {
    background: var(--accent-glow);
    color: var(--accent);
  }

  .current-tool {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    color: var(--text-muted);
    transition: all 0.3s ease;
  }

  .current-tool.active {
    color: var(--text-primary);
  }

  .current-tool.executing {
    color: var(--accent);
  }

  .current-tool.idle {
    opacity: 0.6;
  }

  .tool-icon {
    font-size: 10px;
  }

  .tool-icon.success {
    color: var(--success);
    animation: checkmark 0.5s ease;
  }

  .tool-icon.error {
    color: var(--error);
    animation: shake 0.5s ease;
  }

  @keyframes checkmark {
    0% {
      transform: scale(0);
      opacity: 0;
    }
    50% {
      transform: scale(1.2);
    }
    100% {
      transform: scale(1);
      opacity: 1;
    }
  }

  @keyframes shake {
    0%, 100% { transform: translateX(0); }
    25% { transform: translateX(-5px); }
    75% { transform: translateX(5px); }
  }

  .tool-name {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 140px;
    font-family: var(--font-mono, monospace);
    font-size: 11px;
  }

  .tool-spinner {
    width: 10px;
    height: 10px;
    border: 2px solid var(--accent-glow);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    flex-shrink: 0;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
