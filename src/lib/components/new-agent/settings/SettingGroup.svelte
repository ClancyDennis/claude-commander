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
    margin-bottom: 12px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border);
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
    margin-bottom: 8px;
    font-size: 13px;
    font-weight: 600;
    color: var(--accent);
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
    background-color: var(--border);
    transition: 0.3s;
    border-radius: 20px;
  }

  .slider-small:before {
    position: absolute;
    content: "";
    height: 14px;
    width: 14px;
    left: 3px;
    bottom: 3px;
    background-color: white;
    transition: 0.3s;
    border-radius: 50%;
  }

  input:checked + .slider-small {
    background-color: var(--accent);
  }

  input:checked + .slider-small:before {
    transform: translateX(16px);
  }
</style>
