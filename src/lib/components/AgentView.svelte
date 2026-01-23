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
  import AgentHeader from "./agent-view/AgentHeader.svelte";
  import AgentInput from "./agent-view/AgentInput.svelte";
  import AgentOutputList from "./agent-view/AgentOutputList.svelte";
  import ToolActivity from "./ToolActivity.svelte";
  import AgentStats from "./AgentStats.svelte";
  import OutputControls from "./OutputControls.svelte";
  import ExportDialog from "./ExportDialog.svelte";
  import WorkingFiles from "./WorkingFiles.svelte";
  import TaskProgress from "./TaskProgress.svelte";
  import type { AgentOutput } from "../types";

  const appIcon = new URL("../assets/claude-commander-icon.png", import.meta.url).href;

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

  // Side panel state
  let activeSidePanel = $state<"none" | "tools" | "stats" | "files" | "progress">("none");
  let showExportDialog = $state(false);

  // Filtered outputs managed by OutputControls
  let filteredOutputs = $state<AgentOutput[]>([]);

  // Default filter type (must match OutputControls default)
  const DEFAULT_FILTER_TYPE = "text";

  // Use a plain object for tracking state to avoid reactive loops in Svelte 5
  // Plain objects are not reactive, so modifying them in $effect won't trigger re-runs
  const filterState = {
    hasReceivedFilter: false,
    previousAgentId: undefined as string | undefined
  };

  // Mark agent as viewed when it's displayed
  $effect(() => {
    if (effectiveAgentId) {
      markAgentViewed(effectiveAgentId);
    }
  });

  // Handle agent switches and filter initialization
  // Apply filter IMMEDIATELY to avoid flash of unfiltered content
  $effect(() => {
    const currentAgentId = effectiveAgentId;
    const currentOutputs = outputs;

    // Reset filter on agent switch - apply default filter synchronously
    if (currentAgentId !== filterState.previousAgentId) {
      filterState.previousAgentId = currentAgentId;
      filterState.hasReceivedFilter = true; // Mark as received immediately
      // Apply default filter (type="text") synchronously to avoid flash
      filteredOutputs = currentOutputs.filter(o => o.type === DEFAULT_FILTER_TYPE);
    }
  });

  function toggleSidePanel(panel: "tools" | "stats" | "files" | "progress") {
    if (activeSidePanel === panel) {
      activeSidePanel = "none";
    } else {
      activeSidePanel = panel;
    }
  }

  async function sendPrompt(text: string) {
    if (!effectiveAgentId || !text.trim()) return;

    try {
      await invoke("send_prompt", {
        agentId: effectiveAgentId,
        prompt: text,
      });
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
</script>

{#if agent}
  <main class="agent-view" class:compact data-tutorial="agent-view">
    {#if showHeader}
      <AgentHeader 
        {agent} 
        {activeSidePanel} 
        onToggleSidePanel={toggleSidePanel}
        onClear={() => clearAgentOutput(effectiveAgentId!)}
        onStop={stopAgent}
      />
    {/if}

    <div class="content">
      <div class="output-panel" class:sidebar-open={activeSidePanel !== 'none'}>
        {#if outputs.length > 0}
          <OutputControls
            outputs={outputs}
            initialFilterType={DEFAULT_FILTER_TYPE}
            onFilter={(filtered) => {
              filteredOutputs = filtered;
            }}
            onExport={() => showExportDialog = true}
          />
        {/if}

        <AgentOutputList
          outputs={filteredOutputs}
          hasAnyOutput={outputs.length > 0}
          isProcessing={agent.isProcessing}
          onClearFilter={() => { filteredOutputs = outputs.filter(o => o.type === DEFAULT_FILTER_TYPE); }}
        />

        <AgentInput 
          status={agent.status} 
          onSend={sendPrompt} 
        />
      </div>

      {#if activeSidePanel !== 'none'}
        <div class="side-panel">
          {#if activeSidePanel === 'stats'}
            <AgentStats agentId={effectiveAgentId} />
          {:else if activeSidePanel === 'tools'}
            <ToolActivity />
          {:else if activeSidePanel === 'files'}
            <WorkingFiles />
          {:else if activeSidePanel === 'progress'}
            <TaskProgress />
          {/if}
        </div>
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
        <img src={appIcon} alt="Claude Commander" />
      </div>
      <h2>Select an Agent</h2>
      <p>Choose an agent from the sidebar, or press <strong>New</strong> to create one</p>
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

  .placeholder-icon img {
    width: 80px;
    height: 80px;
    object-fit: contain;
  }

  .placeholder p strong {
    color: var(--accent);
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
    transition: flex 0.2s ease;
  }
  
  .side-panel {
    width: min(400px, 45%);
    min-width: 250px;
    flex-shrink: 0;
    background-color: var(--bg-secondary);
    border-left: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    animation: slideLeft 0.2s ease;
  }

  @keyframes slideLeft {
    from { transform: translateX(20px); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
  }
</style>

