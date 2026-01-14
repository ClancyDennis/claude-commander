<script lang="ts">
  import type { AgentOutput } from "../types";

  let {
    outputs,
    onFilter,
    onExport,
  }: {
    outputs: AgentOutput[];
    onFilter: (filtered: AgentOutput[]) => void;
    onExport: () => void;
  } = $props();

  let searchQuery = $state("");
  let filterType = $state<string>("all");

  // Get unique output types for filter dropdown
  const outputTypes = $derived.by(() => {
    const types = new Set<string>();
    outputs.forEach(o => types.add(o.type));
    return Array.from(types).sort();
  });

  // Apply filters
  const filteredOutputs = $derived.by(() => {
    let filtered = outputs; // No spread unless we need to filter

    // Filter by type
    if (filterType !== "all") {
      filtered = filtered.filter(o => o.type === filterType);
    }

    // Filter by search query
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(o =>
        o.content.toLowerCase().includes(query) ||
        o.type.toLowerCase().includes(query)
      );
    }

    return filtered;
  });

  // Notify parent whenever filtered results change
  $effect(() => {
    onFilter(filteredOutputs);
  });

  // Statistics
  const stats = $derived.by(() => {
    return {
      total: outputs.length,
      filtered: filteredOutputs.length,
      byType: outputs.reduce((acc, o) => {
        acc[o.type] = (acc[o.type] || 0) + 1;
        return acc;
      }, {} as Record<string, number>),
    };
  });

  function clearFilters() {
    searchQuery = "";
    filterType = "all";
  }

  function handleExport() {
    onExport();
  }
</script>

<div class="output-controls">
  <div class="controls-row">
    <div class="search-wrapper">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="search-icon">
        <circle cx="11" cy="11" r="8"/>
        <path d="m21 21-4.35-4.35"/>
      </svg>
      <input
        type="text"
        bind:value={searchQuery}
        placeholder="Search output..."
        class="search-input"
      />
      {#if searchQuery}
        <button class="clear-search" onclick={() => searchQuery = ""} title="Clear search">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      {/if}
    </div>

    <div class="filter-group">
      <label for="filter-type" class="filter-label">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3"/>
        </svg>
        Filter:
      </label>
      <select id="filter-type" bind:value={filterType} class="filter-select">
        <option value="all">All Types ({stats.total})</option>
        {#each outputTypes as type}
          <option value={type}>
            {type} ({stats.byType[type]})
          </option>
        {/each}
      </select>
    </div>

    <div class="action-buttons">
      {#if searchQuery || filterType !== "all"}
        <button class="secondary small" onclick={clearFilters}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
          Clear Filters
        </button>
      {/if}

      <button class="secondary small" onclick={handleExport} title="Export output">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
          <polyline points="7 10 12 15 17 10"/>
          <line x1="12" y1="15" x2="12" y2="3"/>
        </svg>
        Export
      </button>
    </div>
  </div>

  {#if stats.filtered < stats.total}
    <div class="results-info">
      Showing {stats.filtered} of {stats.total} outputs
    </div>
  {/if}
</div>

<style>
  .output-controls {
    padding: var(--space-md) var(--space-lg);
    background-color: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
  }

  .controls-row {
    display: flex;
    gap: var(--space-md);
    align-items: center;
    flex-wrap: wrap;
  }

  .search-wrapper {
    position: relative;
    flex: 1;
    min-width: 200px;
    max-width: 400px;
  }

  .search-icon {
    position: absolute;
    left: 12px;
    top: 50%;
    transform: translateY(-50%);
    width: 18px;
    height: 18px;
    color: var(--text-muted);
    pointer-events: none;
  }

  .search-input {
    width: 100%;
    padding: 10px 40px 10px 40px;
    border: 1px solid var(--border);
    border-radius: 10px;
    background-color: var(--bg-primary);
    color: var(--text-primary);
    font-size: 14px;
    transition: all 0.2s ease;
  }

  .search-input:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-glow);
  }

  .clear-search {
    position: absolute;
    right: 8px;
    top: 50%;
    transform: translateY(-50%);
    padding: 4px;
    background: none;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    color: var(--text-muted);
    transition: all 0.2s ease;
  }

  .clear-search:hover {
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .clear-search svg {
    width: 16px;
    height: 16px;
  }

  .filter-group {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .filter-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .filter-label svg {
    width: 16px;
    height: 16px;
  }

  .filter-select {
    padding: 8px 12px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background-color: var(--bg-primary);
    color: var(--text-primary);
    font-size: 14px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .filter-select:focus {
    outline: none;
    border-color: var(--accent);
  }

  .action-buttons {
    display: flex;
    gap: var(--space-sm);
    margin-left: auto;
  }

  button.small {
    padding: 8px 14px;
    font-size: 13px;
  }

  button.small svg {
    width: 16px;
    height: 16px;
  }

  .results-info {
    margin-top: var(--space-sm);
    font-size: 13px;
    color: var(--text-muted);
    font-weight: 500;
  }
</style>
