<script lang="ts">
  import type { AgentOutput } from "$lib/types";
  import MarkdownRenderer from "../MarkdownRenderer.svelte";

  let { data: output, index: i }: { data: AgentOutput; index: number } = $props();

  function formatTimestamp(timestamp: Date): string {
    if (!timestamp) return "";
    return new Date(timestamp).toLocaleTimeString();
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

  // Should this output type be displayed?
  function shouldDisplay(type: string): boolean {
    // Skip system and stream_event as they're internal messages
    return !['system', 'stream_event'].includes(type);
  }

  // Get display label for output type
  function getTypeLabel(type: string): string {
    switch (type) {
      case 'text': return 'Text';
      case 'tool_use': return 'Tool';
      case 'tool_result': return 'Result';
      case 'error': return 'Error';
      case 'result': return 'Completed';
      default: return type;
    }
  }
</script>

{#if shouldDisplay(output.type)}
  <div class="output-item {output.type} animate-slide-up" data-index={i}>
    <div class="output-header">
      <span class="output-type">{getTypeLabel(output.type)}</span>
      <span class="timestamp">
        {formatTimestamp(output.timestamp)}
      </span>
    </div>

    {#if output.type === 'result'}
      {@const resultText = extractResultText(output.content)}
      {#if resultText}
        <MarkdownRenderer content={resultText} />
      {:else if isJsonContent(output.content)}
        <pre class="json-content"><code>{output.content}</code></pre>
      {:else}
        <MarkdownRenderer content={output.content} />
      {/if}
    {:else if isJsonContent(output.content) && output.type !== 'text' && output.type !== 'tool_use'}
      <pre class="json-content"><code>{output.content}</code></pre>
    {:else}
      <MarkdownRenderer content={output.content} />
    {/if}
  </div>
{/if}

<style>
  .output-item {
    margin-bottom: var(--space-md);
    padding: var(--space-md);
    border-radius: 12px;
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
  }

  .output-item.error {
    background-color: var(--error-glow);
    border-color: var(--error);
  }

  .output-item.tool_use {
    background-color: rgba(245, 158, 11, 0.1);
    border-color: var(--warning);
  }

  .output-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-sm);
  }

  .output-type {
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--accent);
    background-color: rgba(124, 58, 237, 0.15);
    padding: 4px 10px;
    border-radius: 6px;
  }

  .output-item.error .output-type {
    color: var(--error);
    background-color: var(--error-glow);
  }

  .output-item.tool_use .output-type {
    color: var(--warning);
    background-color: rgba(245, 158, 11, 0.15);
  }

  .timestamp {
    font-size: 12px;
    color: var(--text-muted);
  }

  .json-content {
    background-color: #1e1e24;
    padding: 12px;
    border-radius: 8px;
    overflow-x: auto;
    font-family: 'SF Mono', 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    font-size: 12px;
    margin: 0;
    border: 1px solid var(--border);
    color: var(--text-secondary);
    white-space: pre-wrap;
    word-break: break-word;
  }

  .json-content code {
    background: none;
    padding: 0;
    font-size: inherit;
    color: inherit;
  }
</style>
