<script lang="ts">
  import type { Component, Snippet } from 'svelte';
  import { ChevronRight } from '$lib/components/ui/icons';

  let {
    title,
    subtitle,
    leadingIcon,
    leading,
    trailing,
    badge,
    selected = false,
    showDisclosure = true,
    compact = false,
    class: className = '',
    onclick
  }: {
    title: string;
    subtitle?: string;
    leadingIcon?: Component;
    leading?: Snippet;
    trailing?: Snippet;
    badge?: Snippet;
    selected?: boolean;
    showDisclosure?: boolean;
    compact?: boolean;
    class?: string;
    onclick?: () => void;
  } = $props();
</script>

<button
  type="button"
  class="list-item {className}"
  class:selected
  class:compact
  onclick={onclick}
>
  {#if leading}
    <div class="leading">
      {@render leading()}
    </div>
  {:else if leadingIcon}
    <div class="leading-icon">
      <svelte:component this={leadingIcon} size={compact ? 16 : 18} />
    </div>
  {/if}

  <div class="content">
    <div class="title-row">
      <span class="title">{title}</span>
      {#if badge}
        <div class="badge-slot">
          {@render badge()}
        </div>
      {/if}
    </div>
    {#if subtitle}
      <span class="subtitle">{subtitle}</span>
    {/if}
  </div>

  {#if trailing}
    <div class="trailing">
      {@render trailing()}
    </div>
  {/if}

  {#if showDisclosure}
    <ChevronRight size={16} class="disclosure" />
  {/if}
</button>

<style>
  .list-item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    background: transparent;
    border: none;
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: background var(--transition-fast);
    text-align: left;
    color: inherit;
  }

  .list-item.compact {
    padding: var(--space-2) var(--space-3);
    gap: var(--space-2);
  }

  .list-item:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .list-item:active {
    background: rgba(255, 255, 255, 0.08);
  }

  .list-item.selected {
    background: rgba(232, 102, 77, 0.12);
  }

  .list-item.selected:hover {
    background: rgba(232, 102, 77, 0.18);
  }

  .leading {
    flex-shrink: 0;
  }

  .leading-icon {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-tertiary);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .compact .leading-icon {
    width: 24px;
    height: 24px;
  }

  .content {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .title {
    font-size: var(--text-base);
    font-weight: var(--font-medium);
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .compact .title {
    font-size: var(--text-sm);
  }

  .badge-slot {
    flex-shrink: 0;
  }

  .subtitle {
    font-size: var(--text-sm);
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .compact .subtitle {
    font-size: var(--text-xs);
  }

  .trailing {
    flex-shrink: 0;
    display: flex;
    align-items: center;
  }

  .list-item :global(.disclosure) {
    color: var(--text-muted);
    flex-shrink: 0;
    opacity: 0.6;
    transition: opacity var(--transition-fast);
  }

  .list-item:hover :global(.disclosure) {
    opacity: 1;
  }

  .list-item.selected :global(.disclosure) {
    color: var(--accent-hex);
    opacity: 1;
  }
</style>
