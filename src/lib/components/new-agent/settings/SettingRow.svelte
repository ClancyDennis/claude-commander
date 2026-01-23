<script lang="ts">
  import type { Snippet } from "svelte";
  import HelpTip from "../HelpTip.svelte";

  let {
    label,
    helpText,
    helpPlacement = "right",
    children,
  }: {
    label: string;
    helpText?: string;
    helpPlacement?: "top" | "right" | "bottom" | "left";
    children?: Snippet;
  } = $props();
</script>

<label class="setting-row">
  <span>
    {label}
    {#if helpText}
      <HelpTip text={helpText} placement={helpPlacement} />
    {/if}
  </span>
  {@render children?.()}
</label>

<style>
  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-md);
    font-size: 13px;
    color: var(--text-secondary);
    padding: 4px 0;
  }
</style>
