<script lang="ts">
  import { Dialog as DialogPrimitive } from "bits-ui";
  import { cn } from "$lib/utils";
  import type { Snippet } from "svelte";
  import { X } from "lucide-svelte";

  let {
    class: className,
    children,
    ...restProps
  }: DialogPrimitive.ContentProps & {
    class?: string;
    children?: Snippet;
  } = $props();
</script>

<DialogPrimitive.Portal>
  <DialogPrimitive.Overlay
    class="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm animate-fade-in"
  />
  <DialogPrimitive.Content
    class={cn(
      "fixed left-1/2 top-1/2 z-50 grid w-full max-w-lg -translate-x-1/2 -translate-y-1/2 gap-4 border border-border bg-background p-6 shadow-lg animate-scale-in rounded-lg",
      "focus:outline-none",
      className
    )}
    {...restProps}
  >
    {#if children}
      {@render children()}
    {/if}
    <DialogPrimitive.Close
      class="absolute right-4 top-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none"
    >
      <X class="h-4 w-4" />
      <span class="sr-only">Close</span>
    </DialogPrimitive.Close>
  </DialogPrimitive.Content>
</DialogPrimitive.Portal>
