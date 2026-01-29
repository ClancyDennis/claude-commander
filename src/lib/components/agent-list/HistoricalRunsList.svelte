<script lang="ts">
  import type { UnifiedHistoryItem } from '$lib/types';
  import { formatDate, formatDuration } from '$lib/utils/formatting';
  import { getStatusColor, getStatusLabel } from '$lib/utils/status';
  import { formatConversationDate } from '$lib/stores/metaConversations';
  import { MessageSquare } from '$lib/components/ui/icons';

  let {
    items,
    onSelectItem
  }: {
    items: UnifiedHistoryItem[];
    onSelectItem: (item: UnifiedHistoryItem) => void;
  } = $props();

  function truncateText(text: string, maxLength: number): string {
    return text.length > maxLength ? text.substring(0, maxLength) + '...' : text;
  }

  // Count items by type
  const runCount = $derived(items.filter(i => i.type === 'agent_run').length);
  const convCount = $derived(items.filter(i => i.type === 'conversation').length);
</script>

<div class="separator">
  <span>History ({items.length})</span>
  {#if runCount > 0 && convCount > 0}
    <span class="type-counts">{runCount} runs · {convCount} chats</span>
  {/if}
</div>

{#if items.length === 0}
  <div class="empty">
    <div class="empty-icon">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
        <path d="M3 3v5h5"/>
        <circle cx="12" cy="12" r="1"/>
      </svg>
    </div>
    <p class="empty-title">No history yet</p>
    <p class="empty-hint">Agent runs and conversations will appear here</p>
  </div>
{:else}
  <ul>
    {#each items as item (item.id)}
      <li>
        {#if item.type === 'conversation'}
          <!-- Conversation entry -->
          <button
            class="item-btn conversation-btn"
            onclick={() => onSelectItem(item)}
          >
            <div class="chat-icon">
              <MessageSquare size={14} />
            </div>
            <div class="info">
              <div class="name-row">
                <span class="name">{item.title}</span>
                <span class="chat-badge">CHAT</span>
              </div>
              <div class="meta-row">
                <span class="meta-text">
                  {item.messageCount} messages · {formatConversationDate(item.timestamp)}
                </span>
              </div>
              {#if item.preview}
                <div class="item-preview">
                  {truncateText(item.preview, 60)}
                </div>
              {/if}
            </div>
          </button>
        {:else}
          <!-- Agent run entry -->
          <button
            class="item-btn agent-btn"
            onclick={() => onSelectItem(item)}
          >
            <div class="status-indicator" style="background-color: {getStatusColor(item.status)}">
              {#if item.status === "running"}
                <span class="pulse"></span>
              {/if}
            </div>
            <div class="info">
              <div class="name-row">
                <span class="name">{item.title}</span>
                <span class="status-badge" style="background-color: {getStatusColor(item.status)}">
                  {getStatusLabel(item.status)}
                </span>
              </div>
              <div class="meta-row">
                <span class="meta-text">{formatDate(item.startedAt ?? item.timestamp)}</span>
                <span class="activity-time">{formatDuration(item.startedAt ?? item.timestamp, item.endedAt)}</span>
              </div>
              {#if item.preview}
                <div class="item-preview">
                  {truncateText(item.preview, 60)}
                </div>
              {/if}
            </div>
            <svg class="chevron" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="9,6 15,12 9,18"/>
            </svg>
          </button>
        {/if}
      </li>
    {/each}
  </ul>
{/if}

<style>
  ul {
    list-style: none;
    padding: var(--space-2);
  }

  li {
    padding: 0;
    margin-bottom: var(--space-2);
  }

  .item-btn {
    width: 100%;
    padding: var(--space-3) var(--space-4);
    display: flex;
    align-items: center;
    gap: var(--space-4);
    cursor: pointer;
    border-radius: var(--radius-md);
    transition: all var(--transition-fast);
    background-color: var(--bg-tertiary);
    border: 1px solid transparent;
    text-align: left;
    font: inherit;
    color: inherit;
  }

  .item-btn:hover {
    background-color: var(--bg-elevated);
    border-color: var(--border-hex);
  }

  /* Conversation-specific styling */
  .conversation-btn {
    border-left: 3px solid var(--accent-hex);
  }

  .conversation-btn:hover {
    border-left-color: var(--accent-hex);
  }

  .chat-icon {
    width: 24px;
    height: 24px;
    border-radius: var(--radius-full);
    background: var(--accent-hex);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    color: white;
  }

  .chat-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 2px var(--space-2);
    background-color: var(--accent-hex);
    color: white;
    font-size: var(--text-xs);
    font-weight: var(--font-semibold);
    border-radius: var(--radius-sm);
    flex-shrink: 0;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  /* Agent run styling */
  .status-indicator {
    width: 10px;
    height: 10px;
    border-radius: var(--radius-full);
    flex-shrink: 0;
    position: relative;
  }

  .pulse {
    position: absolute;
    inset: -3px;
    border-radius: var(--radius-full);
    background: inherit;
    opacity: 0.4;
    animation: pulse 2s ease-in-out infinite;
  }

  .status-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 2px var(--space-2);
    color: white;
    font-size: var(--text-xs);
    font-weight: var(--font-semibold);
    border-radius: var(--radius-sm);
    flex-shrink: 0;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  /* Shared info styling */
  .info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .name-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .name {
    font-weight: var(--font-medium);
    font-size: var(--text-base);
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
    flex: 1;
  }

  .meta-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
  }

  .meta-text {
    font-size: var(--text-sm);
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
  }

  .activity-time {
    font-size: var(--text-xs);
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .item-preview {
    font-size: var(--text-xs);
    color: var(--text-muted);
    margin-top: 2px;
    font-style: italic;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .chevron {
    width: 16px;
    height: 16px;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .separator {
    padding: var(--space-3) var(--space-4);
    font-size: var(--text-xs);
    font-weight: var(--font-semibold);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .type-counts {
    font-weight: var(--font-normal);
    text-transform: none;
  }

  .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-8);
    text-align: center;
    height: 100%;
    min-height: 300px;
  }

  .empty-icon {
    width: 64px;
    height: 64px;
    border-radius: var(--radius-lg);
    background: var(--bg-tertiary);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: var(--space-4);
    border: 1px solid var(--border-hex);
  }

  .empty-icon svg {
    width: 32px;
    height: 32px;
    color: var(--text-muted);
  }

  .empty-title {
    font-size: var(--text-lg);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
    margin-bottom: var(--space-2);
  }

  .empty-hint {
    font-size: var(--text-sm);
    color: var(--text-muted);
  }

  @keyframes pulse {
    0%, 100% { transform: scale(1); opacity: 0.4; }
    50% { transform: scale(1.5); opacity: 0; }
  }
</style>
