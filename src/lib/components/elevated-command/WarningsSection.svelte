<script lang="ts">
  import type { CommandRiskLevel } from "$lib/types";

  interface Props {
    warnings?: string[];
    riskLevel: CommandRiskLevel;
  }

  let { warnings = [], riskLevel }: Props = $props();
</script>

<!-- Warnings list -->
{#if warnings?.length}
  <div class="warnings-section">
    <h4>Warnings</h4>
    <ul class="warnings-list">
      {#each warnings as warning}
        <li class="warning-item">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 9v4m0 4h.01M12 2L2 22h20L12 2z"/>
          </svg>
          <span>{warning}</span>
        </li>
      {/each}
    </ul>
  </div>
{/if}

<!-- High risk extra warning -->
{#if riskLevel === "high"}
  <div class="danger-warning">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M12 9v4m0 4h.01M12 2L2 22h20L12 2z"/>
    </svg>
    <div class="danger-text">
      <strong>DANGER:</strong> This command has been flagged as potentially destructive.
      Carefully review before approving.
    </div>
  </div>
{/if}

<style>
  .warnings-section h4 {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-muted);
    margin: 0 0 var(--space-sm) 0;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .warnings-section {
    background-color: var(--warning-bg, rgba(234, 179, 8, 0.1));
    border: 1px solid var(--warning);
    border-radius: 8px;
    padding: var(--space-md);
  }

  .warnings-list {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .warning-item {
    display: flex;
    align-items: flex-start;
    gap: var(--space-sm);
    color: var(--warning);
    font-size: 14px;
    margin-bottom: var(--space-xs);
  }

  .warning-item:last-child {
    margin-bottom: 0;
  }

  .warning-item svg {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
    margin-top: 2px;
  }

  .danger-warning {
    display: flex;
    align-items: flex-start;
    gap: var(--space-md);
    background-color: var(--error-bg, rgba(239, 68, 68, 0.1));
    border: 2px solid var(--error);
    border-radius: 8px;
    padding: var(--space-md);
  }

  .danger-warning svg {
    width: 24px;
    height: 24px;
    color: var(--error);
    flex-shrink: 0;
  }

  .danger-text {
    color: var(--error);
    font-size: 14px;
    line-height: 1.5;
  }
</style>
