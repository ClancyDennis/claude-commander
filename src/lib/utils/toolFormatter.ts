/**
 * Tool Content Formatter
 *
 * Formats raw tool_use and tool_result JSON from stored conversations
 * into human-readable display strings.
 */

// ============================================================================
// Types
// ============================================================================

export type FormattedContent = {
  type: 'text' | 'tool_use' | 'tool_result';
  formatted: string;
  toolName?: string;
  isError?: boolean;
};

interface ToolUseBlock {
  type: 'tool_use';
  id?: string;
  name: string;
  input: Record<string, unknown>;
}

interface ToolResultBlock {
  type: 'tool_result';
  tool_use_id?: string;
  content: string | unknown;
  is_error?: boolean;
}

type ContentBlock = ToolUseBlock | ToolResultBlock | { type: string };

// ============================================================================
// Main Formatter
// ============================================================================

/**
 * Parse and format message content for display.
 * Handles tool_use JSON, tool_result JSON, and regular text.
 */
export function formatMessageContent(content: string): FormattedContent[] {
  const trimmed = content.trim();

  // Check if this looks like JSON tool content
  if ((trimmed.startsWith('[{') || trimmed.startsWith('{')) &&
      (trimmed.includes('"type":"tool_use"') || trimmed.includes('"type":"tool_result"'))) {
    try {
      const parsed = JSON.parse(trimmed);
      const blocks: ContentBlock[] = Array.isArray(parsed) ? parsed : [parsed];
      return blocks.map(formatBlock).filter((b): b is FormattedContent => b !== null);
    } catch {
      // Not valid JSON, treat as text
    }
  }

  // Regular text content
  return [{ type: 'text', formatted: content }];
}

/**
 * Format a single content block
 */
function formatBlock(block: ContentBlock): FormattedContent | null {
  if (block.type === 'tool_use') {
    return formatToolUse(block as ToolUseBlock);
  }
  if (block.type === 'tool_result') {
    return formatToolResult(block as ToolResultBlock);
  }
  return null;
}

// ============================================================================
// Tool Use Formatting
// ============================================================================

/**
 * Format a tool_use block into readable text
 */
function formatToolUse(block: ToolUseBlock): FormattedContent {
  const toolName = block.name;
  const summary = summarizeToolInput(toolName, block.input);

  return {
    type: 'tool_use',
    toolName,
    formatted: summary,
  };
}

/**
 * Create a human-readable summary of tool input
 */
function summarizeToolInput(toolName: string, input: Record<string, unknown>): string {
  // Handle specific tools with custom formatting
  switch (toolName) {
    case 'CreateWorkerAgent':
      return truncateText(String(input.initial_prompt || ''), 150);

    case 'SendPromptToAgent':
      return `To: ${input.agent_id || 'agent'}\n${truncateText(String(input.prompt || ''), 120)}`;

    case 'GetAgentOutput':
    case 'GetAllAgents':
    case 'GetAgentStatus':
      return input.agent_id ? `Agent: ${input.agent_id}` : '';

    case 'StopAgent':
      return `Stopping: ${input.agent_id || 'agent'}`;

    case 'ExecuteShellCommand':
      return truncateText(String(input.command || ''), 100);

    case 'ReadFile':
    case 'WriteFile':
      return truncateText(String(input.path || input.file_path || ''), 80);

    case 'SearchFiles':
      return `Pattern: ${input.pattern || input.query || ''}`;

    case 'AskUser':
      return truncateText(String(input.question || ''), 120);

    case 'Sleep': {
      // Backend uses duration_minutes
      const mins = Number(input.duration_minutes || 0);
      const reason = input.reason ? ` - ${input.reason}` : '';
      if (mins >= 1) {
        return `${mins} minute${mins !== 1 ? 's' : ''}${reason}`;
      } else {
        return `${Math.round(mins * 60)} seconds${reason}`;
      }
    }

    default:
      // Generic: show first meaningful field
      return summarizeGenericInput(input);
  }
}

/**
 * Create a generic summary from tool input
 */
function summarizeGenericInput(input: Record<string, unknown>): string {
  const priorityFields = ['prompt', 'message', 'query', 'command', 'path', 'name', 'id'];

  for (const field of priorityFields) {
    if (input[field]) {
      return truncateText(String(input[field]), 120);
    }
  }

  // Fallback: stringify first few fields
  const entries = Object.entries(input).slice(0, 2);
  if (entries.length === 0) return '';

  return entries
    .map(([k, v]) => `${k}: ${truncateText(String(v), 50)}`)
    .join(', ');
}

// ============================================================================
// Tool Result Formatting
// ============================================================================

/**
 * Format a tool_result block into readable text
 */
function formatToolResult(block: ToolResultBlock): FormattedContent {
  const isError = block.is_error === true;
  const content = typeof block.content === 'string'
    ? block.content
    : JSON.stringify(block.content, null, 2);

  return {
    type: 'tool_result',
    isError,
    formatted: truncateText(content, 300),
  };
}

// ============================================================================
// Utilities
// ============================================================================

/**
 * Truncate text to a maximum length with ellipsis
 */
function truncateText(text: string, maxLength: number): string {
  if (!text) return '';
  const cleaned = text.trim().replace(/\n+/g, ' ');
  if (cleaned.length <= maxLength) return cleaned;
  return cleaned.slice(0, maxLength - 3) + '...';
}

/**
 * Check if a message content contains tool JSON
 */
export function isToolContent(content: string): boolean {
  const trimmed = content.trim();
  return (trimmed.startsWith('[{') || trimmed.startsWith('{')) &&
         (trimmed.includes('"type":"tool_use"') || trimmed.includes('"type":"tool_result"'));
}
