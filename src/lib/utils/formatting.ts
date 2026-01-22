/**
 * Unified formatting utilities for the Claude Commander frontend.
 *
 * This module consolidates all formatting functions that were previously
 * duplicated across multiple components.
 */

// ============================================================================
// Time Formatting
// ============================================================================

/**
 * Format a date/timestamp as relative time (e.g., "5m", "2h", "3d")
 * Used in: AgentList, AgentCard
 */
export function formatTimeRelative(date: Date | number): string {
  const now = new Date();
  const targetDate = typeof date === 'number' ? new Date(date) : date;
  const diff = now.getTime() - targetDate.getTime();
  const seconds = Math.floor(diff / 1000);
  const minutes = Math.floor(seconds / 60);

  if (seconds < 60) return seconds <= 0 ? 'now' : `${seconds}s`;
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h`;
  return `${Math.floor(hours / 24)}d`;
}

/**
 * Format a date/timestamp as relative time with "ago" suffix
 * Used in: AgentCard
 */
export function formatTimeAgo(date?: Date | number): string {
  if (!date) return 'Never';
  const now = new Date();
  const targetDate = typeof date === 'number' ? new Date(date) : date;
  const diff = now.getTime() - targetDate.getTime();
  const seconds = Math.floor(diff / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);

  if (seconds < 60) return `${seconds}s ago`;
  if (minutes < 60) return `${minutes}m ago`;
  if (hours < 24) return `${hours}h ago`;
  return targetDate.toLocaleDateString();
}

/**
 * Format a timestamp as absolute time (e.g., "Jan 5, 2:30 PM")
 * Used in: HistoricalRunView
 */
export function formatTimeAbsolute(timestamp: number): string {
  const date = new Date(timestamp);
  return date.toLocaleString('en-US', {
    month: 'short',
    day: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
    hour12: true
  });
}

/**
 * Format a timestamp as time of day only (e.g., "2:30 PM")
 * Used in: ChatMessage
 */
export function formatTimeOfDay(timestamp: number): string {
  const date = new Date(timestamp);
  return date.toLocaleTimeString('en-US', {
    hour: '2-digit',
    minute: '2-digit'
  });
}

/**
 * Format a timestamp as locale time string (e.g., "2:30:45 PM")
 * Used in: OrchestratorActivity, AgentStats
 */
export function formatTimeLocale(timestamp: number | string): string {
  try {
    const date = typeof timestamp === 'string' ? new Date(timestamp) : new Date(timestamp);
    return date.toLocaleTimeString();
  } catch {
    return 'N/A';
  }
}

/**
 * Format milliseconds as duration (e.g., "500ms", "2.5s")
 * Used in: ToolActivity
 */
export function formatTimeDuration(ms: number): string {
  if (ms < 1000) return `${ms.toFixed(0)}ms`;
  return `${(ms / 1000).toFixed(2)}s`;
}

// ============================================================================
// Date Formatting
// ============================================================================

/**
 * Format a timestamp as smart date (e.g., "Today 2:30 PM", "Yesterday", "Jan 5")
 * Used in: AgentList
 */
export function formatDate(timestamp: number): string {
  const date = new Date(timestamp);
  const now = new Date();
  const isToday = date.toDateString() === now.toDateString();

  if (isToday) {
    return date.toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit' });
  }

  const yesterday = new Date(now);
  yesterday.setDate(yesterday.getDate() - 1);
  if (date.toDateString() === yesterday.toDateString()) {
    return 'Yesterday';
  }

  return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
}

// ============================================================================
// Duration Formatting
// ============================================================================

/**
 * Format a duration as compact string (e.g., "5m", "2h 30m", "3d 12h")
 * Used in: AgentList, AgentStats
 */
export function formatDuration(startTime: number | string, endTime?: number): string {
  try {
    const start = typeof startTime === 'string' ? new Date(startTime).getTime() : startTime;
    const end = endTime || Date.now();
    const duration = end - start;
    const minutes = Math.floor(duration / 60000);

    if (minutes < 1) return '< 1m';
    if (minutes < 60) return `${minutes}m`;
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return `${hours}h ${minutes % 60}m`;
    const days = Math.floor(hours / 24);
    return `${days}d ${hours % 24}h`;
  } catch {
    return 'N/A';
  }
}

/**
 * Format a duration as verbose string (e.g., "5 minutes", "2 hours, 30 min")
 * Used in: HistoricalRunView
 */
export function formatDurationVerbose(startTime: number, endTime?: number): string {
  const end = endTime || Date.now();
  const duration = end - startTime;
  const minutes = Math.floor(duration / 60000);

  if (minutes < 1) return '< 1 minute';
  if (minutes < 60) return `${minutes} minute${minutes !== 1 ? 's' : ''}`;
  const hours = Math.floor(minutes / 60);
  const remainingMins = minutes % 60;
  if (hours < 24) {
    return remainingMins > 0
      ? `${hours} hour${hours !== 1 ? 's' : ''}, ${remainingMins} min`
      : `${hours} hour${hours !== 1 ? 's' : ''}`;
  }
  const days = Math.floor(hours / 24);
  const remainingHours = hours % 24;
  return remainingHours > 0
    ? `${days} day${days !== 1 ? 's' : ''}, ${remainingHours} hour${remainingHours !== 1 ? 's' : ''}`
    : `${days} day${days !== 1 ? 's' : ''}`;
}

// ============================================================================
// Size Formatting
// ============================================================================

/**
 * Format bytes as human-readable string (e.g., "1.5 MB")
 * Used in: AgentStats, HistoricalRunView
 */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
}

// ============================================================================
// Cost Formatting
// ============================================================================

/**
 * Format cost in USD (e.g., "$0.0012")
 * Used in: AgentStats, HistoricalRunView
 */
export function formatCost(cost?: number): string {
  if (cost === undefined || cost === null) return 'N/A';
  if (cost === 0) return '$0.00';
  return `$${cost.toFixed(4)}`;
}

/**
 * Format a number with locale separators
 * Used in: AgentStats
 */
export function formatNumber(num?: number): string {
  if (num === undefined || num === null) return 'N/A';
  return num.toLocaleString();
}

// ============================================================================
// Path Formatting
// ============================================================================

/**
 * Format a file path to show just the last segment
 * Used in: AgentList, AgentCard
 */
export function formatPath(path: string): string {
  const parts = path.split('/');
  return parts[parts.length - 1] || path;
}

/**
 * Truncate a path to a maximum length with ellipsis
 */
export function formatPathTruncated(path: string, maxLength: number = 50): string {
  if (path.length <= maxLength) return path;
  const start = path.slice(0, Math.floor(maxLength / 2) - 2);
  const end = path.slice(-(Math.floor(maxLength / 2) - 1));
  return `${start}...${end}`;
}
