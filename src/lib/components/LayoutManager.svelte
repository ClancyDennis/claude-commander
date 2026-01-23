<script lang="ts">
  import { layoutMode, gridSize, selectedAgentIds } from "../stores/agents";
  import HelpDialog from "./HelpDialog.svelte";
  import HelpTip from "./new-agent/HelpTip.svelte";

  let showHelp = $state(false);

  const selectedCount = $derived($selectedAgentIds.size);
  const showSelectionHint = $derived($layoutMode !== 'single' && selectedCount < 2);

  function cycleLayout() {
    layoutMode.update((current) => {
      if (current === 'single') return 'split';
      if (current === 'split') return 'grid';
      return 'single';
    });
  }

  function setLayout(mode: 'single' | 'split' | 'grid') {
    layoutMode.set(mode);
  }

  function updateGridSize(size: number) {
    gridSize.set(size);
  }
</script>

<div class="layout-manager">
  <div class="layout-controls">
    <HelpTip text="Switch between single, split, and grid layouts to view multiple agents at once. Use Ctrl+click (Cmd+click on Mac) to select multiple agents." placement="bottom" />
    <button
      class="layout-btn"
      class:active={$layoutMode === 'single'}
      onclick={() => setLayout('single')}
      title="Single Agent View"
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="3" y="3" width="18" height="18" rx="2"/>
      </svg>
    </button>
    <button
      class="layout-btn"
      class:active={$layoutMode === 'split'}
      onclick={() => setLayout('split')}
      title="Split View"
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="3" y="3" width="8" height="18" rx="2"/>
        <rect x="13" y="3" width="8" height="18" rx="2"/>
      </svg>
    </button>
    <button
      class="layout-btn"
      class:active={$layoutMode === 'grid'}
      onclick={() => setLayout('grid')}
      title="Grid View"
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="3" y="3" width="7" height="7" rx="1"/>
        <rect x="14" y="3" width="7" height="7" rx="1"/>
        <rect x="3" y="14" width="7" height="7" rx="1"/>
        <rect x="14" y="14" width="7" height="7" rx="1"/>
      </svg>
    </button>
  </div>

  {#if $layoutMode === 'grid'}
    <div class="grid-size-controls">
      <span>Grid:</span>
      <button
        class="size-btn"
        class:active={$gridSize === 2}
        onclick={() => updateGridSize(2)}
      >
        2x2
      </button>
      <button
        class="size-btn"
        class:active={$gridSize === 3}
        onclick={() => updateGridSize(3)}
      >
        3x3
      </button>
    </div>
  {/if}

  {#if $layoutMode !== 'single'}
    <div class="selection-info">
      <span class="selection-count">{selectedCount} selected</span>
      {#if showSelectionHint}
        <span class="selection-hint">Ctrl+click agents to add</span>
      {/if}
    </div>
  {/if}

  <button
    class="help-btn"
    onclick={() => showHelp = true}
    title="Help & Shortcuts"
  >
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="12" cy="12" r="10"/>
      <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"/>
      <line x1="12" y1="17" x2="12.01" y2="17"/>
    </svg>
    <span>Help</span>
  </button>
</div>

{#if showHelp}
  <HelpDialog onClose={() => showHelp = false} />
{/if}

<style>
  .layout-manager {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-sm) var(--space-md);
    background-color: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    position: relative;
  }

  .layout-controls {
    display: flex;
    gap: var(--space-sm);
  }

  .layout-btn {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 10px;
    cursor: pointer;
    color: var(--text-muted);
    transition: all 0.2s ease;
  }

  .layout-btn:hover {
    background-color: var(--bg-elevated);
    border-color: var(--border-light);
    color: var(--text-secondary);
  }

  .layout-btn.active {
    background: linear-gradient(135deg, var(--accent-glow) 0%, rgba(240, 112, 90, 0.2) 100%);
    border-color: var(--accent);
    color: var(--accent);
  }

  .layout-btn svg {
    width: 20px;
    height: 20px;
  }

  .grid-size-controls {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding-left: var(--space-md);
    border-left: 1px solid var(--border);
  }

  .grid-size-controls span {
    font-size: 13px;
    color: var(--text-muted);
    font-weight: 600;
  }

  .size-btn {
    padding: 8px 14px;
    font-size: 13px;
    font-weight: 600;
    background-color: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 8px;
    cursor: pointer;
    color: var(--text-muted);
    transition: all 0.2s ease;
  }

  .size-btn:hover {
    background-color: var(--bg-elevated);
    color: var(--text-secondary);
  }

  .size-btn.active {
    background-color: var(--accent-glow);
    border-color: var(--accent);
    color: var(--accent);
  }

  .selection-info {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: 6px 12px;
    background-color: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 8px;
  }

  .selection-count {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .selection-hint {
    font-size: 12px;
    color: var(--text-muted);
    padding-left: var(--space-sm);
    border-left: 1px solid var(--border);
  }

  .help-btn {
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 0 16px;
    background-color: var(--bg-tertiary);
    border: 1px solid var(--accent);
    border-radius: 20px;
    cursor: pointer;
    color: var(--accent);
    font-size: 14px;
    font-weight: 600;
    transition: all 0.2s ease;
    margin-left: auto;
  }

  .help-btn:hover {
    background-color: var(--accent-glow);
    border-color: var(--accent-hover);
    color: var(--accent-hover);
    transform: scale(1.02);
  }

  .help-btn svg {
    width: 18px;
    height: 18px;
  }
</style>

