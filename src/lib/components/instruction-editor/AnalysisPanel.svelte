<script lang="ts">
  import type { InstructionAnalysisResult, SuggestionStatus } from "../../types";
  import IssueItem from "./IssueItem.svelte";
  import SuggestionItem from "./SuggestionItem.svelte";

  let {
    analysis,
    suggestionStatuses = $bindable(new Map()),
    onAcceptAll,
    onRejectAll,
  }: {
    analysis: InstructionAnalysisResult;
    suggestionStatuses: Map<string, SuggestionStatus>;
    onAcceptAll: () => void;
    onRejectAll: () => void;
  } = $props();

  function acceptSuggestion(id: string) {
    suggestionStatuses.set(id, "accepted");
    suggestionStatuses = new Map(suggestionStatuses);
  }

  function rejectSuggestion(id: string) {
    suggestionStatuses.set(id, "rejected");
    suggestionStatuses = new Map(suggestionStatuses);
  }

  let acceptedCount = $derived(
    Array.from(suggestionStatuses.values()).filter((s) => s === "accepted").length
  );

  let pendingCount = $derived(
    analysis.suggestions.length -
    Array.from(suggestionStatuses.values()).filter((s) => s !== "pending").length
  );

  // Quality score color
  let scoreColor = $derived(
    analysis.qualityScore >= 8 ? "#22c55e" :
    analysis.qualityScore >= 6 ? "#f59e0b" :
    analysis.qualityScore >= 4 ? "#f97316" :
    "#ef4444"
  );
</script>

<div class="analysis-panel">
  <!-- Quality Score -->
  <div class="quality-section">
    <div class="quality-header">
      <span class="quality-label">Quality Score</span>
      <span class="quality-score" style="color: {scoreColor};">
        {analysis.qualityScore}/10
      </span>
    </div>
    <div class="quality-bar">
      <div
        class="quality-fill"
        style="width: {analysis.qualityScore * 10}%; background: {scoreColor};"
      ></div>
    </div>
    <p class="quality-summary">{analysis.qualitySummary}</p>
  </div>

  <!-- Issues Section -->
  {#if analysis.issues.length > 0}
    <div class="section">
      <h3 class="section-title">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"/>
          <line x1="12" y1="8" x2="12" y2="12"/>
          <line x1="12" y1="16" x2="12.01" y2="16"/>
        </svg>
        Issues ({analysis.issues.length})
      </h3>
      <div class="issues-list">
        {#each analysis.issues as issue}
          <IssueItem {issue} />
        {/each}
      </div>
    </div>
  {/if}

  <!-- Suggestions Section -->
  {#if analysis.suggestions.length > 0}
    <div class="section">
      <div class="section-header">
        <h3 class="section-title">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 2L2 7l10 5 10-5-10-5z"/>
            <path d="M2 17l10 5 10-5"/>
            <path d="M2 12l10 5 10-5"/>
          </svg>
          Suggestions ({analysis.suggestions.length})
        </h3>
        <div class="bulk-actions">
          {#if pendingCount > 0}
            <button class="bulk-btn accept" onclick={onAcceptAll}>
              Accept All
            </button>
            <button class="bulk-btn reject" onclick={onRejectAll}>
              Reject All
            </button>
          {:else}
            <span class="accepted-count">
              {acceptedCount} accepted
            </span>
          {/if}
        </div>
      </div>
      <div class="suggestions-list">
        {#each analysis.suggestions as suggestion}
          <SuggestionItem
            {suggestion}
            status={suggestionStatuses.get(suggestion.id) || "pending"}
            onAccept={() => acceptSuggestion(suggestion.id)}
            onReject={() => rejectSuggestion(suggestion.id)}
          />
        {/each}
      </div>
    </div>
  {/if}

  {#if analysis.issues.length === 0 && analysis.suggestions.length === 0}
    <div class="empty-state">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/>
        <polyline points="22 4 12 14.01 9 11.01"/>
      </svg>
      <p>Your instruction looks good! No issues or suggestions.</p>
    </div>
  {/if}
</div>

<style>
  .analysis-panel {
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
    height: 100%;
    overflow-y: auto;
  }

  .quality-section {
    padding: var(--space-md);
    background: var(--bg-elevated);
    border-radius: 10px;
    border: 1px solid var(--border);
  }

  .quality-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-sm);
  }

  .quality-label {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .quality-score {
    font-size: 20px;
    font-weight: 700;
  }

  .quality-bar {
    height: 8px;
    background: var(--bg-tertiary);
    border-radius: 4px;
    overflow: hidden;
    margin-bottom: var(--space-sm);
  }

  .quality-fill {
    height: 100%;
    border-radius: 4px;
    transition: width 0.3s ease;
  }

  .quality-summary {
    font-size: 13px;
    color: var(--text-secondary);
    margin: 0;
    line-height: 1.5;
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .section-title {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .section-title svg {
    width: 18px;
    height: 18px;
    color: var(--accent);
  }

  .bulk-actions {
    display: flex;
    gap: var(--space-sm);
  }

  .bulk-btn {
    padding: 4px 10px;
    border-radius: 6px;
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
    border: 1px solid transparent;
  }

  .bulk-btn.accept {
    background: rgba(34, 197, 94, 0.15);
    color: #22c55e;
    border-color: rgba(34, 197, 94, 0.3);
  }

  .bulk-btn.accept:hover {
    background: rgba(34, 197, 94, 0.25);
  }

  .bulk-btn.reject {
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    border-color: var(--border);
  }

  .bulk-btn.reject:hover {
    background: rgba(239, 68, 68, 0.1);
    color: #ef4444;
  }

  .accepted-count {
    font-size: 12px;
    color: #22c55e;
    font-weight: 500;
  }

  .issues-list, .suggestions-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-xl);
    text-align: center;
    color: var(--text-muted);
  }

  .empty-state svg {
    width: 48px;
    height: 48px;
    color: #22c55e;
    margin-bottom: var(--space-md);
  }

  .empty-state p {
    font-size: 14px;
    margin: 0;
  }
</style>
