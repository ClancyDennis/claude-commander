<script lang="ts" module>
  import { tv, type VariantProps } from "tailwind-variants";

  export const sheetVariants = tv({
    base: "fixed z-50 gap-4 bg-background p-6 shadow-lg transition-transform duration-300 ease-in-out",
    variants: {
      side: {
        top: "inset-x-0 top-0 border-b border-border",
        bottom: "inset-x-0 bottom-0 border-t border-border",
        left: "inset-y-0 left-0 h-full w-3/4 border-r border-border sm:max-w-sm",
        right: "inset-y-0 right-0 h-full w-3/4 border-l border-border sm:max-w-sm",
      },
    },
    defaultVariants: {
      side: "right",
    },
  });

  export type SheetSide = VariantProps<typeof sheetVariants>["side"];
</script>

<script lang="ts">
  import { Dialog as DialogPrimitive } from "bits-ui";
  import { cn } from "$lib/utils";
  import type { Snippet } from "svelte";
  import { X } from "lucide-svelte";

  let {
    class: className,
    side = "right",
    children,
    ...restProps
  }: DialogPrimitive.ContentProps & {
    class?: string;
    side?: SheetSide;
    children?: Snippet;
  } = $props();

  const slideAnimations = {
    top: {
      enter: "animate-slide-in-from-top",
      exit: "animate-slide-out-to-top",
    },
    bottom: {
      enter: "animate-slide-in-from-bottom",
      exit: "animate-slide-out-to-bottom",
    },
    left: {
      enter: "animate-slide-in-from-left",
      exit: "animate-slide-out-to-left",
    },
    right: {
      enter: "animate-slide-in-from-right",
      exit: "animate-slide-out-to-right",
    },
  };

  const getCurrentSide = () => side ?? "right";
</script>

<DialogPrimitive.Portal>
  <DialogPrimitive.Overlay
    class="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm animate-fade-in data-[state=closed]:animate-fade-out"
  />
  <DialogPrimitive.Content
    class={cn(
      sheetVariants({ side: getCurrentSide() }),
      slideAnimations[getCurrentSide()].enter,
      "data-[state=closed]:" + slideAnimations[getCurrentSide()].exit,
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

<style>
  :global(.animate-slide-in-from-top) {
    animation: slide-in-from-top 0.3s ease-out;
  }

  :global(.animate-slide-out-to-top) {
    animation: slide-out-to-top 0.2s ease-in;
  }

  :global(.animate-slide-in-from-bottom) {
    animation: slide-in-from-bottom 0.3s ease-out;
  }

  :global(.animate-slide-out-to-bottom) {
    animation: slide-out-to-bottom 0.2s ease-in;
  }

  :global(.animate-slide-in-from-left) {
    animation: slide-in-from-left 0.3s ease-out;
  }

  :global(.animate-slide-out-to-left) {
    animation: slide-out-to-left 0.2s ease-in;
  }

  :global(.animate-slide-in-from-right) {
    animation: slide-in-from-right 0.3s ease-out;
  }

  :global(.animate-slide-out-to-right) {
    animation: slide-out-to-right 0.2s ease-in;
  }

  :global(.animate-fade-out) {
    animation: fade-out 0.2s ease-in;
  }

  @keyframes slide-in-from-top {
    from {
      transform: translateY(-100%);
    }
    to {
      transform: translateY(0);
    }
  }

  @keyframes slide-out-to-top {
    from {
      transform: translateY(0);
    }
    to {
      transform: translateY(-100%);
    }
  }

  @keyframes slide-in-from-bottom {
    from {
      transform: translateY(100%);
    }
    to {
      transform: translateY(0);
    }
  }

  @keyframes slide-out-to-bottom {
    from {
      transform: translateY(0);
    }
    to {
      transform: translateY(100%);
    }
  }

  @keyframes slide-in-from-left {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(0);
    }
  }

  @keyframes slide-out-to-left {
    from {
      transform: translateX(0);
    }
    to {
      transform: translateX(-100%);
    }
  }

  @keyframes slide-in-from-right {
    from {
      transform: translateX(100%);
    }
    to {
      transform: translateX(0);
    }
  }

  @keyframes slide-out-to-right {
    from {
      transform: translateX(0);
    }
    to {
      transform: translateX(100%);
    }
  }

  @keyframes fade-out {
    from {
      opacity: 1;
    }
    to {
      opacity: 0;
    }
  }
</style>
