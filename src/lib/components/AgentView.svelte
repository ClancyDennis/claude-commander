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
  import AgentSettings from "./AgentSettings.svelte";
  import WorkingFiles from "./WorkingFiles.svelte";
  import TaskProgress from "./TaskProgress.svelte";
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

  // Side panel state
  let activeSidePanel = $state<"none" | "tools" | "stats" | "settings" | "files" | "progress">("none");
  let showExportDialog = $state(false);

  // Filtered outputs managed by OutputControls
  let filteredOutputs = $state<AgentOutput[]>([]);

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
  $effect(() => {
    const currentAgentId = effectiveAgentId;
    const currentOutputs = outputs;

    // Reset filter on agent switch
    if (currentAgentId !== filterState.previousAgentId) {
      filterState.previousAgentId = currentAgentId;
      filterState.hasReceivedFilter = false;
      filteredOutputs = currentOutputs;
    } else if (!filterState.hasReceivedFilter && currentOutputs.length > 0) {
      // Initialize filtered outputs with all outputs when first loaded
      filteredOutputs = currentOutputs;
    }
  });

  function toggleSidePanel(panel: "tools" | "stats" | "settings" | "files" | "progress") {
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
  <main class="agent-view" class:compact>
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
            onFilter={(filtered) => {
              filterState.hasReceivedFilter = true;
              filteredOutputs = filtered;
            }}
            onExport={() => showExportDialog = true}
          />
        {/if}
        
        <AgentOutputList
          outputs={filterState.hasReceivedFilter ? filteredOutputs : outputs}
          hasAnyOutput={outputs.length > 0}
          isProcessing={agent.isProcessing}
          onClearFilter={() => { filterState.hasReceivedFilter = false; filteredOutputs = outputs; }}
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
          {:else if activeSidePanel === 'settings' && effectiveAgentId}
            <AgentSettings agentId={effectiveAgentId} onClose={() => activeSidePanel = "none"} />
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

