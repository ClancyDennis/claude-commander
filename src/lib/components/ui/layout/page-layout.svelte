<script lang="ts" module>
  import { tv, type VariantProps } from "tailwind-variants";

  export const pageLayoutVariants = tv({
    base: "flex flex-col h-full w-full overflow-hidden",
    variants: {
      variant: {
        default: "bg-[var(--bg-primary)]",
        centered: "bg-[var(--bg-primary)] items-center justify-center",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  });

  export type PageLayoutVariant = VariantProps<typeof pageLayoutVariants>["variant"];

  export type PageLayoutProps = {
    variant?: PageLayoutVariant;
    class?: string;
  };
</script>

<script lang="ts">
  import { cn } from "$lib/utils";
  import type { Snippet } from "svelte";

  let {
    class: className,
    variant = "default",
    children,
  }: PageLayoutProps & { children?: Snippet } = $props();
</script>

<main class={cn(pageLayoutVariants({ variant }), className)}>
  {#if children}
    {@render children()}
  {/if}
</main>
