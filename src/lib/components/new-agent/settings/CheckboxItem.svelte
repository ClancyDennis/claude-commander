<script lang="ts">
  import HelpTip from "../HelpTip.svelte";

  let {
    label,
    checked = $bindable(false),
    disabled = false,
    helpText,
    helpPlacement = "right",
  }: {
    label: string;
    checked?: boolean;
    disabled?: boolean;
    helpText?: string;
    helpPlacement?: "top" | "right" | "bottom" | "left";
  } = $props();
</script>

<label class="checkbox-item">
  <input type="checkbox" bind:checked disabled={disabled} />
  <span>
    {label}
    {#if helpText}
      <HelpTip text={helpText} placement={helpPlacement} />
    {/if}
  </span>
</label>

<style>
  .checkbox-item {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: 4px 0;
    cursor: pointer;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .checkbox-item:hover {
    color: var(--text-primary);
  }

  .checkbox-item input[type="checkbox"] {
    margin: 0;
    width: 16px;
    height: 16px;
    cursor: pointer;
    accent-color: var(--accent);
  }
</style>
