<script lang="ts">
  /**
   * ViewRouter Component
   *
   * Centralized routing logic for the main content area.
   * Determines which view to display based on application state.
   */

  // View components
  import DatabaseStats from "./DatabaseStats.svelte";
  import InstructionManagementPanel from "./InstructionManagementPanel.svelte";
  import CommanderSettings from "./CommanderSettings.svelte";
  import { Settings } from "./settings";
  import AutoPipelineView from "./AutoPipelineView.svelte";
  import PhaseProgress from "./PhaseProgress.svelte";
  import HistoricalRunView from "./HistoricalRunView.svelte";
  import ChatView from "./ChatView.svelte";
  import LayoutManager from "./LayoutManager.svelte";
  import AgentView from "./AgentView.svelte";
  import SplitView from "./SplitView.svelte";
  import GridView from "./GridView.svelte";

  // Stores for routing decisions
  import { selectedAutoPipelineId } from "$lib/stores/autoPipelines";
  import { selectedPipelineId } from "$lib/stores/pipelines";
  import {
    sidebarMode,
    selectedHistoricalRun,
    viewMode,
    layoutMode,
  } from "$lib/stores/agents";

  // Props for local state from parent
  interface Props {
    showDatabaseStats: boolean;
    showInstructionPanel: boolean;
    showSettings: boolean;
    showCommanderSettings: boolean;
    onCloseInstructions: () => void;
    onCloseSettings: () => void;
    onCloseCommanderSettings: () => void;
  }

  let {
    showDatabaseStats,
    showInstructionPanel,
    showSettings,
    showCommanderSettings,
    onCloseInstructions,
    onCloseSettings,
    onCloseCommanderSettings,
  }: Props = $props();

  // View routing types
  type ViewType =
    | "database-stats"
    | "instructions"
    | "settings"
    | "commander-settings"
    | "auto-pipeline"
    | "pipeline"
    | "historical-run"
    | "chat"
    | "agent";

  // Derive the current view based on state priority
  const currentView = $derived.by((): ViewType => {
    // Database stats can appear alongside other views (rendered separately)
    // Instruction panel takes full priority
    if (showInstructionPanel) return "instructions";
    // Commander settings view
    if (showCommanderSettings) return "commander-settings";
    // Settings view
    if (showSettings) return "settings";
    // Auto-pipeline view (orchestrator)
    if ($selectedAutoPipelineId) return "auto-pipeline";
    // Regular pipeline view
    if ($selectedPipelineId) return "pipeline";
    // Historical run view (history mode with selection)
    if ($sidebarMode === "history" && $selectedHistoricalRun) return "historical-run";
    // Chat view (meta-agent)
    if ($viewMode === "chat") return "chat";
    // Default: agent view
    return "agent";
  });
</script>

<!-- Database stats panel (shows above other content when enabled) -->
{#if showDatabaseStats}
  <div class="database-stats-container">
    <DatabaseStats />
  </div>
{/if}

<!-- Main view content based on routing -->
{#if currentView === "instructions"}
  <div class="instruction-panel-container">
    <InstructionManagementPanel onClose={onCloseInstructions} />
  </div>
{:else if currentView === "commander-settings"}
  <div class="settings-container">
    <CommanderSettings onClose={onCloseCommanderSettings} />
  </div>
{:else if currentView === "settings"}
  <div class="settings-container">
    <Settings onClose={onCloseSettings} />
  </div>
{:else if currentView === "auto-pipeline"}
  <div class="pipeline-view-container">
    <AutoPipelineView pipelineId={$selectedAutoPipelineId!} />
  </div>
{:else if currentView === "pipeline"}
  <div class="pipeline-view-container">
    <PhaseProgress pipelineId={$selectedPipelineId!} />
  </div>
{:else if currentView === "historical-run"}
  <HistoricalRunView />
{:else if currentView === "chat"}
  <ChatView />
{:else}
  <!-- Agent view with layout manager -->
  <LayoutManager />
  {#if $layoutMode === "single"}
    <AgentView />
  {:else if $layoutMode === "split"}
    <SplitView direction="horizontal" />
  {:else if $layoutMode === "grid"}
    <GridView />
  {/if}
{/if}

<style>
  .database-stats-container {
    padding: var(--space-lg);
    border-bottom: 1px solid var(--border);
  }

  .pipeline-view-container {
    flex: 1;
    overflow: auto;
    padding: var(--space-lg);
  }

  .settings-container {
    flex: 1;
    overflow: auto;
    background: var(--bg-primary);
  }

  .instruction-panel-container {
    flex: 1;
    overflow: auto;
    background: var(--bg-primary);
  }
</style>
