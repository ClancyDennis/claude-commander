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
    padding: 12px 20px;
    background: rgba(124, 58, 237, 0.05);
    border-top: 1px solid rgba(124, 58, 237, 0.2);
    border-bottom: 1px solid rgba(124, 58, 237, 0.2);
  }

  .section-title {
    font-size: 12px;
    font-weight: 600;
    color: #7c3aed;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 8px;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .agent-results-buttons {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .process-results-btn {
    padding: 10px 14px;
    background: linear-gradient(135deg, rgba(124, 58, 237, 0.1) 0%, rgba(109, 40, 217, 0.1) 100%);
    border: 1px solid rgba(124, 58, 237, 0.3);
    border-radius: 8px;
    color: #e0e0e0;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
    text-align: left;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .process-results-btn:hover:not(:disabled) {
    background: linear-gradient(135deg, rgba(124, 58, 237, 0.2) 0%, rgba(109, 40, 217, 0.2) 100%);
    border-color: rgba(124, 58, 237, 0.5);
    transform: translateX(2px);
  }

  .process-results-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .process-results-btn.processing {
    background: linear-gradient(135deg, rgba(124, 58, 237, 0.3) 0%, rgba(109, 40, 217, 0.3) 100%);
    animation: pulse 1.5s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }

  .output-count {
    color: #999;
    font-size: 12px;
    margin-left: auto;
  }
</style>
