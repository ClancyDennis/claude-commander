import type { ToolEvent, ToolCallStatistics } from "../types";

/**
 * Calculate statistics from tool events.
 * Only counts PostToolUse events to avoid double counting.
 */
export function calculateToolStats(tools: ToolEvent[]): ToolCallStatistics {
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
  const postTools = tools.filter((t) => t.hookEventName === "PostToolUse");

  postTools.forEach((tool) => {
    stats.totalCalls++;

    if (tool.status === "success") stats.successfulCalls++;
    else if (tool.status === "failed") stats.failedCalls++;
    else if (tool.status === "pending") stats.pendingCalls++;

    if (tool.executionTimeMs !== undefined) {
      totalTime += tool.executionTimeMs;
      countWithTime++;
    }

    stats.callsByTool[tool.toolName] =
      (stats.callsByTool[tool.toolName] || 0) + 1;
  });

  if (countWithTime > 0) {
    stats.averageExecutionTimeMs = totalTime / countWithTime;
  }

  return stats;
}

/**
 * Get unique tool names from tool events, sorted alphabetically.
 */
export function getUniqueToolNames(tools: ToolEvent[]): string[] {
  return [...new Set(tools.map((t) => t.toolName))].sort();
}

/**
 * Format tool input for display.
 * Returns the most relevant field or a JSON representation.
 */
export function formatToolInput(input: Record<string, unknown>): string {
  if (input.command) return String(input.command);
  if (input.file_path) return String(input.file_path);
  if (input.pattern) return String(input.pattern);
  return JSON.stringify(input, null, 2);
}

/**
 * Filter tool events based on status, tool name, and search query.
 */
export function filterToolEvents(
  tools: ToolEvent[],
  filterType: "all" | "success" | "failed" | "pending",
  selectedTool: string | "all",
  searchQuery: string
): ToolEvent[] {
  // Check if we need to filter at all
  if (filterType === "all" && selectedTool === "all" && !searchQuery.trim()) {
    return [...tools].reverse();
  }

  const filtered = tools.filter((t) => {
    // Filter by status
    if (filterType !== "all" && t.status !== filterType) return false;

    // Filter by tool name
    if (selectedTool !== "all" && t.toolName !== selectedTool) return false;

    // Filter by search query
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      if (
        !t.toolName.toLowerCase().includes(query) &&
        !formatToolInput(t.toolInput).toLowerCase().includes(query)
      ) {
        return false;
      }
    }

    return true;
  });

  // Return reversed copy (newest first)
  return [...filtered].reverse();
}

export type FilterType = "all" | "success" | "failed" | "pending";
