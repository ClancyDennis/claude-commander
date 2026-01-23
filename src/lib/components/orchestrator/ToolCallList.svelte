<script lang="ts">
  import { formatTimeLocale } from '$lib/utils/formatting';
  import type { OrchestratorToolCall } from '$lib/types';

  // Props
  let {
    toolCalls = [],
    scrollTop = 0,
  }: {
    toolCalls: OrchestratorToolCall[];
    scrollTop?: number;
  } = $props();

  // Virtualization settings
  const VISIBLE_ITEMS = 20;
  const ITEM_HEIGHT = 80;

  // Calculate visible window for virtualization
  function getVisibleItems(items: OrchestratorToolCall[]): { visible: OrchestratorToolCall[]; startIndex: number; totalHeight: number } {
    if (items.length <= VISIBLE_ITEMS) {
      return { visible: items, startIndex: 0, totalHeight: items.length * ITEM_HEIGHT };
    }

    const startIndex = Math.max(0, Math.floor(scrollTop / ITEM_HEIGHT) - 5);
    const endIndex = Math.min(items.length, startIndex + VISIBLE_ITEMS + 10);

    return {
      visible: items.slice(startIndex, endIndex),
      startIndex,
      totalHeight: items.length * ITEM_HEIGHT
    };
  }

  let visibleItems = $derived(getVisibleItems(toolCalls));

  // Tool icon mapping
  function getToolIcon(toolName: string): string {
    const icons: Record<string, string> = {
      read_instruction_file: "📖",
      create_skill: "🎯",
      create_subagent: "🤖",
      generate_claudemd: "📝",
      start_planning: "📋",
      approve_plan: "✅",
      start_execution: "🔨",
      start_verification: "🔍",
      complete: "🎉",
      iterate: "🔄",
      replan: "📋",
      give_up: "❌",
    };
    return icons[toolName] || "⚙️";
  }
</script>

<div class="tool-list" style="height: {visibleItems.totalHeight}px; position: relative;">
  {#if toolCalls.length === 0}
    <div class="empty-state">No tool calls yet</div>
  {:else}
    <div style="position: absolute; top: {visibleItems.startIndex * ITEM_HEIGHT}px; width: 100%;">
      {#each visibleItems.visible as call, i (visibleItems.startIndex + i)}
        <div class="tool-item" class:error={call.is_error}>
          <div class="tool-header">
            <span class="tool-icon">{getToolIcon(call.tool_name)}</span>
            <span class="tool-name">{call.tool_name}</span>
            <span class="tool-time">{formatTimeLocale(call.timestamp)}</span>
          </div>
          {#if call.summary}
            <div class="tool-summary">{call.summary}</div>
          {/if}
          {#if call.tool_input && Object.keys(call.tool_input).length > 0}
            <details class="tool-details">
              <summary>Input</summary>
              <pre>{JSON.stringify(call.tool_input, null, 2)}</pre>
            </details>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .tool-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .empty-state {
    text-align: center;
    padding: 24px;
    color: var(--text-muted, #666);
    font-size: 13px;
  }

  .tool-item {
    background: var(--bg-primary, #0f0f13);
    border: 1px solid var(--border, rgba(240, 112, 90, 0.15));
    border-radius: 8px;
    padding: 10px 12px;
  }

  .tool-item.error {
    border-color: rgba(239, 68, 68, 0.4);
  }

  .tool-header {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .tool-icon {
    font-size: 14px;
  }

  .tool-name {
    font-weight: 600;
    font-size: 13px;
    color: var(--text-primary, #e0e0e0);
    flex: 1;
  }

  .tool-time {
    font-size: 11px;
    color: var(--text-muted, #666);
  }

  .tool-summary {
    margin-top: 6px;
    font-size: 12px;
    color: var(--text-secondary, #999);
    line-height: 1.4;
  }

  .tool-details {
    margin-top: 8px;
  }

  .tool-details summary {
    font-size: 11px;
    color: var(--accent, #f0705a);
    cursor: pointer;
  }

  .tool-details pre {
    margin-top: 6px;
    padding: 8px;
    background: var(--bg-tertiary, #252530);
    border-radius: 6px;
    font-size: 11px;
    overflow-x: auto;
    color: var(--text-secondary, #999);
  }
</style>
