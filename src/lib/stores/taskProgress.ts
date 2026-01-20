import { derived } from 'svelte/store';
import { selectedAgentTools } from './agents';
import type { ToolEvent } from '../types';

export interface TaskStep {
  content: string;
  status: 'pending' | 'in_progress' | 'completed';
  activeForm?: string;
}

/**
 * Derives task progress from TodoWrite tool calls.
 *
 * Claude Code uses TodoWrite to track task progress with structured todos.
 * Each todo has: content, status, and activeForm fields.
 */
export const taskSteps = derived(selectedAgentTools, ($tools: ToolEvent[]) => {
  // Find all TodoWrite tool calls, sort by timestamp descending
  const todoEvents = $tools
    .filter(e => e.toolName === 'TodoWrite' && e.hookEventName === 'PreToolUse')
    .sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime());

  if (todoEvents.length === 0) return [];

  // Get the most recent TodoWrite call
  const latestTodo = todoEvents[0];
  const todos = latestTodo.toolInput?.todos as TaskStep[] | undefined;

  if (!todos || !Array.isArray(todos)) return [];

  // Validate and normalize the todos
  return todos.map(todo => ({
    content: String(todo.content || ''),
    status: validateStatus(todo.status),
    activeForm: todo.activeForm ? String(todo.activeForm) : undefined
  }));
});

function validateStatus(status: unknown): 'pending' | 'in_progress' | 'completed' {
  if (status === 'completed' || status === 'in_progress' || status === 'pending') {
    return status;
  }
  return 'pending';
}

/**
 * Derived store for summary statistics
 */
export const taskSummary = derived(taskSteps, ($steps) => {
  const total = $steps.length;
  const completed = $steps.filter(s => s.status === 'completed').length;
  const inProgress = $steps.filter(s => s.status === 'in_progress').length;
  const pending = $steps.filter(s => s.status === 'pending').length;

  const progress = total > 0 ? Math.round((completed / total) * 100) : 0;

  return {
    total,
    completed,
    inProgress,
    pending,
    progress
  };
});

/**
 * Get the current active task (in_progress status)
 */
export const currentTask = derived(taskSteps, ($steps) => {
  return $steps.find(s => s.status === 'in_progress') || null;
});
