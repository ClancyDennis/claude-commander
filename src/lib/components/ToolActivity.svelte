<script lang="ts">
  import { derived } from "svelte/store";
  import { selectedAgentTools } from "../stores/agents";
  import type { ToolEvent, ToolCallStatistics } from "../types";
  import { formatTimeDuration } from '$lib/utils/formatting';
  import { isResizing } from "$lib/stores/resize";

  let filterType = $state<"all" | "success" | "failed" | "pending">("all");
  let selectedTool = $state<string | "all">("all");
  let searchQuery = $state("");
  let eventsContainer: HTMLDivElement | null = $state(null);
  let scrollTop = $state(0);

  // Virtualization settings
  const VISIBLE_ITEMS = 20;
  const ITEM_HEIGHT = 100; // Approximate height of each tool event card

  // Calculate statistics from tool events
  const toolStats = $derived.by(() => {
    const tools = $selectedAgentTools;
    const stats: ToolCallStatistics = {
      totalCalls: 0,
      successfulCalls: 0,
      failedCalls: 0,
      pendingCalls: 0,
      averageExecutionTimeMs: 0,
      callsByTool: {},
    };

    let totalTime = 0;
    let countWithTime = 0;

    // Only count PostToolUse events for stats to avoid double counting
    const postTools = tools.filter(t => t.hookEventName === "PostToolUse");

    postTools.forEach(tool => {
      stats.totalCalls++;

      if (tool.status === "success") stats.successfulCalls++;
      else if (tool.status === "failed") stats.failedCalls++;
      else if (tool.status === "pending") stats.pendingCalls++;

      if (tool.executionTimeMs !== undefined) {
        totalTime += tool.executionTimeMs;
        countWithTime++;
      }

      stats.callsByTool[tool.toolName] = (stats.callsByTool[tool.toolName] || 0) + 1;
    });

    if (countWithTime > 0) {
      stats.averageExecutionTimeMs = totalTime / countWithTime;
    }

    return stats;
  });

  // Get unique tool names
  const uniqueTools = $derived([...new Set($selectedAgentTools.map(t => t.toolName))].sort());

  // Filter tool events
  const filteredTools = $derived.by(() => {
    let filtered = $selectedAgentTools;
    let needsFilter = false;

    // Check if we need to filter at all
    if (filterType !== "all" || selectedTool !== "all" || searchQuery.trim()) {
      needsFilter = true;
      filtered = $selectedAgentTools.filter(t => {
        // Filter by status
        if (filterType !== "all" && t.status !== filterType) return false;

        // Filter by tool name
        if (selectedTool !== "all" && t.toolName !== selectedTool) return false;

        // Filter by search query
        if (searchQuery.trim()) {
          const query = searchQuery.toLowerCase();
          if (!t.toolName.toLowerCase().includes(query) &&
              !formatToolInput(t.toolInput).toLowerCase().includes(query)) {
            return false;
          }
        }

        return true;
      });
    }

    // Only reverse if we filtered (to show newest first)
    return needsFilter ? filtered.reverse() : filtered.slice().reverse();
  });

  // Virtualization: calculate visible items based on scroll position
  const visibleTools = $derived.by(() => {
    const filtered = filteredTools;
    if (filtered.length <= VISIBLE_ITEMS) {
      return { visible: filtered, startIndex: 0, totalHeight: filtered.length * ITEM_HEIGHT };
    }

    const startIndex = Math.max(0, Math.floor(scrollTop / ITEM_HEIGHT) - 5);
    const endIndex = Math.min(filtered.length, startIndex + VISIBLE_ITEMS + 10);

    return {
      visible: filtered.slice(startIndex, endIndex),
      startIndex,
      totalHeight: filtered.length * ITEM_HEIGHT
    };
  });

  function handleScroll(e: Event) {
    const target = e.target as HTMLElement;
    scrollTop = target.scrollTop;
  }

  function formatToolInput(input: Record<string, unknown>): string {
    if (input.command) return String(input.command);
    if (input.file_path) return String(input.file_path);
    if (input.pattern) return String(input.pattern);
    return JSON.stringify(input, null, 2);
  }

  // Track previous tools length to only scroll on actual new events
  // Using a plain object to avoid reactive tracking issues in Svelte 5
  const scrollState = { prevLength: 0 };

  // Auto-scroll to latest tool events when new events arrive (not on every re-render)
  $effect(() => {
    const currentLength = $selectedAgentTools.length;
    const container = eventsContainer; // capture current value
    const totalHeight = visibleTools.totalHeight; // capture for virtualized scroll
    // Skip scroll operations during resize to prevent layout thrashing
    if ($isResizing) return;

    // Only scroll if we have new events
    if (container && currentLength > scrollState.prevLength && currentLength > 0 && filterType === "all" && selectedTool === "all" && !searchQuery.trim()) {
      scrollState.prevLength = currentLength;

      const { scrollTop: currentScrollTop, clientHeight } = container;
      const distanceToBottom = totalHeight - currentScrollTop - clientHeight;

      // Auto-scroll if we are within 500px of the bottom (or at the top)
      if (distanceToBottom < 500 || currentScrollTop === 0) {
        requestAnimationFrame(() => {
          if (container) {
            container.scrollTop = totalHeight;
          }
        });
      }
    } else if (currentLength < scrollState.prevLength) {
      // Reset if tools were cleared
      scrollState.prevLength = currentLength;
    }
  });
</script>

<aside class="tool-activity">
  <header>
    <div class="header-content">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/>
      </svg>
      <h3>Tool Activity</h3>
    </div>
    <span class="count">{toolStats.totalCalls}</span>
  </header>

  <!-- Statistics Summary -->
  {#if toolStats.totalCalls > 0}
    <div class="stats-summary">
      <div class="stat">
        <div class="stat-label">Success Rate</div>
        <div class="stat-value success">
          {((toolStats.successfulCalls / toolStats.totalCalls) * 100).toFixed(0)}%
        </div>
      </div>
      <div class="stat">
        <div class="stat-label">Avg Time</div>
        <div class="stat-value">{formatTimeDuration(toolStats.averageExecutionTimeMs)}</div>
      </div>
      <div class="stat">
        <div class="stat-label">Failed</div>
        <div class="stat-value {toolStats.failedCalls > 0 ? 'error' : ''}">{toolStats.failedCalls}</div>
      </div>
    </div>
  {/if}

  <!-- Filters -->
  <div class="filters">
    <div class="filter-row">
      <select bind:value={filterType} class="filter-select">
        <option value="all">All Status</option>
        <option value="success">Success</option>
        <option value="failed">Failed</option>
        <option value="pending">Pending</option>
      </select>

      <select bind:value={selectedTool} class="filter-select">
        <option value="all">All Tools</option>
        {#each uniqueTools as tool}
          <option value={tool}>{tool}</option>
        {/each}
      </select>
    </div>

    <input
      type="text"
      placeholder="Search tools..."
      bind:value={searchQuery}
      class="search-input"
    />
  </div>

  <div class="events" bind:this={eventsContainer} onscroll={handleScroll}>
    <div class="virtual-spacer" style="height: {visibleTools.totalHeight}px; position: relative;">
      <div style="position: absolute; top: {visibleTools.startIndex * ITEM_HEIGHT}px; width: 100%;">
        {#each visibleTools.visible as event, i (event.toolCallId + (visibleTools.startIndex + i))}
          <div class="event {event.hookEventName.toLowerCase()}">
        <div class="event-header">
          <div class="tool-info">
            <span class="tool-icon {event.toolName.toLowerCase()}">
              {#if event.toolName === "Bash"}
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="4,17 10,11 4,5"/>
                  <line x1="12" y1="19" x2="20" y2="19"/>
                </svg>
              {:else if event.toolName === "Read"}
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                  <polyline points="14,2 14,8 20,8"/>
                  <line x1="16" y1="13" x2="8" y2="13"/>
                  <line x1="16" y1="17" x2="8" y2="17"/>
                </svg>
              {:else if event.toolName === "Write" || event.toolName === "Edit"}
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
                  <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
                </svg>
              {:else if event.toolName === "Glob" || event.toolName === "Grep"}
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="11" cy="11" r="8"/>
                  <line x1="21" y1="21" x2="16.65" y2="16.65"/>
                </svg>
              {:else if event.toolName === "WebFetch"}
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="12" r="10"/>
                  <line x1="2" y1="12" x2="22" y2="12"/>
                  <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
                </svg>
              {:else}
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="12" r="3"/>
                  <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/>
                </svg>
              {/if}
            </span>
            <div class="tool-details">
              <span class="tool-name">{event.toolName}</span>
              {#if event.executionTimeMs !== undefined}
                <span class="execution-time">{formatTimeDuration(event.executionTimeMs)}</span>
              {/if}
            </div>
          </div>

          <!-- Status indicator -->
          {#if event.status}
            <div class="status-badge {event.status}">
              {#if event.status === "pending"}
                <div class="spinner"></div>
              {:else if event.status === "success"}
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="20 6 9 17 4 12"/>
                </svg>
              {:else if event.status === "failed"}
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <line x1="18" y1="6" x2="6" y2="18"/>
                  <line x1="6" y1="6" x2="18" y2="18"/>
                </svg>
              {/if}
            </div>
          {/if}
        </div>

        <div class="event-content">
          <code>{formatToolInput(event.toolInput)}</code>
        </div>

        {#if event.errorMessage}
          <div class="error-message">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"/>
              <line x1="12" y1="8" x2="12" y2="12"/>
              <line x1="12" y1="16" x2="12.01" y2="16"/>
            </svg>
            {event.errorMessage}
          </div>
        {/if}

        <span class="timestamp">
          {event.timestamp.toLocaleTimeString()}
        </span>
      </div>
        {/each}
      </div>
    </div>

    {#if filteredTools.length === 0 && $selectedAgentTools.length > 0}
      <div class="empty">
        <div class="empty-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <circle cx="11" cy="11" r="8"/>
            <line x1="21" y1="21" x2="16.65" y2="16.65"/>
          </svg>
        </div>
        <p class="empty-title">No matching tools</p>
        <p class="empty-hint">Try adjusting your filters</p>
      </div>
    {/if}

    {#if $selectedAgentTools.length === 0}
      <div class="empty">
        <div class="empty-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/>
          </svg>
        </div>
        <p class="empty-title">No tool activity</p>
        <p class="empty-hint">Tool usage will appear here</p>
      </div>
    {/if}
  </div>
</aside>

<style>
  .tool-activity {
    width: 340px;
    min-width: 340px;
    height: 100%;
    background-color: var(--bg-secondary);
    border-left: 1px solid var(--border);
    display: flex;
    flex-direction: column;
  }

  header {
    padding: var(--space-md) var(--space-lg);
    border-bottom: 1px solid var(--border);
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: linear-gradient(180deg, var(--bg-tertiary) 0%, var(--bg-secondary) 100%);
  }

  .header-content {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .header-content svg {
    width: 20px;
    height: 20px;
    color: var(--accent);
  }

  h3 {
    font-size: 16px;
    font-weight: 600;
  }

  .count {
    background-color: var(--bg-elevated);
    padding: 4px 12px;
    border-radius: 12px;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .stats-summary {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-sm);
    padding: var(--space-md);
    border-bottom: 1px solid var(--border);
    background-color: var(--bg-tertiary);
  }

  .stat {
    text-align: center;
  }

  .stat-label {
    font-size: 11px;
    color: var(--text-muted);
    text-transform: uppercase;
    font-weight: 600;
    letter-spacing: 0.5px;
    margin-bottom: 4px;
  }

  .stat-value {
    font-size: 18px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .stat-value.success {
    color: var(--success);
  }

  .stat-value.error {
    color: var(--error);
  }

  .filters {
    padding: var(--space-md);
    border-bottom: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .filter-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-sm);
  }

  .filter-select, .search-input {
    padding: 8px 12px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
    font-size: 13px;
    font-weight: 500;
  }

  .filter-select:focus, .search-input:focus {
    outline: none;
    border-color: var(--accent);
  }

  .search-input {
    width: 100%;
  }

  .events {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-sm);
  }

  .event {
    padding: var(--space-md);
    margin-bottom: var(--space-sm);
    background-color: var(--bg-tertiary);
    border-radius: 12px;
    border: 1px solid var(--border);
  }

  .event.pretooluse {
    border-left: 3px solid var(--warning);
  }

  .event.posttooluse {
    border-left: 3px solid var(--success);
  }

  .event-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-sm);
  }

  .tool-info {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .tool-details {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .tool-icon {
    width: 32px;
    height: 32px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--bg-elevated);
    flex-shrink: 0;
  }

  .tool-icon svg {
    width: 16px;
    height: 16px;
    color: var(--text-secondary);
  }

  .tool-icon.bash {
    background-color: rgba(34, 197, 94, 0.15);
  }
  .tool-icon.bash svg {
    color: var(--success);
  }

  .tool-icon.read {
    background-color: rgba(59, 130, 246, 0.15);
  }
  .tool-icon.read svg {
    color: #3b82f6;
  }

  .tool-icon.write, .tool-icon.edit {
    background-color: rgba(245, 158, 11, 0.15);
  }
  .tool-icon.write svg, .tool-icon.edit svg {
    color: var(--warning);
  }

  .tool-icon.glob, .tool-icon.grep {
    background-color: rgba(45, 177, 160, 0.15);
  }
  .tool-icon.glob svg, .tool-icon.grep svg {
    color: #2db1a0;
  }

  .tool-icon.webfetch {
    background-color: rgba(236, 72, 153, 0.15);
  }
  .tool-icon.webfetch svg {
    color: #ec4899;
  }

  .tool-name {
    font-weight: 600;
    font-size: 15px;
  }

  .execution-time {
    font-size: 11px;
    color: var(--text-muted);
    font-weight: 500;
  }

  .status-badge {
    width: 24px;
    height: 24px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .status-badge svg {
    width: 14px;
    height: 14px;
  }

  .status-badge.pending {
    background-color: rgba(156, 163, 175, 0.15);
  }

  .status-badge.success {
    background-color: var(--success-glow);
    color: var(--success);
  }

  .status-badge.failed {
    background-color: var(--error-glow);
    color: var(--error);
  }

  .spinner {
    width: 14px;
    height: 14px;
    border: 2px solid rgba(156, 163, 175, 0.3);
    border-top-color: var(--text-muted);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .event-content {
    margin-bottom: var(--space-sm);
    padding: var(--space-sm);
    background-color: var(--bg-primary);
    border-radius: 8px;
  }

  code {
    font-family: 'SF Mono', 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    font-size: 12px;
    color: var(--text-secondary);
    display: block;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .error-message {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-sm);
    background-color: var(--error-glow);
    border: 1px solid var(--error);
    border-radius: 8px;
    margin-bottom: var(--space-sm);
    font-size: 12px;
    color: var(--error);
  }

  .error-message svg {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
  }

  .timestamp {
    font-size: 12px;
    color: var(--text-muted);
  }

  .empty {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-xl);
    text-align: center;
  }

  .empty-icon {
    width: 64px;
    height: 64px;
    border-radius: 20px;
    background: linear-gradient(135deg, var(--bg-tertiary) 0%, var(--bg-elevated) 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: var(--space-md);
    border: 1px solid var(--border);
  }

  .empty-icon svg {
    width: 32px;
    height: 32px;
    color: var(--text-muted);
  }

  .empty-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: var(--space-xs);
  }

  .empty-hint {
    font-size: 13px;
    color: var(--text-muted);
  }
</style>
