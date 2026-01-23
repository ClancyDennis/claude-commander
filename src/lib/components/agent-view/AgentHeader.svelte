<script lang="ts">
  import type { Agent } from "$lib/types";
  import { ViewHeader, PanelToggleBar, type PanelToggleItem } from "$lib/components/ui/layout";
  import { IconButton } from "$lib/components/ui/button";
  import { LayoutGrid, BarChart2, Wrench, FileText, CheckSquare, Trash2, Square, Github } from "$lib/components/ui/icons";
  import StatusBadge from "../StatusBadge.svelte";

  let {
    agent,
    activeSidePanel,
    onToggleSidePanel,
    onClear,
    onStop
  }: {
    agent: Agent;
    activeSidePanel: "none" | "tools" | "stats" | "files" | "progress";
    onToggleSidePanel: (panel: "tools" | "stats" | "files" | "progress") => void;
    onClear: () => void;
    onStop: () => void;
  } = $props();

  const panelItems: PanelToggleItem[] = [
    { id: "stats", label: "Stats", icon: BarChart2 },
    { id: "tools", label: "Tools", icon: Wrench },
    { id: "files", label: "Files", icon: FileText },
    { id: "progress", label: "Progress", icon: CheckSquare },
  ];

  function handleToggle(id: string) {
    onToggleSidePanel(id as "tools" | "stats" | "files" | "progress");
  }

  const directoryName = $derived(agent.workingDir.split("/").pop() || "Agent");
</script>

<ViewHeader
  icon={LayoutGrid}
  title={directoryName}
  subtitle={agent.workingDir}
>
  {#snippet children()}
    {#if agent.githubContext}
      <a
        href={agent.githubContext.repositoryUrl}
        target="_blank"
        rel="noopener noreferrer"
        class="github-badge"
        title="Open on GitHub"
      >
        <Github size={12} />
        {agent.githubContext.owner}/{agent.githubContext.repo}
        <span class="branch">{agent.githubContext.branch}</span>
      </a>
    {/if}
  {/snippet}
  {#snippet status()}
    <StatusBadge status={agent.status} />
  {/snippet}
  {#snippet actions()}
    <PanelToggleBar
      items={panelItems}
      active={activeSidePanel === "none" ? null : activeSidePanel}
      onToggle={handleToggle}
    />
    <IconButton
      icon={Trash2}
      label="Clear"
      variant="ghost"
      onclick={onClear}
    />
    {#if agent.status === "running"}
      <IconButton
        icon={Square}
        label="Stop"
        variant="danger"
        onclick={onStop}
      />
    {/if}
  {/snippet}
</ViewHeader>

<style>
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

  .github-badge .branch {
    padding: 1px 4px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
    font-family: 'SF Mono', 'Monaco', 'Menlo', monospace;
  }
</style>
