<script lang="ts">
  import { Dialog as DialogPrimitive } from "bits-ui";
  import { cn } from "$lib/utils";
  import type { Snippet } from "svelte";

  let {
    class: className,
    children,
    ...restProps
  }: DialogPrimitive.DescriptionProps & {
    class?: string;
    children?: Snippet;
  } = $props();
</script>

<DialogPrimitive.Description
  class={cn("text-sm text-muted-foreground", className)}
  {...restProps}
>
  {#if children}
    {@render children()}
  {/if}
</DialogPrimitive.Description>
