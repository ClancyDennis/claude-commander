<script lang="ts">
  import type { ToolCall } from "../types";

  interface Props {
    toolCall: ToolCall;
  }

  let { toolCall }: Props = $props();

  let expanded = $state(false);

  const toolIcons: Record<string, string> = {
    CreateWorkerAgent: "🤖",
    SendPromptToWorker: "📤",
    StopWorkerAgent: "⏹️",
    ListWorkerAgents: "📋",
    GetAgentOutput: "📊",
    NavigateToAgent: "🧭",
    ToggleToolPanel: "🔧",
    ShowNotification: "🔔",
  };

  const icon = toolIcons[toolCall.toolName] || "⚙️";
</script>

<div class="tool-call" class:expanded>
  <div class="tool-header" onclick={() => (expanded = !expanded)}>
    <span class="tool-icon">{icon}</span>
    <span class="tool-name">{toolCall.toolName}</span>
    <span class="expand-icon">{expanded ? "▼" : "▶"}</span>
  </div>

  {#if expanded}
    <div class="tool-details">
      <div class="tool-section">
        <div class="section-title">Input:</div>
        <pre class="json-block">{JSON.stringify(toolCall.input, null, 2)}</pre>
      </div>

      {#if toolCall.output}
        <div class="tool-section">
          <div class="section-title">Output:</div>
          <pre class="json-block">{JSON.stringify(toolCall.output, null, 2)}</pre>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .tool-call {
    background: rgba(124, 58, 237, 0.1);
    border: 1px solid rgba(124, 58, 237, 0.3);
    border-radius: 8px;
    margin: 8px 0;
    overflow: hidden;
    transition: all 0.2s ease;
  }

  .tool-call.expanded {
    background: rgba(124, 58, 237, 0.15);
  }

  .tool-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    cursor: pointer;
    user-select: none;
  }

  .tool-header:hover {
    background: rgba(124, 58, 237, 0.05);
  }

  .tool-icon {
    font-size: 16px;
  }

  .tool-name {
    flex: 1;
    font-weight: 500;
    color: #7c3aed;
    font-size: 13px;
  }

  .expand-icon {
    font-size: 10px;
    color: #888;
  }

  .tool-details {
    padding: 0 14px 14px 14px;
    animation: slideDown 0.2s ease;
  }

  @keyframes slideDown {
    from {
      opacity: 0;
      transform: translateY(-5px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .tool-section {
    margin-top: 12px;
  }

  .section-title {
    font-size: 11px;
    font-weight: 600;
    color: #999;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 6px;
  }

  .json-block {
    background: rgba(0, 0, 0, 0.3);
    border-radius: 6px;
    padding: 10px;
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
    font-size: 11px;
    line-height: 1.5;
    color: #e0e0e0;
    overflow-x: auto;
    max-height: 200px;
    overflow-y: auto;
  }

  .json-block::-webkit-scrollbar {
    width: 6px;
    height: 6px;
  }

  .json-block::-webkit-scrollbar-track {
    background: rgba(0, 0, 0, 0.2);
    border-radius: 3px;
  }

  .json-block::-webkit-scrollbar-thumb {
    background: rgba(124, 58, 237, 0.4);
    border-radius: 3px;
  }

  .json-block::-webkit-scrollbar-thumb:hover {
    background: rgba(124, 58, 237, 0.6);
  }
</style>
