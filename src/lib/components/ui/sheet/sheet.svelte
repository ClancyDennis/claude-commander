<script lang="ts">
  import { Dialog as DialogPrimitive } from "bits-ui";

  let {
    open = $bindable(false),
    onOpenChange,
    children,
    ...restProps
  }: DialogPrimitive.RootProps = $props();
</script>

<DialogPrimitive.Root bind:open {onOpenChange} {...restProps}>
  {#if children}
    {@render children()}
  {/if}
</DialogPrimitive.Root>
