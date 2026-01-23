/**
 * Unified output type utilities for the Claude Commander frontend.
 *
 * Consolidates output type classification that was duplicated across:
 * - HistoricalOutputView.svelte
 * - AgentOutputItem.svelte
 * - AutoPipelineView.svelte
 */

// Output type values
export type OutputType =
  | 'text'
  | 'tool_use'
  | 'tool_result'
  | 'error'
  | 'result'
  | 'system'
  | 'stream_event'
  | 'orchestrator_tool'
  | 'state_change'
  | 'decision';

// ============================================================================
// Output Type Labels
// ============================================================================

/**
 * Get human-readable label for output type
 * Used in: HistoricalOutputView, AgentOutputItem, AutoPipelineView
 */
export function getOutputTypeLabel(type: string): string {
  switch (type) {
    case 'text':
      return 'Text';
    case 'tool_use':
      return 'Tool';
    case 'tool_result':
      return 'Result';
    case 'error':
      return 'Error';
    case 'result':
      return 'Completed';
    case 'system':
      return 'System';
    case 'orchestrator_tool':
      return 'Orchestrator';
    case 'state_change':
      return 'State';
    case 'decision':
      return 'Decision';
    default:
      return type;
  }
}

// ============================================================================
// Output Type CSS Classes
// ============================================================================

/**
 * Get CSS class for output type styling
 * Used in: HistoricalOutputView
 */
export function getOutputTypeClass(type: string): string {
  switch (type) {
    case 'error':
      return 'output-error';
    case 'tool_use':
      return 'output-tool';
    case 'tool_result':
      return 'output-tool-result';
    case 'result':
      return 'output-completed';
    case 'orchestrator_tool':
      return 'output-orchestrator';
    case 'state_change':
      return 'output-state-change';
    case 'decision':
      return 'output-decision';
    default:
      return '';
  }
}

// ============================================================================
// Output Type Filtering
// ============================================================================

/**
 * Check if output type should be displayed in UI
 * Used in: AgentOutputItem
 */
export function shouldDisplayOutputType(type: string): boolean {
  // Skip system and stream_event as they're internal messages
  return !['system', 'stream_event'].includes(type);
}

/**
 * Filter types that are typically hidden from user view
 */
export const HIDDEN_OUTPUT_TYPES = ['system', 'stream_event'] as const;

// ============================================================================
// Content Helpers
// ============================================================================

/**
 * Check if content string is valid JSON
 * Used in: HistoricalOutputView, AgentOutputItem
 */
export function isJsonContent(content: string): boolean {
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

/**
 * Extract meaningful text from result JSON
 * Used in: HistoricalOutputView, AgentOutputItem
 */
export function extractResultText(content: string): string | null {
  try {
    const json = JSON.parse(content);
    if (json.result && typeof json.result === 'string') {
      return json.result;
    }
  } catch {
    /* ignore parse errors */
  }
  return null;
}

/**
 * Truncate long content for preview display
 * Used in: HistoricalOutputView
 */
export function truncateContent(content: string, maxLength: number = 500): string {
  if (content.length <= maxLength) return content;
  return content.substring(0, maxLength) + '...';
}
