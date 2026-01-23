<script lang="ts">
  import type { PromptData } from './dataLoader';
  import { VirtualScroll } from "svelte-virtual-scroll-list";
  import HistoricalPromptItem from '../HistoricalPromptItem.svelte';

  interface Props {
    loading: boolean;
    error: string | null;
    prompts: PromptData[];
  }

  let { loading, error, prompts }: Props = $props();
</script>

<div class="prompts-content">
  {#if loading}
    <div class="loading">
      <div class="spinner"></div>
      <p>Loading conversation history...</p>
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
  {:else if prompts.length === 0}
    <div class="empty-state">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
      </svg>
      <p>No conversation history available</p>
    </div>
  {:else}
    <div class="prompts-list-wrapper">
      <VirtualScroll
        data={prompts}
        key="timestamp"
        let:data
        let:index
      >
        <HistoricalPromptItem {data} {index} />
      </VirtualScroll>
    </div>
  {/if}
</div>

<style>
  .prompts-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: var(--space-4);
  }

  .prompts-list-wrapper {
    flex: 1;
    overflow-y: hidden;
    height: 100%;
    min-height: 200px;
  }

  /* Loading, error, and empty states */
  .loading,
  .error-message,
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-8);
    gap: var(--space-4);
    color: var(--text-muted);
    flex: 1;
    text-align: center;
  }

  .loading p,
  .error-message p,
  .empty-state p {
    font-size: var(--text-sm);
    margin: 0;
  }

  .error-message svg,
  .empty-state svg {
    width: 40px;
    height: 40px;
    opacity: 0.5;
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 2px solid var(--border-hex);
    border-top-color: var(--accent-hex);
    border-radius: var(--radius-full);
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
