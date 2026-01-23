<script lang="ts">
  import { Dialog as DialogPrimitive } from "bits-ui";
  import { cn } from "$lib/utils";
  import type { Snippet } from "svelte";

  let {
    class: className,
    children,
    ...restProps
  }: DialogPrimitive.TitleProps & {
    class?: string;
    children?: Snippet;
  } = $props();
</script>

<DialogPrimitive.Title
  class={cn("text-lg font-semibold text-foreground", className)}
  {...restProps}
>
  {#if children}
    {@render children()}
  {/if}
</DialogPrimitive.Title>
