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
    onProcessResults: (agentId: string) => void;
  }

  let { agents, processingAgentId, disabled, onProcessResults }: Props = $props();
</script>

<div class="agent-results-section">
  <div class="section-title">
    Agents with results:
    <HelpTip
      text="Click to have the System Commander analyze and summarize agent outputs."
      placement="right"
    />
  </div>
  <div class="agent-results-buttons">
    {#each agents as agent (agent.id)}
      <button
        onclick={() => onProcessResults(agent.id)}
        disabled={processingAgentId !== null || disabled}
        class="process-results-btn"
        class:processing={processingAgentId === agent.id}
      >
        📊 Process results from {agent.workingDir}
        <span class="output-count">({agent.outputCount} outputs)</span>
      </button>
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

  .agent-results-buttons {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .process-results-btn {
    padding: var(--space-2) var(--space-3);
    background: var(--bg-elevated);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-md);
    color: var(--text-primary);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    cursor: pointer;
    transition: all var(--transition-fast);
    text-align: left;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-width: 0;
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

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.6;
    }
  }

  .output-count {
    color: var(--text-muted);
    font-size: var(--text-xs);
    margin-left: auto;
    flex-shrink: 0;
  }
</style>
