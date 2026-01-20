<script lang="ts">
  import { formatTimeAbsolute } from '$lib/utils/formatting';
  import MarkdownRenderer from './MarkdownRenderer.svelte';

  let { data, index: i }: { data: { prompt: string; timestamp: number }; index: number } = $props();
</script>

<div class="prompt-item">
  <div class="prompt-header">
    <span class="prompt-number">Prompt #{i + 1}</span>
    <span class="prompt-timestamp">{formatTimeAbsolute(data.timestamp)}</span>
  </div>
  <div class="prompt-text">
    <MarkdownRenderer content={data.prompt} />
  </div>
</div>

<style>
  .prompt-item {
    background-color: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
    margin-bottom: var(--space-md);
  }

  .prompt-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-sm) var(--space-md);
    background-color: var(--bg-elevated);
    border-bottom: 1px solid var(--border);
  }

  .prompt-number {
    font-size: 12px;
    font-weight: 600;
    color: var(--accent);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .prompt-timestamp {
    font-size: 11px;
    color: var(--text-muted);
  }

  .prompt-text {
    padding: var(--space-md);
    color: var(--text-primary);
    font-size: 14px;
    line-height: 1.6;
  }
</style>
