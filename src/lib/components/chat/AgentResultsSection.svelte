<script lang="ts">
  import HelpTip from "../new-agent/HelpTip.svelte";

  interface AgentWithOutputs {
    id: string;
    workingDir: string;
    outputCount: number;
  }

  interface Props {
    agents: AgentWithOutputs[];
    processingAgentId: string | null;
    disabled: boolean;
    onProcessResults: (agentId: string, resultsOnly: boolean) => void;
    onDismiss: (agentId: string) => void;
  }

  let { agents, processingAgentId, disabled, onProcessResults, onDismiss }: Props = $props();
</script>

<div class="agent-results-section">
  <div class="section-title">
    Agents with results:
    <HelpTip
      text="Send agent outputs to System Commander. 'Results only' sends just the assistant's text responses. 'Full output' includes tool calls and their results."
      placement="right"
    />
  </div>
  <div class="agent-results-list">
    {#each agents as agent (agent.id)}
      <div class="agent-result-item">
        <div class="agent-info">
          <span class="agent-name">{agent.workingDir}</span>
          <span class="output-count">({agent.outputCount} outputs)</span>
        </div>
        <div class="agent-buttons">
          <button
            onclick={() => onProcessResults(agent.id, true)}
            disabled={processingAgentId !== null || disabled}
            class="process-results-btn results-only"
            class:processing={processingAgentId === agent.id}
            title="Send only assistant text responses"
          >
            📝 Results only
          </button>
          <button
            onclick={() => onProcessResults(agent.id, false)}
            disabled={processingAgentId !== null || disabled}
            class="process-results-btn full-output"
            class:processing={processingAgentId === agent.id}
            title="Send full output including tool calls"
          >
            📊 Full output
          </button>
          <button
            onclick={() => onDismiss(agent.id)}
            class="dismiss-btn"
            title="Dismiss this notification"
          >
            ✕
          </button>
        </div>
      </div>
    {/each}
  </div>
</div>

<style>
  .agent-results-section {
    padding: var(--space-3) var(--space-5);
    background: var(--bg-tertiary);
    border-top: 1px solid var(--border-hex);
    border-bottom: 1px solid var(--border-hex);
  }

  .section-title {
    font-size: var(--text-xs);
    font-weight: var(--font-semibold);
    color: var(--accent-hex);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: var(--space-2);
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .agent-results-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .agent-result-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-3);
    background: var(--bg-elevated);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-md);
  }

  .agent-info {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-width: 0;
    flex: 1;
  }

  .agent-name {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .output-count {
    color: var(--text-muted);
    font-size: var(--text-xs);
    flex-shrink: 0;
  }

  .agent-buttons {
    display: flex;
    gap: var(--space-2);
    flex-shrink: 0;
  }

  .process-results-btn {
    padding: var(--space-1) var(--space-2);
    background: var(--bg-primary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: var(--text-xs);
    font-weight: var(--font-medium);
    cursor: pointer;
    transition: all var(--transition-fast);
    display: flex;
    align-items: center;
    gap: var(--space-1);
    white-space: nowrap;
  }

  .process-results-btn:hover:not(:disabled) {
    background: rgba(232, 102, 77, 0.1);
    border-color: var(--accent-hex);
  }

  .process-results-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .process-results-btn.processing {
    background: rgba(232, 102, 77, 0.15);
    border-color: var(--accent-hex);
    animation: pulse 1.5s ease-in-out infinite;
  }

  .process-results-btn.results-only {
    background: rgba(59, 130, 246, 0.1);
    border-color: rgba(59, 130, 246, 0.3);
  }

  .process-results-btn.results-only:hover:not(:disabled) {
    background: rgba(59, 130, 246, 0.2);
    border-color: rgba(59, 130, 246, 0.5);
  }

  .dismiss-btn {
    padding: var(--space-1);
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    font-size: var(--text-sm);
    cursor: pointer;
    transition: all var(--transition-fast);
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
  }

  .dismiss-btn:hover {
    background: rgba(239, 68, 68, 0.1);
    border-color: rgba(239, 68, 68, 0.3);
    color: rgb(239, 68, 68);
  }

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.6;
    }
  }
</style>
