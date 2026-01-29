/**
 * Meta Agent Conversation Store
 *
 * Manages conversation history persistence for the meta agent.
 * Allows users to view, load, and delete previous conversations.
 */

import { writable, derived, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import type { MetaConversation, ChatMessage, UnifiedHistoryItem } from "../types";
import { metaAgentChat, historicalRuns } from "./agents";
import { formatPath, formatDate } from "../utils/formatting";

// ============================================================================
// Stores
// ============================================================================

/** List of all conversations */
export const conversations = writable<MetaConversation[]>([]);

/** Currently active conversation ID */
export const currentConversationId = writable<string | null>(null);

/** Loading state for conversation operations */
export const conversationsLoading = writable<boolean>(false);

/** Error message if any operation fails */
export const conversationsError = writable<string | null>(null);

/** Whether the conversation history panel is visible */
export const historyPanelOpen = writable<boolean>(false);

/** Currently viewing conversation in read-only mode (not loaded into active chat) */
export const viewingConversation = writable<{
  id: string;
  title: string;
  messages: ChatMessage[];
} | null>(null);

// ============================================================================
// Derived Stores
// ============================================================================

/**
 * Unified history combining agent runs and meta conversations,
 * sorted by most recent activity first.
 */
export const unifiedHistory = derived(
  [historicalRuns, conversations],
  ([$historicalRuns, $conversations]): UnifiedHistoryItem[] => {
    const items: UnifiedHistoryItem[] = [];

    // Map agent runs to unified items
    for (const run of $historicalRuns) {
      items.push({
        type: 'agent_run',
        id: run.agent_id,
        title: formatPath(run.working_dir),
        preview: run.initial_prompt,
        timestamp: run.started_at,
        status: run.status,
        workingDir: run.working_dir,
        startedAt: run.started_at,
        endedAt: run.ended_at,
      });
    }

    // Map conversations to unified items
    for (const conv of $conversations) {
      items.push({
        type: 'conversation',
        id: conv.conversation_id,
        title: conv.title || conv.preview_text?.slice(0, 40) || 'Untitled conversation',
        preview: conv.preview_text,
        timestamp: conv.updated_at,
        messageCount: conv.message_count,
      });
    }

    // Sort by timestamp descending (most recent first)
    items.sort((a, b) => b.timestamp - a.timestamp);

    return items;
  }
);

// ============================================================================
// API Functions
// ============================================================================

/**
 * Load all conversations from the database
 */
export async function loadConversations(includeArchived = false): Promise<void> {
  conversationsLoading.set(true);
  conversationsError.set(null);

  try {
    const result = await invoke<MetaConversation[]>("list_conversations", {
      includeArchived,
      limit: 50,
    });
    conversations.set(result);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    conversationsError.set(message);
    console.error("[metaConversations] Failed to load conversations:", error);
  } finally {
    conversationsLoading.set(false);
  }
}

/**
 * Load a specific conversation and set it as current
 */
export async function loadConversation(conversationId: string): Promise<void> {
  conversationsLoading.set(true);
  conversationsError.set(null);

  try {
    const messages = await invoke<ChatMessage[]>("load_conversation", {
      conversationId,
    });

    // Update the chat messages store
    metaAgentChat.set(messages);
    currentConversationId.set(conversationId);

    console.log(`[metaConversations] Loaded conversation ${conversationId} with ${messages.length} messages`);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    conversationsError.set(message);
    console.error("[metaConversations] Failed to load conversation:", error);
    throw error;
  } finally {
    conversationsLoading.set(false);
  }
}

/**
 * Start a new conversation (clears current and creates a new DB entry)
 */
export async function startNewConversation(): Promise<string> {
  conversationsLoading.set(true);
  conversationsError.set(null);

  try {
    const newId = await invoke<string>("new_conversation");

    // Clear the chat messages
    metaAgentChat.set([]);
    currentConversationId.set(newId);

    // Refresh the conversations list
    await loadConversations();

    console.log(`[metaConversations] Started new conversation: ${newId}`);
    return newId;
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    conversationsError.set(message);
    console.error("[metaConversations] Failed to start new conversation:", error);
    throw error;
  } finally {
    conversationsLoading.set(false);
  }
}

/**
 * Delete a conversation
 */
export async function deleteConversation(conversationId: string): Promise<void> {
  conversationsLoading.set(true);
  conversationsError.set(null);

  try {
    await invoke("delete_conversation", { conversationId });

    // If this was the current conversation, clear it
    if (get(currentConversationId) === conversationId) {
      metaAgentChat.set([]);
      currentConversationId.set(null);
    }

    // Refresh the conversations list
    await loadConversations();

    console.log(`[metaConversations] Deleted conversation: ${conversationId}`);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    conversationsError.set(message);
    console.error("[metaConversations] Failed to delete conversation:", error);
    throw error;
  } finally {
    conversationsLoading.set(false);
  }
}

/**
 * Rename a conversation
 */
export async function renameConversation(
  conversationId: string,
  newTitle: string
): Promise<void> {
  conversationsError.set(null);

  try {
    await invoke("rename_conversation", { conversationId, newTitle });

    // Update local state
    conversations.update((convs) =>
      convs.map((c) =>
        c.conversation_id === conversationId ? { ...c, title: newTitle } : c
      )
    );

    console.log(`[metaConversations] Renamed conversation ${conversationId} to: ${newTitle}`);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    conversationsError.set(message);
    console.error("[metaConversations] Failed to rename conversation:", error);
    throw error;
  }
}

/**
 * Get the current conversation ID from the backend
 */
export async function syncCurrentConversationId(): Promise<void> {
  try {
    const id = await invoke<string | null>("get_current_conversation_id");
    currentConversationId.set(id);
  } catch (error) {
    console.error("[metaConversations] Failed to sync current conversation ID:", error);
  }
}

/**
 * Toggle the history panel visibility
 */
export function toggleHistoryPanel(): void {
  historyPanelOpen.update((open) => !open);
}

// ============================================================================
// Conversation View Mode (Read-Only History)
// ============================================================================

/**
 * Open a conversation in view mode (read-only, not loaded into active chat)
 */
export async function viewConversation(conversationId: string): Promise<void> {
  conversationsLoading.set(true);
  conversationsError.set(null);

  try {
    const messages = await invoke<ChatMessage[]>("load_conversation", {
      conversationId,
    });

    // Find the conversation title
    const convList = get(conversations);
    const conv = convList.find((c) => c.conversation_id === conversationId);
    const title = conv?.title || conv?.preview_text?.slice(0, 40) || "Conversation History";

    viewingConversation.set({
      id: conversationId,
      title,
      messages,
    });

    console.log(`[metaConversations] Viewing conversation ${conversationId} with ${messages.length} messages`);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    conversationsError.set(message);
    console.error("[metaConversations] Failed to view conversation:", error);
    throw error;
  } finally {
    conversationsLoading.set(false);
  }
}

/**
 * Close the conversation view mode
 * @param resume - If true, load the conversation into active chat before closing
 */
export async function closeConversationView(resume: boolean = false): Promise<void> {
  const viewing = get(viewingConversation);

  if (resume && viewing) {
    // Load into active chat
    await loadConversation(viewing.id);
  }

  // Clear the viewing state
  viewingConversation.set(null);
}

/**
 * Format a timestamp for display
 */
export function formatConversationDate(timestamp: number): string {
  const date = new Date(timestamp);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

  if (diffDays === 0) {
    // Today - show time
    return date.toLocaleTimeString(undefined, {
      hour: "numeric",
      minute: "2-digit",
    });
  } else if (diffDays === 1) {
    return "Yesterday";
  } else if (diffDays < 7) {
    return date.toLocaleDateString(undefined, { weekday: "long" });
  } else {
    return date.toLocaleDateString(undefined, {
      month: "short",
      day: "numeric",
    });
  }
}
