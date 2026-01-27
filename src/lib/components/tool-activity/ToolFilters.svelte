<script lang="ts">
  import type { FilterType } from "$lib/utils/toolStats";

  let {
    filterType = $bindable(),
    selectedTool = $bindable(),
    searchQuery = $bindable(),
    uniqueTools,
  }: {
    filterType: FilterType;
    selectedTool: string | "all";
    searchQuery: string;
    uniqueTools: string[];
  } = $props();
</script>

<div class="filters">
  <div class="filter-row">
    <select bind:value={filterType} class="filter-select">
      <option value="all">All status</option>
      <option value="success">Success</option>
      <option value="failed">Failed</option>
      <option value="pending">Pending</option>
    </select>

    <select bind:value={selectedTool} class="filter-select">
      <option value="all">All tools</option>
      {#each uniqueTools as tool}
        <option value={tool}>{tool}</option>
      {/each}
    </select>
  </div>

  <div class="search-wrapper">
    <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <circle cx="11" cy="11" r="8"/>
      <line x1="21" y1="21" x2="16.65" y2="16.65"/>
    </svg>
    <input
      type="text"
      placeholder="Search..."
      bind:value={searchQuery}
      class="search-input"
    />
  </div>
</div>

<style>
  .filters {
    padding: var(--space-3) var(--space-4);
    border-bottom: 1px solid var(--border-hex);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .filter-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-2);
  }

  .filter-select {
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm);
    border: none;
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    cursor: pointer;
    transition: background var(--transition-fast);
  }

  .filter-select:hover {
    background-color: var(--bg-elevated);
  }

  .filter-select:focus {
    outline: none;
    box-shadow: 0 0 0 2px var(--accent-glow);
  }

  .search-wrapper {
    position: relative;
    display: flex;
    align-items: center;
  }

  .search-icon {
    position: absolute;
    left: var(--space-3);
    width: 14px;
    height: 14px;
    color: var(--text-muted);
    pointer-events: none;
  }

  .search-input {
    width: 100%;
    padding: var(--space-2) var(--space-3) var(--space-2) calc(var(--space-3) + 14px + var(--space-2));
    border-radius: var(--radius-sm);
    border: none;
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
    font-size: var(--text-sm);
    transition: background var(--transition-fast);
  }

  .search-input::placeholder {
    color: var(--text-muted);
  }

  .search-input:hover {
    background-color: var(--bg-elevated);
  }

  .search-input:focus {
    outline: none;
    box-shadow: 0 0 0 2px var(--accent-glow);
  }
</style>
