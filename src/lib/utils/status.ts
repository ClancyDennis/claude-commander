/**
 * Unified status color and label utilities for the Claude Commander frontend.
 *
 * This module consolidates status-related functions that were previously
 * duplicated across AgentList, StatusBadge, and HistoricalRunView.
 */

import type { AgentStatus, RunStatus, AgentComplexity } from '$lib/types';

// Union type for all status values
type Status = AgentStatus | RunStatus | string;

// ============================================================================
// Complexity Colors (for model tier indicators)
// ============================================================================

/**
 * Get hex color for complexity level
 * Used in: AgentListItem
 */
export function getComplexityColor(complexity?: AgentComplexity | string): string {
  switch (complexity) {
    case 'simple':
      return '#10b981'; // emerald-500 (Haiku)
    case 'complex':
      return '#8b5cf6'; // violet-500 (Opus)
    case 'easy':
    default:
      return '#3b82f6'; // blue-500 (Sonnet)
  }
}

/**
 * Get background tint for complexity level (subtle indicator)
 * Used in: AgentListItem
 */
export function getComplexityBackground(complexity?: AgentComplexity | string): string {
  switch (complexity) {
    case 'simple':
      return 'rgba(16, 185, 129, 0.08)'; // emerald tint
    case 'complex':
      return 'rgba(139, 92, 246, 0.08)'; // violet tint
    case 'easy':
    default:
      return 'transparent'; // no tint for default
  }
}

/**
 * Get human-readable label for complexity level
 * Used in: AgentListItem tooltip
 */
export function getComplexityLabel(complexity?: AgentComplexity | string): string {
  switch (complexity) {
    case 'simple':
      return 'Simple (Haiku)';
    case 'complex':
      return 'Complex (Opus)';
    case 'easy':
      return 'Easy (Sonnet)';
    default:
      return 'Default';
  }
}

// ============================================================================
// Status Colors
// ============================================================================

/**
 * Get CSS variable color for agent status
 * Used in: StatusBadge
 */
export function getAgentStatusColor(status: AgentStatus): string {
  switch (status) {
    case 'running':
      return 'var(--success)';
    case 'waitingforinput':
      return 'var(--warning)';
    case 'idle':
      return 'var(--text-secondary)';
    case 'processing':
      return 'var(--accent)';
    case 'stopped':
      return 'var(--text-muted)';
    case 'error':
      return 'var(--error)';
    default:
      return 'var(--text-muted)';
  }
}

/**
 * Get CSS variable color for run status
 * Used in: AgentList
 */
export function getRunStatusColor(status: RunStatus): string {
  switch (status) {
    case 'running':
      return 'var(--success)';
    case 'completed':
      return 'var(--success)';
    case 'stopped':
      return 'var(--text-muted)';
    case 'crashed':
      return 'var(--text-secondary)';
    case 'waiting_input':
      return 'var(--warning)';
    default:
      return 'var(--text-muted)';
  }
}

/**
 * Unified status color getter that works with both AgentStatus and RunStatus
 * Uses CSS variables for consistency with the design system.
 * Used in: AgentList, StatusBadge
 */
export function getStatusColor(status: Status): string {
  switch (status) {
    // Common statuses
    case 'running':
      return 'var(--success)';
    case 'stopped':
      return 'var(--text-muted)';
    case 'error':
      return 'var(--error)';
    case 'crashed':
      return 'var(--text-secondary)';

    // AgentStatus specific
    case 'waitingforinput':
      return 'var(--warning)';
    case 'idle':
      return 'var(--text-secondary)';
    case 'processing':
      return 'var(--accent)';

    // RunStatus specific
    case 'completed':
      return 'var(--success)';
    case 'waiting_input':
      return 'var(--warning)';

    default:
      return 'var(--text-muted)';
  }
}

/**
 * Get hex color for status (useful for charts and places where CSS vars don't work)
 * Used in: HistoricalRunView
 */
export function getStatusColorHex(status: Status): string {
  switch (status) {
    case 'running':
    case 'completed':
      return '#10b981'; // green-500
    case 'stopped':
    case 'idle':
      return '#6b7280'; // gray-500
    case 'error':
      return '#ef4444'; // red-500
    case 'crashed':
      return '#9ca3af'; // gray-400 (slightly lighter than stopped)
    case 'waitingforinput':
    case 'waiting_input':
      return '#f59e0b'; // amber-500
    case 'processing':
      return '#f0705a'; // coral (accent)
    default:
      return '#6b7280'; // gray-500
  }
}

/**
 * Get color for pipeline status strings
 * Used in: AgentList
 */
export function getPipelineStatusColor(status: string): string {
  switch (status) {
    case 'Completed':
      return '#10b981'; // green
    case 'Running':
      return 'var(--accent)';
    case 'Failed':
      return 'var(--error)';
    default:
      return 'var(--text-muted)';
  }
}

// ============================================================================
// Status Labels
// ============================================================================

/**
 * Get human-readable label for agent status
 * Used in: StatusBadge
 */
export function getAgentStatusLabel(status: AgentStatus): string {
  switch (status) {
    case 'running':
      return 'Running';
    case 'waitingforinput':
      return 'Needs Input';
    case 'idle':
      return 'Idle';
    case 'processing':
      return 'Processing';
    case 'stopped':
      return 'Stopped';
    case 'error':
      return 'Error';
    default:
      return 'Unknown';
  }
}

/**
 * Get human-readable label for run status
 * Used in: AgentList
 */
export function getRunStatusLabel(status: RunStatus): string {
  switch (status) {
    case 'running':
      return 'Running';
    case 'completed':
      return 'Completed';
    case 'stopped':
      return 'Stopped';
    case 'crashed':
      return 'Ended';
    case 'waiting_input':
      return 'Waiting';
    default:
      return status;
  }
}

/**
 * Unified status label getter
 * Used in: AgentList, StatusBadge
 */
export function getStatusLabel(status: Status): string {
  switch (status) {
    // Common
    case 'running':
      return 'Running';
    case 'stopped':
      return 'Stopped';
    case 'error':
      return 'Error';

    // AgentStatus specific
    case 'waitingforinput':
      return 'Needs Input';
    case 'idle':
      return 'Idle';
    case 'processing':
      return 'Processing';

    // RunStatus specific
    case 'completed':
      return 'Completed';
    case 'crashed':
      return 'Ended';
    case 'waiting_input':
      return 'Waiting';

    default:
      return String(status);
  }
}
