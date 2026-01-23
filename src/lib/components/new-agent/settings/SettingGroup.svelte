<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    title,
    hasToggle = false,
    enabled = $bindable(true),
    isCreating = false,
    children,
  }: {
    title: string;
    hasToggle?: boolean;
    enabled?: boolean;
    isCreating?: boolean;
    children?: Snippet;
  } = $props();
</script>

<div class="setting-group">
  <div class="group-header">
    <span>{title}</span>
    {#if hasToggle}
      <label class="toggle-small">
        <input type="checkbox" bind:checked={enabled} disabled={isCreating} />
        <span class="slider-small"></span>
      </label>
    {/if}
  </div>
  {#if !hasToggle || enabled}
    {@render children?.()}
  {/if}
</div>

<style>
  .setting-group {
    margin-bottom: var(--space-3);
    padding-bottom: var(--space-3);
    border-bottom: 1px solid var(--border-hex);
  }

  .setting-group:last-child {
    margin-bottom: 0;
    padding-bottom: 0;
    border-bottom: none;
  }

  .group-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-2);
    font-size: var(--text-sm);
    font-weight: var(--font-semibold);
    color: var(--accent-hex);
  }

  .toggle-small {
    position: relative;
    display: inline-block;
    width: 36px;
    height: 20px;
  }

  .toggle-small input {
    opacity: 0;
    width: 0;
    height: 0;
  }

  .slider-small {
    position: absolute;
    cursor: pointer;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: var(--border-hex);
    transition: all var(--transition-fast);
    border-radius: var(--radius-full);
  }

  .slider-small:before {
    position: absolute;
    content: "";
    height: 14px;
    width: 14px;
    left: 3px;
    bottom: 3px;
    background-color: white;
    transition: all var(--transition-fast);
    border-radius: var(--radius-full);
  }

  input:checked + .slider-small {
    background-color: var(--accent-hex);
  }

  input:checked + .slider-small:before {
    transform: translateX(16px);
  }
</style>
