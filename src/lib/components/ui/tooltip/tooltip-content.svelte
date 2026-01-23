<script lang="ts">
  import { Tooltip as TooltipPrimitive } from "bits-ui";
  import { cn } from "$lib/utils";
  import type { Snippet } from "svelte";

  let {
    class: className,
    sideOffset = 4,
    children,
    ...restProps
  }: TooltipPrimitive.ContentProps & {
    class?: string;
    children?: Snippet;
  } = $props();
</script>

<TooltipPrimitive.Portal>
  <TooltipPrimitive.Content
    {sideOffset}
    class={cn(
      "z-50 overflow-hidden rounded-md bg-popover px-3 py-1.5 text-sm text-popover-foreground shadow-md animate-fade-in",
      className
    )}
    {...restProps}
  >
    {#if children}
      {@render children()}
    {/if}
  </TooltipPrimitive.Content>
</TooltipPrimitive.Portal>
