<script lang="ts">
  import type { AgentOutputRecord } from "$lib/types";
  import { HistoricalOutputView } from '../history';

  interface Props {
    loading: boolean;
    error: string | null;
    outputs: AgentOutputRecord[];
    scrollTop: number;
  }

  let { loading, error, outputs, scrollTop }: Props = $props();
</script>

<div class="outputs-content">
  {#if loading}
    <div class="loading">
      <div class="spinner"></div>
      <p>Loading outputs...</p>
    </div>
  {:else if error}
    <div class="error-message">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <line x1="12" y1="8" x2="12" y2="12"/>
        <line x1="12" y1="16" x2="12.01" y2="16"/>
      </svg>
      <p>{error}</p>
    </div>
  {:else}
    <HistoricalOutputView {outputs} {scrollTop} />
  {/if}
</div>

<style>
  .outputs-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  /* Loading and error states */
  .loading,
  .error-message {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-xl);
    gap: var(--space-md);
    color: var(--text-muted);
    flex: 1;
    text-align: center;
  }

  .error-message svg {
    width: 48px;
    height: 48px;
    opacity: 0.5;
  }

  .spinner {
    width: 36px;
    height: 36px;
    border: 3px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
