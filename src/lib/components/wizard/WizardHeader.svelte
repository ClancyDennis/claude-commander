<script lang="ts">
  import type { Snippet, Component } from "svelte";

  interface Props {
    title: string;
    description?: string;
    icon?: Component;
    iconClass?: string;
    centered?: boolean;
    children?: Snippet;
  }

  let {
    title,
    description = "",
    icon: Icon,
    iconClass = "w-12 h-12 rounded-lg bg-primary/10 flex items-center justify-center",
    centered = true,
    children,
  }: Props = $props();
</script>

<div class="mb-6 {centered ? 'text-center' : ''}">
  {#if Icon}
    <div class="{centered ? 'flex justify-center mb-4' : 'mb-4'}">
      <div class={iconClass}>
        <svelte:component this={Icon} class="w-6 h-6 text-primary" />
      </div>
    </div>
  {/if}

  <div class="space-y-1.5 {centered ? 'flex flex-col items-center' : ''}">
    <h3 class="text-xl font-semibold leading-none tracking-tight">{title}</h3>
    {#if description}
      <p class="text-base text-muted-foreground mt-2">
        {description}
      </p>
    {/if}
  </div>

  {#if children}
    {@render children()}
  {/if}
</div>
