<script lang="ts">
  import type { InstructionSuggestion, SuggestionStatus } from "../../types";

  let {
    suggestion,
    status = "pending",
    onAccept,
    onReject,
  }: {
    suggestion: InstructionSuggestion;
    status: SuggestionStatus;
    onAccept: () => void;
    onReject: () => void;
  } = $props();

  const categoryColors: Record<string, string> = {
    clarity: "#f58573",
    structure: "#3b82f6",
    completeness: "#10b981",
    specificity: "#f59e0b",
    formatting: "#6366f1",
    other: "#6b7280",
  };

  let categoryColor = $derived(categoryColors[suggestion.category] || categoryColors.other);

  let statusClass = $derived(
    status === "accepted" ? "accepted" : status === "rejected" ? "rejected" : ""
  );
</script>

<div class="suggestion-item {statusClass}">
  <div class="suggestion-header">
    <span class="category-badge" style="background: {categoryColor};">
      {suggestion.category}
    </span>
    <span class="suggestion-title">{suggestion.title}</span>
    {#if status !== "pending"}
      <span class="status-indicator {status}">
        {status === "accepted" ? "Accepted" : "Rejected"}
      </span>
    {/if}
  </div>

  <p class="suggestion-description">{suggestion.description}</p>

  {#if suggestion.originalText && suggestion.suggestedText}
    <div class="text-comparison">
      <div class="original">
        <span class="label">Original:</span>
        <code>{suggestion.originalText}</code>
      </div>
      <div class="arrow">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M5 12h14M12 5l7 7-7 7"/>
        </svg>
      </div>
      <div class="suggested">
        <span class="label">Suggested:</span>
        <code>{suggestion.suggestedText}</code>
      </div>
    </div>
  {:else if suggestion.suggestedText}
    <div class="suggested-only">
      <span class="label">Suggested:</span>
      <code>{suggestion.suggestedText}</code>
    </div>
  {/if}

  {#if status === "pending"}
    <div class="actions">
      <button class="accept-btn" onclick={onAccept}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="20 6 9 17 4 12"/>
        </svg>
        Accept
      </button>
      <button class="reject-btn" onclick={onReject}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18"/>
          <line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
        Reject
      </button>
    </div>
  {/if}
</div>

<style>
  .suggestion-item {
    padding: var(--space-md);
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 8px;
    transition: all 0.2s ease;
  }

  .suggestion-item.accepted {
    background: rgba(34, 197, 94, 0.1);
    border-color: rgba(34, 197, 94, 0.3);
  }

  .suggestion-item.rejected {
    background: rgba(239, 68, 68, 0.05);
    border-color: rgba(239, 68, 68, 0.2);
    opacity: 0.7;
  }

  .suggestion-header {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    margin-bottom: var(--space-sm);
    flex-wrap: wrap;
  }

  .category-badge {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    padding: 2px 6px;
    border-radius: 4px;
    color: white;
  }

  .suggestion-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .status-indicator {
    font-size: 11px;
    font-weight: 500;
    padding: 2px 8px;
    border-radius: 4px;
    margin-left: auto;
  }

  .status-indicator.accepted {
    background: rgba(34, 197, 94, 0.2);
    color: #22c55e;
  }

  .status-indicator.rejected {
    background: rgba(239, 68, 68, 0.2);
    color: #ef4444;
  }

  .suggestion-description {
    font-size: 12px;
    color: var(--text-secondary);
    margin: 0 0 var(--space-md) 0;
    line-height: 1.5;
  }

  .text-comparison {
    display: flex;
    align-items: stretch;
    gap: var(--space-sm);
    margin-bottom: var(--space-md);
    font-size: 12px;
  }

  .original, .suggested, .suggested-only {
    flex: 1;
    padding: var(--space-sm);
    border-radius: 6px;
    background: var(--bg-tertiary);
  }

  .original {
    border-left: 3px solid #ef4444;
  }

  .suggested, .suggested-only {
    border-left: 3px solid #22c55e;
  }

  .label {
    display: block;
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--text-muted);
    margin-bottom: 4px;
  }

  code {
    display: block;
    font-family: 'Monaco', 'Menlo', monospace;
    font-size: 11px;
    color: var(--text-primary);
    white-space: pre-wrap;
    word-break: break-word;
  }

  .arrow {
    display: flex;
    align-items: center;
    color: var(--text-muted);
  }

  .arrow svg {
    width: 16px;
    height: 16px;
  }

  .actions {
    display: flex;
    gap: var(--space-sm);
  }

  .accept-btn, .reject-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 12px;
    border-radius: 6px;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
    border: 1px solid transparent;
  }

  .accept-btn {
    background: rgba(34, 197, 94, 0.15);
    color: #22c55e;
    border-color: rgba(34, 197, 94, 0.3);
  }

  .accept-btn:hover {
    background: rgba(34, 197, 94, 0.25);
  }

  .reject-btn {
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    border-color: var(--border);
  }

  .reject-btn:hover {
    background: rgba(239, 68, 68, 0.1);
    color: #ef4444;
    border-color: rgba(239, 68, 68, 0.3);
  }

  .accept-btn svg, .reject-btn svg {
    width: 14px;
    height: 14px;
  }
</style>
