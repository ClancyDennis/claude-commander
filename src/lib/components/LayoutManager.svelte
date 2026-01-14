<script lang="ts">
  import { layoutMode, gridSize } from "../stores/agents";

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
      <label>Grid:</label>
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
</div>

<style>
  .layout-manager {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-sm) var(--space-md);
    background-color: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
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
    background: linear-gradient(135deg, var(--accent-glow) 0%, rgba(124, 58, 237, 0.2) 100%);
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

  .grid-size-controls label {
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
</style>
