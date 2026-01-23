<script lang="ts">
  import type { CommandRiskLevel } from "$lib/types";

  interface Props {
    riskLevel: CommandRiskLevel;
    isProcessing: boolean;
    error: string | null;
    onApprove: () => void;
    onDeny: () => void;
  }

  let { riskLevel, isProcessing, error, onApprove, onDeny }: Props = $props();
</script>

<!-- Error display -->
{#if error}
  <div class="error-message">
    {error}
  </div>
{/if}

<div class="footer">
  <button
    class="btn btn-deny"
    onclick={onDeny}
    disabled={isProcessing}
  >
    {isProcessing ? "Processing..." : "Deny"}
  </button>
  <button
    class="btn btn-approve"
    class:btn-dangerous={riskLevel === "high"}
    onclick={onApprove}
    disabled={isProcessing}
  >
    {#if riskLevel === "high"}
      {isProcessing ? "Processing..." : "Approve Anyway"}
    {:else}
      {isProcessing ? "Processing..." : "Approve"}
    {/if}
  </button>
</div>

<style>
  .error-message {
    background-color: var(--error-bg, rgba(239, 68, 68, 0.1));
    border: 1px solid var(--error);
    border-radius: 8px;
    padding: var(--space-md);
    color: var(--error);
    font-size: 14px;
  }

  .footer {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-md);
    padding: var(--space-lg);
    border-top: 1px solid var(--border);
    background-color: var(--bg-primary);
  }

  .btn {
    padding: 10px 24px;
    font-size: 14px;
    font-weight: 600;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s ease;
    border: none;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-deny {
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
    border: 1px solid var(--border);
  }

  .btn-deny:hover:not(:disabled) {
    background-color: var(--bg-secondary);
    border-color: var(--text-muted);
  }

  .btn-approve {
    background-color: var(--success);
    color: white;
  }

  .btn-approve:hover:not(:disabled) {
    opacity: 0.9;
  }

  .btn-approve.btn-dangerous {
    background-color: var(--error);
  }
</style>
