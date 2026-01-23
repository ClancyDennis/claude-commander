<script lang="ts" module>
  import type { Component } from "svelte";

  export interface PanelToggleItem {
    id: string;
    label: string;
    icon: Component;
  }

  export type PanelToggleBarProps = {
    items: PanelToggleItem[];
    active: string | null;
    onToggle: (id: string) => void;
    class?: string;
  };
</script>

<script lang="ts">
  import { cn } from "$lib/utils";

  let { items, active, onToggle, class: className }: PanelToggleBarProps = $props();
</script>

<div class={cn("panel-toggle-bar", className)}>
  {#each items as item}
    {@const Icon = item.icon}
    <button
      class={cn("toggle-btn", { active: active === item.id })}
      onclick={() => onToggle(item.id)}
      title={item.label}
    >
      <Icon size={18} />
      <span class="toggle-label">{item.label}</span>
    </button>
  {/each}
</div>

<style>
  .panel-toggle-bar {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: 4px;
    background: var(--bg-tertiary);
    border-radius: 10px;
    border: 1px solid var(--border-hex);
  }

  .toggle-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 12px;
    background: transparent;
    border: none;
    border-radius: 8px;
    color: var(--text-secondary);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .toggle-btn:hover {
    background: var(--bg-elevated);
    color: var(--text-primary);
  }

  .toggle-btn.active {
    background: linear-gradient(135deg, var(--accent-hex) 0%, #e85a45 100%);
    color: white;
    box-shadow: 0 2px 8px var(--accent-glow);
  }

  .toggle-label {
    display: none;
  }

  @media (min-width: 768px) {
    .toggle-label {
      display: inline;
    }
  }
</style>
