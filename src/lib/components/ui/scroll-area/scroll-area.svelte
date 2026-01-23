<script lang="ts" module>
  import type { ScrollArea as ScrollAreaPrimitive } from "bits-ui";

  export type ScrollbarVisibility = "auto" | "always" | "hover";
  export type ScrollOrientation = "vertical" | "horizontal" | "both";

  export type ScrollAreaProps = ScrollAreaPrimitive.RootProps & {
    class?: string;
    orientation?: ScrollOrientation;
    scrollbarVisibility?: ScrollbarVisibility;
  };
</script>

<script lang="ts">
  import { ScrollArea as ScrollAreaPrimitive } from "bits-ui";
  import { cn } from "$lib/utils";
  import type { Snippet } from "svelte";

  let {
    class: className,
    orientation = "vertical",
    scrollbarVisibility = "auto",
    children,
    ...restProps
  }: ScrollAreaProps & { children?: Snippet } = $props();

  const showVertical = $derived(orientation === "vertical" || orientation === "both");
  const showHorizontal = $derived(orientation === "horizontal" || orientation === "both");

  const scrollbarBaseClasses = "flex touch-none select-none transition-colors";
  const thumbClasses = "relative flex-1 rounded-full bg-border hover:bg-muted-foreground/50 transition-colors";

  const visibilityClasses = $derived({
    auto: "opacity-0 group-hover/scroll:opacity-100 data-[state=visible]:opacity-100",
    always: "opacity-100",
    hover: "opacity-0 group-hover/scroll:opacity-100",
  }[scrollbarVisibility]);
</script>

<ScrollAreaPrimitive.Root
  class={cn("relative overflow-hidden group/scroll", className)}
  {...restProps}
>
  <ScrollAreaPrimitive.Viewport class="h-full w-full rounded-[inherit]">
    {#if children}
      {@render children()}
    {/if}
  </ScrollAreaPrimitive.Viewport>

  {#if showVertical}
    <ScrollAreaPrimitive.Scrollbar
      orientation="vertical"
      class={cn(
        scrollbarBaseClasses,
        visibilityClasses,
        "h-full w-2.5 border-l border-l-transparent p-[1px]"
      )}
    >
      <ScrollAreaPrimitive.Thumb class={thumbClasses} />
    </ScrollAreaPrimitive.Scrollbar>
  {/if}

  {#if showHorizontal}
    <ScrollAreaPrimitive.Scrollbar
      orientation="horizontal"
      class={cn(
        scrollbarBaseClasses,
        visibilityClasses,
        "h-2.5 flex-col border-t border-t-transparent p-[1px]"
      )}
    >
      <ScrollAreaPrimitive.Thumb class={thumbClasses} />
    </ScrollAreaPrimitive.Scrollbar>
  {/if}

  <ScrollAreaPrimitive.Corner />
</ScrollAreaPrimitive.Root>
