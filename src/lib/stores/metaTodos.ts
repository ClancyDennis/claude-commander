/**
 * Meta-Agent Todo Store
 *
 * Stores the System Commander's task list for tracking orchestration progress.
 * Unlike the agent-specific taskProgress store (which derives from TodoWrite tool calls),
 * this store is updated directly via events from the UpdateMetaTodoList tool.
 */

import { writable, derived } from "svelte/store";
import type { MetaTodoItem } from "../types";

// Main store for todos
export const metaAgentTodos = writable<MetaTodoItem[]>([]);

// Derived store for summary statistics
export const metaTodoSummary = derived(metaAgentTodos, ($todos) => {
  const total = $todos.length;
  const completed = $todos.filter((t) => t.status === "completed").length;
  const inProgress = $todos.filter((t) => t.status === "in_progress").length;
  const pending = $todos.filter((t) => t.status === "pending").length;
  const progress = total > 0 ? Math.round((completed / total) * 100) : 0;

  return {
    total,
    completed,
    inProgress,
    pending,
    progress,
  };
});

// Get the current active task (in_progress status)
export const currentMetaTask = derived(metaAgentTodos, ($todos) => {
  return $todos.find((t) => t.status === "in_progress") || null;
});

// Helper functions
export function setMetaTodos(todos: MetaTodoItem[]) {
  metaAgentTodos.set(todos);
}

export function clearMetaTodos() {
  metaAgentTodos.set([]);
}
