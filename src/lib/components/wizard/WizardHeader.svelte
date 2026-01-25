<script lang="ts">
  import type { Snippet, Component } from "svelte";
  import * as Dialog from "$lib/components/ui/dialog";

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

  <Dialog.Header class={centered ? "items-center" : ""}>
    <Dialog.Title class="text-xl">{title}</Dialog.Title>
    {#if description}
      <Dialog.Description class="text-base mt-2">
        {description}
      </Dialog.Description>
    {/if}
  </Dialog.Header>

  {#if children}
    {@render children()}
  {/if}
</div>
