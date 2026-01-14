<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import {
    agents,
    agentOutputs,
    selectedAgent,
    selectedAgentOutputs,
    selectedAgentId,
    updateAgentStatus,
    clearAgentOutput,
    markAgentViewed,
  } from "../stores/agents";
  import ToolActivity from "./ToolActivity.svelte";
  import TypingIndicator from "./TypingIndicator.svelte";
  import StatusBadge from "./StatusBadge.svelte";
  import AgentStats from "./AgentStats.svelte";
  import OutputControls from "./OutputControls.svelte";
  import ExportDialog from "./ExportDialog.svelte";
  import type { AgentOutput } from "../types";

  let { agentId, showHeader = true, compact = false }: { agentId?: string; showHeader?: boolean; compact?: boolean } = $props();

  // Use provided agentId or fall back to selected agent
  const effectiveAgentId = $derived(agentId || $selectedAgentId);

  const agent = $derived(
    effectiveAgentId ? $agents.get(effectiveAgentId) ?? null : $selectedAgent
  );

  const outputs = $derived(
    effectiveAgentId
      ? $agentOutputs.get(effectiveAgentId) ?? []
      : $selectedAgentOutputs
  );

  let promptInput = $state("");
  let outputContainer: HTMLDivElement | null = $state(null);
  let showTools = $state(false);
  let showStats = $state(false);
  let showExportDialog = $state(false);

  // Filtered outputs managed by OutputControls
  let filteredOutputs = $state<AgentOutput[]>([]);
  let hasReceivedFilter = $state(false);

  // Initialize filtered outputs with all outputs when first loaded
  $effect(() => {
    if (!hasReceivedFilter && outputs.length > 0) {
      filteredOutputs = outputs;
    }
  });

  // Mark agent as viewed when it's displayed
  $effect(() => {
    if (effectiveAgentId) {
      markAgentViewed(effectiveAgentId);
    }
  });

  $effect(() => {
    if (outputContainer && outputs.length > 0) {
      outputContainer.scrollTop = outputContainer.scrollHeight;
    }
  });

  async function sendPrompt() {
    if (!effectiveAgentId || !promptInput.trim()) return;

    try {
      await invoke("send_prompt", {
        agentId: effectiveAgentId,
        prompt: promptInput,
      });
      promptInput = "";
    } catch (e) {
      console.error("Failed to send prompt:", e);
    }
  }

  async function stopAgent() {
    if (!effectiveAgentId) return;

    try {
      await invoke("stop_agent", { agentId: effectiveAgentId });
      updateAgentStatus(effectiveAgentId, "stopped");
    } catch (e) {
      console.error("Failed to stop agent:", e);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      sendPrompt();
    }
  }

  function formatOutput(content: string): string {
    return content;
  }

  function formatTimestamp(timestamp: Date): string {
    return timestamp.toLocaleTimeString();
  }

  function getStatusLabel(status: string): string {
    return status.charAt(0).toUpperCase() + status.slice(1);
  }
</script>

{#if agent}
  <main class="agent-view" class:compact>
    {#if showHeader}
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
          class:active={showStats}
          onclick={() => (showStats = !showStats)}
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
          class:active={showTools}
          onclick={() => (showTools = !showTools)}
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/>
          </svg>
          Tools
        </button>
        <button class="secondary" onclick={() => clearAgentOutput(effectiveAgentId!)}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="3,6 5,6 21,6"/>
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/>
            <path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
          </svg>
          Clear
        </button>
        {#if agent.status === "running"}
          <button class="danger" onclick={stopAgent}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="6" y="6" width="12" height="12" rx="2"/>
            </svg>
            Stop
          </button>
        {/if}
      </div>
    </header>
    {/if}

    <div class="content">
      <div class="output-panel" class:with-tools={showTools} class:with-stats={showStats}>
        {#if outputs.length > 0}
          <OutputControls
            outputs={outputs}
            onFilter={(filtered) => {
              hasReceivedFilter = true;
              filteredOutputs = filtered;
            }}
            onExport={() => showExportDialog = true}
          />
        {/if}
        <div class="output" bind:this={outputContainer}>
          {#each filteredOutputs as output, i (output.timestamp.getTime() + i)}
            <div class="output-item {output.type} animate-slide-up" data-index={i}>
              <div class="output-header">
                <span class="output-type">{output.type}</span>
                <span class="timestamp">
                  {formatTimestamp(output.timestamp)}
                </span>
              </div>
              <pre>{formatOutput(output.content)}</pre>
            </div>
          {/each}

          {#if agent.isProcessing}
            <TypingIndicator />
          {/if}

          {#if outputs.length === 0}
            <div class="empty-output">
              <div class="empty-icon">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                  <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
                </svg>
              </div>
              <p class="empty-title">Ready for input</p>
              <p class="empty-hint">Send a prompt to start the conversation</p>
            </div>
          {/if}
        </div>

        <div class="input-area">
          <div class="input-wrapper">
            <textarea
              bind:value={promptInput}
              placeholder="Type your prompt here..."
              onkeydown={handleKeydown}
              disabled={agent.status !== "running"}
            ></textarea>
            <button
              class="primary send-btn"
              onclick={sendPrompt}
              disabled={agent.status !== "running" || !promptInput.trim()}
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="22" y1="2" x2="11" y2="13"/>
                <polygon points="22,2 15,22 11,13 2,9"/>
              </svg>
              Send
            </button>
          </div>
        </div>
      </div>

      {#if showStats}
        <AgentStats agentId={effectiveAgentId} />
      {/if}

      {#if showTools}
        <ToolActivity />
      {/if}
    </div>

    {#if showExportDialog && effectiveAgentId}
      <ExportDialog
        outputs={filteredOutputs}
        agentId={effectiveAgentId}
        onClose={() => showExportDialog = false}
      />
    {/if}
  </main>
{:else}
  <main class="agent-view empty">
    <div class="placeholder">
      <div class="placeholder-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <circle cx="12" cy="12" r="10"/>
          <path d="M8 14s1.5 2 4 2 4-2 4-2"/>
          <line x1="9" y1="9" x2="9.01" y2="9"/>
          <line x1="15" y1="9" x2="15.01" y2="9"/>
        </svg>
      </div>
      <h2>Select an Agent</h2>
      <p>Choose an agent from the sidebar to view its output and send prompts</p>
    </div>
  </main>
{/if}

<style>
  .agent-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    background-color: var(--bg-primary);
  }

  .agent-view.empty {
    align-items: center;
    justify-content: center;
  }

  .placeholder {
    text-align: center;
    color: var(--text-secondary);
    padding: var(--space-xl);
  }

  .placeholder-icon {
    width: 100px;
    height: 100px;
    border-radius: 32px;
    background: linear-gradient(135deg, var(--bg-secondary) 0%, var(--bg-tertiary) 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    margin: 0 auto var(--space-lg);
    border: 1px solid var(--border);
  }

  .placeholder-icon svg {
    width: 50px;
    height: 50px;
    color: var(--text-muted);
  }

  .placeholder h2 {
    font-size: 24px;
    margin-bottom: var(--space-sm);
    color: var(--text-primary);
  }

  .placeholder p {
    font-size: 16px;
    color: var(--text-muted);
  }

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

  .status {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    border-radius: 20px;
    font-size: 14px;
    font-weight: 600;
    flex-shrink: 0;
  }

  .status-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
  }

  .status.running {
    background-color: var(--success-glow);
    color: var(--success);
  }

  .status.running .status-dot {
    background-color: var(--success);
    animation: pulse 2s ease-in-out infinite;
  }

  .status.stopped {
    background-color: rgba(160, 160, 160, 0.15);
    color: var(--text-muted);
  }

  .status.stopped .status-dot {
    background-color: var(--text-muted);
  }

  .status.error {
    background-color: var(--error-glow);
    color: var(--error);
  }

  .status.error .status-dot {
    background-color: var(--error);
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

  .actions button.danger {
    background-color: var(--error-glow);
    color: var(--error);
    border: 1px solid var(--error);
  }

  .actions button.danger:hover {
    background-color: var(--error);
    color: white;
  }

  .content {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .output-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-width: 0;
  }

  .output-panel.with-tools {
    flex: 2;
  }

  .output-panel.with-stats {
    flex: 2;
  }

  .output {
    flex: 1;
    padding: var(--space-lg);
    overflow-y: auto;
    font-family: 'SF Mono', 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    font-size: 14px;
    line-height: 1.6;
  }

  .output-item {
    margin-bottom: var(--space-md);
    padding: var(--space-md);
    border-radius: 12px;
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
  }

  .output-item.error {
    background-color: var(--error-glow);
    border-color: var(--error);
  }

  .output-item.tool_use {
    background-color: rgba(245, 158, 11, 0.1);
    border-color: var(--warning);
  }

  .output-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-sm);
  }

  .output-type {
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--accent);
    background-color: rgba(124, 58, 237, 0.15);
    padding: 4px 10px;
    border-radius: 6px;
  }

  .output-item.error .output-type {
    color: var(--error);
    background-color: var(--error-glow);
  }

  .output-item.tool_use .output-type {
    color: var(--warning);
    background-color: rgba(245, 158, 11, 0.15);
  }

  .timestamp {
    font-size: 12px;
    color: var(--text-muted);
  }

  pre {
    white-space: pre-wrap;
    word-wrap: break-word;
    margin: 0;
    color: var(--text-primary);
  }

  .empty-output {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-xl);
  }

  .empty-output .empty-icon {
    width: 80px;
    height: 80px;
    border-radius: 24px;
    background: linear-gradient(135deg, var(--bg-secondary) 0%, var(--bg-tertiary) 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: var(--space-lg);
    border: 1px solid var(--border);
  }

  .empty-output .empty-icon svg {
    width: 40px;
    height: 40px;
    color: var(--text-muted);
  }

  .empty-output .empty-title {
    font-size: 18px;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: var(--space-sm);
  }

  .empty-output .empty-hint {
    font-size: 14px;
    color: var(--text-muted);
  }

  .input-area {
    padding: var(--space-lg);
    border-top: 1px solid var(--border);
    background-color: var(--bg-secondary);
  }

  .input-wrapper {
    display: flex;
    gap: var(--space-md);
    align-items: flex-end;
  }

  textarea {
    flex: 1;
    resize: none;
    min-height: 80px;
    max-height: 160px;
    font-family: inherit;
    border-radius: 16px;
    padding: var(--space-md) var(--space-lg);
  }

  .send-btn {
    padding: 16px 28px;
    flex-shrink: 0;
    height: fit-content;
  }

  .send-btn svg {
    width: 20px;
    height: 20px;
  }

  .send-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; transform: scale(1); }
    50% { opacity: 0.5; transform: scale(1.2); }
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
