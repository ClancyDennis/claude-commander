<script lang="ts">
  import type { Component } from 'svelte';

  type Segment = {
    id: string;
    label: string;
    icon?: Component;
  };

  let {
    segments,
    selected,
    onSelect,
    size = 'md',
    class: className = ''
  }: {
    segments: Segment[];
    selected: string;
    onSelect: (id: string) => void;
    size?: 'sm' | 'md' | 'lg';
    class?: string;
  } = $props();

  function handleClick(id: string) {
    if (id !== selected) {
      onSelect(id);
    }
  }
</script>

<div class="segmented-control size-{size} {className}" role="tablist">
  {#each segments as segment (segment.id)}
    <button
      type="button"
      class="segment"
      class:active={selected === segment.id}
      role="tab"
      aria-selected={selected === segment.id}
      onclick={() => handleClick(segment.id)}
    >
      {#if segment.icon}
        <svelte:component this={segment.icon} size={size === 'sm' ? 12 : size === 'lg' ? 16 : 14} />
      {/if}
      <span class="label">{segment.label}</span>
    </button>
  {/each}
  <div class="indicator" style="--segment-count: {segments.length}; --active-index: {segments.findIndex(s => s.id === selected)};"></div>
</div>

<style>
  .segmented-control {
    display: inline-flex;
    position: relative;
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
    padding: 2px;
  }

  /* Sizes */
  .size-sm {
    border-radius: var(--radius-sm);
  }

  .size-sm .segment {
    padding: 4px 10px;
    font-size: var(--text-xs);
    gap: 4px;
  }

  .size-md .segment {
    padding: 6px 14px;
    font-size: var(--text-sm);
    gap: 6px;
  }

  .size-lg .segment {
    padding: 8px 18px;
    font-size: var(--text-base);
    gap: 8px;
  }

  .segment {
    position: relative;
    z-index: 1;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-weight: var(--font-medium);
    color: var(--text-secondary);
    background: transparent;
    border: none;
    border-radius: calc(var(--radius-md) - 2px);
    cursor: pointer;
    transition: color var(--transition-normal);
    white-space: nowrap;
  }

  .size-sm .segment {
    border-radius: calc(var(--radius-sm) - 2px);
  }

  .segment:hover:not(.active) {
    color: var(--text-primary);
  }

  .segment.active {
    color: var(--text-primary);
  }

  .label {
    line-height: 1;
  }

  /* Animated indicator */
  .indicator {
    position: absolute;
    top: 2px;
    bottom: 2px;
    left: 2px;
    width: calc((100% - 4px) / var(--segment-count));
    background: var(--bg-elevated);
    border-radius: calc(var(--radius-md) - 2px);
    box-shadow: var(--shadow-sm);
    transition: transform 0.2s var(--spring-bounce);
    transform: translateX(calc(var(--active-index) * 100%));
    pointer-events: none;
  }

  .size-sm .indicator {
    border-radius: calc(var(--radius-sm) - 2px);
  }
</style>
