<script lang="ts">
  let {
    original,
    improved,
    viewMode = "side-by-side",
  }: {
    original: string;
    improved: string;
    viewMode?: "side-by-side" | "inline";
  } = $props();

  let localViewMode = $state(viewMode);

  // Simple line-by-line diff computation
  function computeDiff(orig: string, impr: string) {
    const origLines = orig.split('\n');
    const imprLines = impr.split('\n');

    const diff: Array<{
      type: 'unchanged' | 'removed' | 'added' | 'modified';
      original?: string;
      improved?: string;
      lineNum: number;
    }> = [];

    const maxLen = Math.max(origLines.length, imprLines.length);

    for (let i = 0; i < maxLen; i++) {
      const origLine = origLines[i];
      const imprLine = imprLines[i];

      if (origLine === undefined && imprLine !== undefined) {
        diff.push({ type: 'added', improved: imprLine, lineNum: i + 1 });
      } else if (origLine !== undefined && imprLine === undefined) {
        diff.push({ type: 'removed', original: origLine, lineNum: i + 1 });
      } else if (origLine === imprLine) {
        diff.push({ type: 'unchanged', original: origLine, improved: imprLine, lineNum: i + 1 });
      } else {
        diff.push({ type: 'modified', original: origLine, improved: imprLine, lineNum: i + 1 });
      }
    }

    return diff;
  }

  let diff = $derived(computeDiff(original, improved));

  let changeStats = $derived({
    added: diff.filter(d => d.type === 'added').length,
    removed: diff.filter(d => d.type === 'removed').length,
    modified: diff.filter(d => d.type === 'modified').length,
  });
</script>

<div class="diff-view">
  <div class="diff-header">
    <div class="diff-stats">
      {#if changeStats.added > 0}
        <span class="stat added">+{changeStats.added} added</span>
      {/if}
      {#if changeStats.removed > 0}
        <span class="stat removed">-{changeStats.removed} removed</span>
      {/if}
      {#if changeStats.modified > 0}
        <span class="stat modified">~{changeStats.modified} modified</span>
      {/if}
      {#if changeStats.added === 0 && changeStats.removed === 0 && changeStats.modified === 0}
        <span class="stat unchanged">No changes</span>
      {/if}
    </div>
    <div class="view-toggle">
      <button
        class:active={localViewMode === 'side-by-side'}
        onclick={() => localViewMode = 'side-by-side'}
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="3" width="8" height="18" rx="1"/>
          <rect x="13" y="3" width="8" height="18" rx="1"/>
        </svg>
        Side-by-side
      </button>
      <button
        class:active={localViewMode === 'inline'}
        onclick={() => localViewMode = 'inline'}
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="3" width="18" height="18" rx="1"/>
          <line x1="3" y1="9" x2="21" y2="9"/>
          <line x1="3" y1="15" x2="21" y2="15"/>
        </svg>
        Inline
      </button>
    </div>
  </div>

  {#if localViewMode === 'side-by-side'}
    <div class="side-by-side">
      <div class="panel original-panel">
        <div class="panel-header">Original</div>
        <div class="panel-content">
          {#each diff as line}
            <div class="line {line.type === 'removed' || line.type === 'modified' ? 'highlight-removed' : ''}">
              <span class="line-num">{line.lineNum}</span>
              <span class="line-content">{line.original ?? ''}</span>
            </div>
          {/each}
        </div>
      </div>
      <div class="panel improved-panel">
        <div class="panel-header">Improved</div>
        <div class="panel-content">
          {#each diff as line}
            <div class="line {line.type === 'added' || line.type === 'modified' ? 'highlight-added' : ''}">
              <span class="line-num">{line.lineNum}</span>
              <span class="line-content">{line.improved ?? ''}</span>
            </div>
          {/each}
        </div>
      </div>
    </div>
  {:else}
    <div class="inline-view">
      {#each diff as line}
        {#if line.type === 'unchanged'}
          <div class="line">
            <span class="line-num">{line.lineNum}</span>
            <span class="line-marker">&nbsp;</span>
            <span class="line-content">{line.original}</span>
          </div>
        {:else if line.type === 'removed'}
          <div class="line highlight-removed">
            <span class="line-num">{line.lineNum}</span>
            <span class="line-marker">-</span>
            <span class="line-content">{line.original}</span>
          </div>
        {:else if line.type === 'added'}
          <div class="line highlight-added">
            <span class="line-num">{line.lineNum}</span>
            <span class="line-marker">+</span>
            <span class="line-content">{line.improved}</span>
          </div>
        {:else if line.type === 'modified'}
          <div class="line highlight-removed">
            <span class="line-num">{line.lineNum}</span>
            <span class="line-marker">-</span>
            <span class="line-content">{line.original}</span>
          </div>
          <div class="line highlight-added">
            <span class="line-num">{line.lineNum}</span>
            <span class="line-marker">+</span>
            <span class="line-content">{line.improved}</span>
          </div>
        {/if}
      {/each}
    </div>
  {/if}
</div>

<style>
  .diff-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 10px;
    overflow: hidden;
  }

  .diff-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-sm) var(--space-md);
    background: var(--bg-tertiary);
    border-bottom: 1px solid var(--border);
  }

  .diff-stats {
    display: flex;
    gap: var(--space-md);
  }

  .stat {
    font-size: 12px;
    font-weight: 500;
  }

  .stat.added { color: #22c55e; }
  .stat.removed { color: #ef4444; }
  .stat.modified { color: #f59e0b; }
  .stat.unchanged { color: var(--text-muted); }

  .view-toggle {
    display: flex;
    gap: 2px;
    background: var(--bg-secondary);
    border-radius: 6px;
    padding: 2px;
  }

  .view-toggle button {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 500;
    color: var(--text-secondary);
    background: transparent;
    border: none;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .view-toggle button:hover {
    color: var(--text-primary);
  }

  .view-toggle button.active {
    background: var(--bg-elevated);
    color: var(--text-primary);
  }

  .view-toggle button svg {
    width: 14px;
    height: 14px;
  }

  .side-by-side {
    display: grid;
    grid-template-columns: 1fr 1fr;
    flex: 1;
    overflow: hidden;
  }

  .panel {
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .original-panel {
    border-right: 1px solid var(--border);
  }

  .panel-header {
    padding: var(--space-sm) var(--space-md);
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--text-muted);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
  }

  .panel-content, .inline-view {
    flex: 1;
    overflow-y: auto;
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    font-size: 12px;
    line-height: 1.5;
  }

  .line {
    display: flex;
    padding: 1px 0;
    min-height: 20px;
  }

  .line-num {
    width: 40px;
    padding: 0 var(--space-sm);
    text-align: right;
    color: var(--text-muted);
    user-select: none;
    flex-shrink: 0;
  }

  .line-marker {
    width: 20px;
    text-align: center;
    flex-shrink: 0;
    font-weight: 600;
  }

  .line-content {
    flex: 1;
    padding: 0 var(--space-sm);
    white-space: pre-wrap;
    word-break: break-word;
    color: var(--text-primary);
  }

  .highlight-removed {
    background: rgba(239, 68, 68, 0.1);
  }

  .highlight-removed .line-marker {
    color: #ef4444;
  }

  .highlight-added {
    background: rgba(34, 197, 94, 0.1);
  }

  .highlight-added .line-marker {
    color: #22c55e;
  }
</style>
