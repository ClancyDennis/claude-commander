<script lang="ts">
  import { onMount } from "svelte";
  import { IconButton } from "$lib/components/ui/button";
  import { Plus, Trash2, MessageSquare, X, Pencil, Check } from "$lib/components/ui/icons";
  import {
    conversations,
    currentConversationId,
    conversationsLoading,
    conversationsError,
    loadConversations,
    loadConversation,
    startNewConversation,
    deleteConversation,
    renameConversation,
    formatConversationDate,
    historyPanelOpen,
  } from "$lib/stores/metaConversations";
  import type { MetaConversation } from "$lib/types";

  let editingId: string | null = $state(null);
  let editingTitle: string = $state("");

  onMount(() => {
    loadConversations();
  });

  async function handleLoadConversation(conv: MetaConversation) {
    if (editingId) return; // Don't load while editing
    try {
      await loadConversation(conv.conversation_id);
    } catch (e) {
      console.error("Failed to load conversation:", e);
    }
  }

  async function handleNewConversation() {
    try {
      await startNewConversation();
    } catch (e) {
      console.error("Failed to start new conversation:", e);
    }
  }

  async function handleDeleteConversation(e: Event, conversationId: string) {
    e.stopPropagation();
    if (!confirm("Delete this conversation?")) return;
    try {
      await deleteConversation(conversationId);
    } catch (e) {
      console.error("Failed to delete conversation:", e);
    }
  }

  function startEditing(e: Event, conv: MetaConversation) {
    e.stopPropagation();
    editingId = conv.conversation_id;
    editingTitle = conv.title || conv.preview_text?.slice(0, 50) || "Untitled";
  }

  async function saveTitle(e: Event, conversationId: string) {
    e.stopPropagation();
    if (editingTitle.trim()) {
      try {
        await renameConversation(conversationId, editingTitle.trim());
      } catch (e) {
        console.error("Failed to rename conversation:", e);
      }
    }
    editingId = null;
  }

  function cancelEditing(e: Event) {
    e.stopPropagation();
    editingId = null;
  }

  function handleKeydown(e: KeyboardEvent, conversationId: string) {
    if (e.key === "Enter") {
      saveTitle(e, conversationId);
    } else if (e.key === "Escape") {
      cancelEditing(e);
    }
  }

  function closePanel() {
    historyPanelOpen.set(false);
  }
</script>

<div class="conversation-history">
  <div class="header">
    <div class="header-title">
      <MessageSquare size={16} />
      <span>Conversations</span>
    </div>
    <div class="header-actions">
      <IconButton
        icon={Plus}
        label="New"
        variant="ghost"
        size="sm"
        onclick={handleNewConversation}
        title="Start new conversation"
      />
      <IconButton
        icon={X}
        label="Close"
        variant="ghost"
        size="sm"
        onclick={closePanel}
        title="Close panel"
      />
    </div>
  </div>

  <div class="conversation-list">
    {#if $conversationsLoading && $conversations.length === 0}
      <div class="loading-state">Loading...</div>
    {:else if $conversationsError}
      <div class="error-state">{$conversationsError}</div>
    {:else if $conversations.length === 0}
      <div class="empty-state">
        <div class="empty-icon">💬</div>
        <div class="empty-text">No conversations yet</div>
        <div class="empty-hint">Start chatting to create one</div>
      </div>
    {:else}
      {#each $conversations as conv (conv.conversation_id)}
        <button
          class="conversation-item"
          class:active={$currentConversationId === conv.conversation_id}
          onclick={() => handleLoadConversation(conv)}
        >
          <div class="conversation-content">
            {#if editingId === conv.conversation_id}
              <input
                type="text"
                class="title-input"
                bind:value={editingTitle}
                onkeydown={(e) => handleKeydown(e, conv.conversation_id)}
                onclick={(e) => e.stopPropagation()}
              />
            {:else}
              <div class="conversation-title">
                {conv.title || conv.preview_text?.slice(0, 40) || "Untitled conversation"}
              </div>
            {/if}
            <div class="conversation-meta">
              <span class="message-count">{conv.message_count} messages</span>
              <span class="separator">·</span>
              <span class="timestamp">{formatConversationDate(conv.updated_at)}</span>
            </div>
          </div>
          <div class="conversation-actions">
            {#if editingId === conv.conversation_id}
              <IconButton
                icon={Check}
                label="Save"
                variant="ghost"
                size="sm"
                onclick={(e) => saveTitle(e, conv.conversation_id)}
              />
              <IconButton
                icon={X}
                label="Cancel"
                variant="ghost"
                size="sm"
                onclick={cancelEditing}
              />
            {:else}
              <IconButton
                icon={Pencil}
                label="Rename"
                variant="ghost"
                size="sm"
                onclick={(e) => startEditing(e, conv)}
              />
              <IconButton
                icon={Trash2}
                label="Delete"
                variant="ghost"
                size="sm"
                onclick={(e) => handleDeleteConversation(e, conv.conversation_id)}
              />
            {/if}
          </div>
        </button>
      {/each}
    {/if}
  </div>
</div>

<style>
  .conversation-history {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border-hex);
    width: 280px;
    min-width: 280px;
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-3) var(--space-4);
    border-bottom: 1px solid var(--border-hex);
    background: var(--bg-tertiary);
  }

  .header-title {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-sm);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: var(--space-1);
  }

  .conversation-list {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-2);
  }

  .conversation-item {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    width: 100%;
    padding: var(--space-3);
    margin-bottom: var(--space-1);
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition-fast);
    text-align: left;
  }

  .conversation-item:hover {
    background: var(--bg-tertiary);
    border-color: var(--border-hex);
  }

  .conversation-item.active {
    background: var(--bg-elevated);
    border-color: var(--accent-hex);
  }

  .conversation-content {
    flex: 1;
    min-width: 0;
    overflow: hidden;
  }

  .conversation-title {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-bottom: var(--space-1);
  }

  .conversation-meta {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    font-size: var(--text-xs);
    color: var(--text-muted);
  }

  .separator {
    opacity: 0.5;
  }

  .conversation-actions {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    opacity: 0;
    transition: opacity var(--transition-fast);
  }

  .conversation-item:hover .conversation-actions,
  .conversation-item.active .conversation-actions {
    opacity: 1;
  }

  .title-input {
    width: 100%;
    padding: var(--space-1) var(--space-2);
    font-size: var(--text-sm);
    background: var(--bg-primary);
    border: 1px solid var(--accent-hex);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    outline: none;
  }

  .loading-state,
  .error-state {
    padding: var(--space-4);
    text-align: center;
    font-size: var(--text-sm);
    color: var(--text-muted);
  }

  .error-state {
    color: var(--error-hex);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-8) var(--space-4);
    text-align: center;
  }

  .empty-icon {
    font-size: 32px;
    margin-bottom: var(--space-2);
    opacity: 0.6;
  }

  .empty-text {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--text-secondary);
    margin-bottom: var(--space-1);
  }

  .empty-hint {
    font-size: var(--text-xs);
    color: var(--text-muted);
  }
</style>
