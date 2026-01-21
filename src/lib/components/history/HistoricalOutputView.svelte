<script lang="ts">
  import { formatTimeLocale } from '$lib/utils/formatting';
  import type { AgentOutputRecord } from '$lib/types';
  import MarkdownRenderer from '../MarkdownRenderer.svelte';

  // Props
  let {
    outputs = [],
    scrollTop = 0,
  }: {
    outputs: AgentOutputRecord[];
    scrollTop?: number;
  } = $props();

  // Filter state
  let filterType = $state<'all' | 'text' | 'tool_use' | 'tool_result' | 'error' | 'result'>('all');

  // Virtualization settings
  const VISIBLE_ITEMS = 15;
  const ITEM_HEIGHT = 120; // Approximate height per output item

  // Filtered outputs
  let filteredOutputs = $derived(
    filterType === 'all'
      ? outputs.filter(o => o.output_type !== 'system') // Skip system messages
      : outputs.filter(o => o.output_type === filterType)
  );

  // Calculate visible window for virtualization
  function getVisibleItems(items: AgentOutputRecord[]): { visible: AgentOutputRecord[]; startIndex: number; totalHeight: number } {
    if (items.length <= VISIBLE_ITEMS) {
      return { visible: items, startIndex: 0, totalHeight: items.length * ITEM_HEIGHT };
    }

    const startIndex = Math.max(0, Math.floor(scrollTop / ITEM_HEIGHT) - 3);
    const endIndex = Math.min(items.length, startIndex + VISIBLE_ITEMS + 6);

    return {
      visible: items.slice(startIndex, endIndex),
      startIndex,
      totalHeight: items.length * ITEM_HEIGHT
    };
  }

  let visibleItems = $derived(getVisibleItems(filteredOutputs));

  // Get display label for output type
  function getTypeLabel(type: string): string {
    switch (type) {
      case 'text': return 'Text';
      case 'tool_use': return 'Tool';
      case 'tool_result': return 'Result';
      case 'error': return 'Error';
      case 'result': return 'Completed';
      case 'system': return 'System';
      default: return type;
    }
  }

  // Get CSS class for output type
  function getTypeClass(type: string): string {
    switch (type) {
      case 'error': return 'output-error';
      case 'tool_use': return 'output-tool';
      case 'tool_result': return 'output-tool-result';
      case 'result': return 'output-completed';
      default: return '';
    }
  }

  // Check if content is JSON
  function isJsonContent(content: string): boolean {
    if (!content || typeof content !== 'string') return false;
    const trimmed = content.trim();
    if (!trimmed.startsWith('{') && !trimmed.startsWith('[')) return false;
    try {
      JSON.parse(content);
      return true;
    } catch {
      return false;
    }
  }

  // Extract meaningful text from result JSON
  function extractResultText(content: string): string | null {
    try {
      const json = JSON.parse(content);
      if (json.result && typeof json.result === 'string') {
        return json.result;
      }
    } catch { /* ignore parse errors */ }
    return null;
  }

  // Truncate long content for preview
  function truncateContent(content: string, maxLength: number = 500): string {
    if (content.length <= maxLength) return content;
    return content.substring(0, maxLength) + '...';
  }

  // Count outputs by type
  function countByType(type: string): number {
    if (type === 'all') return outputs.filter(o => o.output_type !== 'system').length;
    return outputs.filter(o => o.output_type === type).length;
  }
</script>

<div class="historical-output-view">
  <div class="filter-bar">
    <button
      class="filter-btn"
      class:active={filterType === 'all'}
      onclick={() => filterType = 'all'}
    >
      All ({countByType('all')})
    </button>
    <button
      class="filter-btn"
      class:active={filterType === 'text'}
      onclick={() => filterType = 'text'}
    >
      Text ({countByType('text')})
    </button>
    <button
      class="filter-btn"
      class:active={filterType === 'tool_use'}
      onclick={() => filterType = 'tool_use'}
    >
      Tools ({countByType('tool_use')})
    </button>
    <button
      class="filter-btn"
      class:active={filterType === 'tool_result'}
      onclick={() => filterType = 'tool_result'}
    >
      Results ({countByType('tool_result')})
    </button>
    <button
      class="filter-btn"
      class:active={filterType === 'error'}
      onclick={() => filterType = 'error'}
    >
      Errors ({countByType('error')})
    </button>
  </div>

  <div class="output-list" style="height: {visibleItems.totalHeight}px; position: relative;">
    {#if filteredOutputs.length === 0}
      <div class="empty-state">No outputs recorded</div>
    {:else}
      <div style="position: absolute; top: {visibleItems.startIndex * ITEM_HEIGHT}px; width: 100%;">
        {#each visibleItems.visible as output, i (visibleItems.startIndex + i)}
          <div class="output-item {getTypeClass(output.output_type)}">
            <div class="output-header">
              <span class="output-type-badge">{getTypeLabel(output.output_type)}</span>
              <span class="output-time">{formatTimeLocale(output.timestamp)}</span>
            </div>
            <div class="output-content">
              {#if output.output_type === 'result'}
                {@const resultText = extractResultText(output.content)}
                {#if resultText}
                  <MarkdownRenderer content={truncateContent(resultText)} />
                {:else if isJsonContent(output.content)}
                  <pre class="json-content"><code>{truncateContent(output.content)}</code></pre>
                {:else}
                  <MarkdownRenderer content={truncateContent(output.content)} />
                {/if}
              {:else if isJsonContent(output.content) && output.output_type !== 'text'}
                <details class="content-details">
                  <summary>View JSON content</summary>
                  <pre class="json-content"><code>{output.content}</code></pre>
                </details>
              {:else if output.content.length > 500}
                <details class="content-details">
                  <summary>{truncateContent(output.content, 200)}</summary>
                  <MarkdownRenderer content={output.content} />
                </details>
              {:else}
                <MarkdownRenderer content={output.content} />
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .historical-output-view {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .filter-bar {
    display: flex;
    gap: 8px;
    padding: 12px;
    background: var(--bg-tertiary, #252530);
    border-bottom: 1px solid var(--border, rgba(124, 58, 237, 0.2));
    flex-wrap: wrap;
  }

  .filter-btn {
    padding: 6px 12px;
    border: 1px solid var(--border, rgba(124, 58, 237, 0.2));
    border-radius: 6px;
    background: transparent;
    color: var(--text-secondary, #999);
    font-size: 12px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .filter-btn:hover {
    background: var(--bg-hover, #2a2a35);
    color: var(--text-primary, #e0e0e0);
  }

  .filter-btn.active {
    background: var(--accent, #7c3aed);
    border-color: var(--accent, #7c3aed);
    color: white;
  }

  .output-list {
    flex: 1;
    overflow-y: auto;
    padding: 12px;
  }

  .empty-state {
    text-align: center;
    padding: 48px 24px;
    color: var(--text-muted, #666);
    font-size: 14px;
  }

  .output-item {
    background: var(--bg-primary, #0f0f13);
    border: 1px solid var(--border, rgba(124, 58, 237, 0.15));
    border-radius: 10px;
    padding: 12px;
    margin-bottom: 10px;
  }

  .output-item.output-error {
    border-color: rgba(239, 68, 68, 0.4);
    background: rgba(239, 68, 68, 0.05);
  }

  .output-item.output-tool {
    border-color: rgba(245, 158, 11, 0.4);
    background: rgba(245, 158, 11, 0.05);
  }

  .output-item.output-tool-result {
    border-color: rgba(59, 130, 246, 0.4);
    background: rgba(59, 130, 246, 0.05);
  }

  .output-item.output-completed {
    border-color: rgba(34, 197, 94, 0.4);
    background: rgba(34, 197, 94, 0.05);
  }

  .output-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 10px;
  }

  .output-type-badge {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--accent, #7c3aed);
    background: rgba(124, 58, 237, 0.15);
    padding: 4px 10px;
    border-radius: 6px;
  }

  .output-item.output-error .output-type-badge {
    color: #ef4444;
    background: rgba(239, 68, 68, 0.15);
  }

  .output-item.output-tool .output-type-badge {
    color: #f59e0b;
    background: rgba(245, 158, 11, 0.15);
  }

  .output-item.output-tool-result .output-type-badge {
    color: #3b82f6;
    background: rgba(59, 130, 246, 0.15);
  }

  .output-item.output-completed .output-type-badge {
    color: #22c55e;
    background: rgba(34, 197, 94, 0.15);
  }

  .output-time {
    font-size: 11px;
    color: var(--text-muted, #666);
  }

  .output-content {
    font-size: 13px;
    color: var(--text-primary, #e0e0e0);
    line-height: 1.5;
    overflow: hidden;
  }

  .output-content :global(p) {
    margin: 0 0 8px 0;
  }

  .output-content :global(p:last-child) {
    margin-bottom: 0;
  }

  .json-content {
    background: var(--bg-tertiary, #252530);
    padding: 10px;
    border-radius: 6px;
    overflow-x: auto;
    font-family: 'SF Mono', 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    font-size: 11px;
    margin: 0;
    border: 1px solid var(--border, rgba(124, 58, 237, 0.1));
    color: var(--text-secondary, #999);
    white-space: pre-wrap;
    word-break: break-word;
    max-height: 300px;
  }

  .json-content code {
    background: none;
    padding: 0;
    font-size: inherit;
    color: inherit;
  }

  .content-details {
    margin: 0;
  }

  .content-details summary {
    cursor: pointer;
    color: var(--text-secondary, #999);
    font-size: 13px;
  }

  .content-details[open] summary {
    margin-bottom: 10px;
  }

  /* Scrollbar styles */
  .output-list::-webkit-scrollbar {
    width: 6px;
  }

  .output-list::-webkit-scrollbar-track {
    background: transparent;
  }

  .output-list::-webkit-scrollbar-thumb {
    background: var(--border, rgba(124, 58, 237, 0.3));
    border-radius: 3px;
  }

  .output-list::-webkit-scrollbar-thumb:hover {
    background: var(--accent, rgba(124, 58, 237, 0.5));
  }
</style>
