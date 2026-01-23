<script lang="ts">
  interface Props {
    scriptHash?: string;
    parentCmd?: string;
    approveScope: boolean;
    onApproveScopeChange: (value: boolean) => void;
  }

  let { scriptHash, parentCmd, approveScope, onApproveScopeChange }: Props = $props();

  function handleChange(event: Event) {
    const target = event.target as HTMLInputElement;
    onApproveScopeChange(target.checked);
  }
</script>

{#if scriptHash}
  <div class="scope-option">
    <label class="checkbox-label">
      <input type="checkbox" checked={approveScope} onchange={handleChange} />
      <span>Approve all commands from this script</span>
    </label>
    {#if parentCmd}
      <span class="parent-cmd">Script: {parentCmd}</span>
    {/if}
  </div>
{/if}

<style>
  .scope-option {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
    padding: var(--space-md);
    background-color: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 8px;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    cursor: pointer;
    font-size: 14px;
    color: var(--text-primary);
  }

  .checkbox-label input {
    width: 16px;
    height: 16px;
    cursor: pointer;
  }

  .parent-cmd {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-muted);
    margin-left: 24px;
  }
</style>
