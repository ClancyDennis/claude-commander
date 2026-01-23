<script lang="ts" module>
  import type { Component } from "svelte";

  export type ViewHeaderProps = {
    icon?: Component;
    emojiIcon?: string;
    title: string;
    subtitle?: string;
    class?: string;
  };
</script>

<script lang="ts">
  import { cn } from "$lib/utils";
  import type { Snippet } from "svelte";

  let {
    icon,
    emojiIcon,
    title,
    subtitle,
    class: className,
    children,
    status,
    actions,
  }: ViewHeaderProps & {
    children?: Snippet;
    status?: Snippet;
    actions?: Snippet;
  } = $props();
</script>

<header class={cn("view-header", className)}>
  <div class="header-left">
    <div class="header-icon">
      {#if icon}
        {@const Icon = icon}
        <Icon size={24} />
      {:else if emojiIcon}
        <span class="emoji-icon">{emojiIcon}</span>
      {/if}
    </div>
    <div class="header-info">
      <div class="title-row">
        <h2 class="header-title">{title}</h2>
        {#if children}
          {@render children()}
        {/if}
      </div>
      {#if subtitle}
        <span class="header-subtitle" title={subtitle}>{subtitle}</span>
      {/if}
    </div>
    {#if status}
      <div class="header-status">
        {@render status()}
      </div>
    {/if}
  </div>
  {#if actions}
    <div class="header-actions">
      {@render actions()}
    </div>
  {/if}
</header>

<style>
  .view-header {
    padding: var(--space-lg);
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--border-hex);
    background: linear-gradient(180deg, var(--bg-secondary) 0%, var(--bg-primary) 100%);
    gap: var(--space-md);
    flex-wrap: wrap;
    flex-shrink: 0;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    min-width: 0;
    flex: 1;
  }

  .header-icon {
    width: 48px;
    height: 48px;
    border-radius: 14px;
    background: linear-gradient(135deg, var(--accent-hex) 0%, #e85a45 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 4px 12px var(--accent-glow);
    color: white;
    flex-shrink: 0;
  }

  .emoji-icon {
    font-size: 24px;
    line-height: 1;
  }

  .header-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    flex: 1;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    flex-wrap: wrap;
  }

  .header-title {
    font-size: 20px;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
  }

  .header-subtitle {
    font-size: 13px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .header-status {
    flex-shrink: 0;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    flex-shrink: 0;
    flex-wrap: wrap;
  }
</style>
