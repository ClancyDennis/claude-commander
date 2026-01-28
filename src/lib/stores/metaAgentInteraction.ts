/**
 * Meta-Agent Interaction Store
 *
 * Stores state for meta-agent interaction tools:
 * - Pending questions from AskUserQuestion
 * - User updates from UpdateUser
 * - Sleep status from Sleep
 */

import { writable, derived } from "svelte/store";
import type {
  MetaAgentUserUpdateEvent,
  MetaAgentQuestionEvent,
  MetaAgentSleepEvent,
} from "../types";

// Pending question from AskUserQuestion tool
export const pendingMetaQuestion = writable<MetaAgentQuestionEvent | null>(null);

// Recent user updates from UpdateUser tool
export const metaUserUpdates = writable<MetaAgentUserUpdateEvent[]>([]);

// Sleep status from Sleep tool
export const metaSleepStatus = writable<MetaAgentSleepEvent | null>(null);

// Derived: is the meta-agent currently waiting for user input?
export const isWaitingForInput = derived(pendingMetaQuestion, ($question) => $question !== null);

// Derived: is the meta-agent currently sleeping?
export const isSleeping = derived(metaSleepStatus, ($status) => $status?.status === "sleeping");

// Helper functions
export function setMetaQuestion(question: MetaAgentQuestionEvent) {
  pendingMetaQuestion.set(question);
}

export function clearMetaQuestion() {
  pendingMetaQuestion.set(null);
}

export function addMetaUserUpdate(update: MetaAgentUserUpdateEvent) {
  metaUserUpdates.update((updates) => {
    // Keep last 50 updates
    const newUpdates = [...updates, update];
    if (newUpdates.length > 50) {
      return newUpdates.slice(-50);
    }
    return newUpdates;
  });
}

export function clearMetaUserUpdates() {
  metaUserUpdates.set([]);
}

export function setMetaSleepStatus(status: MetaAgentSleepEvent) {
  metaSleepStatus.set(status);
}

export function clearMetaSleepStatus() {
  metaSleepStatus.set(null);
}

// Clear all interaction state (useful when chat is cleared)
export function clearMetaInteractionState() {
  pendingMetaQuestion.set(null);
  metaUserUpdates.set([]);
  metaSleepStatus.set(null);
}
