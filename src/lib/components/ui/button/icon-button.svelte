<script lang="ts" module>
  import { tv, type VariantProps } from "tailwind-variants";
  import type { Component } from "svelte";

  export const iconButtonVariants = tv({
    base: "inline-flex items-center justify-center gap-2 rounded-lg font-medium transition-all duration-200 cursor-pointer border-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent-hex)] disabled:pointer-events-none disabled:opacity-50",
    variants: {
      variant: {
        default:
          "bg-[var(--bg-elevated)] text-[var(--text-primary)] border border-[var(--border-hex)] hover:bg-[var(--bg-tertiary)] hover:border-[var(--border-light)]",
        ghost:
          "bg-transparent text-[var(--text-secondary)] hover:bg-[var(--bg-tertiary)] hover:text-[var(--text-primary)]",
        danger:
          "bg-[var(--error)] text-white hover:bg-[#c94545] shadow-[0_2px_8px_var(--error-glow)]",
        active:
          "bg-gradient-to-br from-[var(--accent-hex)] to-[#e85a45] text-white shadow-[0_2px_8px_var(--accent-glow)]",
      },
      size: {
        sm: "h-8 px-2 text-xs [&_svg]:size-4",
        md: "h-9 px-3 text-sm [&_svg]:size-[18px]",
        lg: "h-10 px-4 text-base [&_svg]:size-5",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "md",
    },
  });

  export type IconButtonVariant = VariantProps<typeof iconButtonVariants>["variant"];
  export type IconButtonSize = VariantProps<typeof iconButtonVariants>["size"];

  export type IconButtonProps = {
    icon: Component;
    label?: string;
    variant?: IconButtonVariant;
    size?: IconButtonSize;
    class?: string;
    title?: string;
  };
</script>

<script lang="ts">
  import { cn } from "$lib/utils";
  import type { HTMLButtonAttributes } from "svelte/elements";

  let {
    icon,
    label,
    variant = "default",
    size = "md",
    class: className,
    title,
    ...restProps
  }: IconButtonProps & HTMLButtonAttributes = $props();
</script>

<button
  class={cn(iconButtonVariants({ variant, size }), className)}
  title={title ?? label}
  {...restProps}
>
  {#if icon}
    {@const Icon = icon}
    <Icon />
  {/if}
  {#if label}
    <span class="label">{label}</span>
  {/if}
</button>

<style>
  .label {
    display: none;
  }

  @media (min-width: 640px) {
    .label {
      display: inline;
    }
  }
</style>
