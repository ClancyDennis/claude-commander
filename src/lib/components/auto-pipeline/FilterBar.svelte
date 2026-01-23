<script lang="ts">
  type StageFilter = 'all' | 'Orchestrator' | 'Planning' | 'Building' | 'Verifying';
  type TypeFilter = 'all' | 'text' | 'tool_use' | 'tool_result' | 'error' | 'orchestrator_tool' | 'state_change' | 'decision';

  type StageCounts = {
    Orchestrator: number;
    Planning: number;
    Building: number;
    Verifying: number;
  };

  type TypeCounts = {
    text: number;
    tool_use: number;
    tool_result: number;
    error: number;
    orchestrator_tool: number;
    state_change: number;
    decision: number;
  };

  let {
    stageFilter = $bindable('all'),
    typeFilter = $bindable('all'),
    searchQuery = $bindable(''),
    stageCounts,
    typeCounts,
    totalOutputs,
    filteredCount
  }: {
    stageFilter: StageFilter;
    typeFilter: TypeFilter;
    searchQuery: string;
    stageCounts: StageCounts;
    typeCounts: TypeCounts;
    totalOutputs: number;
    filteredCount: number;
  } = $props();

  const hasActiveFilters = $derived(stageFilter !== 'all' || typeFilter !== 'all' || searchQuery !== '');

  function clearFilters() {
    stageFilter = 'all';
    typeFilter = 'all';
    searchQuery = '';
  }
</script>

<div class="filter-bar">
  <div class="filter-group">
    <label class="filter-label" for="stage-filter">Stage</label>
    <select id="stage-filter" bind:value={stageFilter} class="filter-select">
      <option value="all">All ({totalOutputs})</option>
      <option value="Orchestrator">Orchestrator ({stageCounts.Orchestrator})</option>
      <option value="Planning">Planning ({stageCounts.Planning})</option>
      <option value="Building">Building ({stageCounts.Building})</option>
      <option value="Verifying">Verifying ({stageCounts.Verifying})</option>
    </select>
  </div>

  <div class="filter-group">
    <label class="filter-label" for="type-filter">Type</label>
    <select id="type-filter" bind:value={typeFilter} class="filter-select">
      <option value="all">All Types</option>
      <option value="text">Text ({typeCounts.text})</option>
      <option value="tool_use">Tool Use ({typeCounts.tool_use})</option>
      <option value="tool_result">Result ({typeCounts.tool_result})</option>
      <option value="error">Error ({typeCounts.error})</option>
      <option value="orchestrator_tool">Orchestrator Tool ({typeCounts.orchestrator_tool})</option>
      <option value="state_change">State Change ({typeCounts.state_change})</option>
      <option value="decision">Decision ({typeCounts.decision})</option>
    </select>
  </div>

  <div class="search-wrapper">
    <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="11" cy="11" r="8"/>
      <line x1="21" y1="21" x2="16.65" y2="16.65"/>
    </svg>
    <input
      type="text"
      placeholder="Search outputs..."
      bind:value={searchQuery}
      class="search-input"
    />
    {#if searchQuery}
      <button class="clear-search" onclick={() => searchQuery = ''}>x</button>
    {/if}
  </div>

  {#if hasActiveFilters}
    <button class="clear-filters-btn" onclick={clearFilters}>
      Clear Filters
    </button>
  {/if}

  <span class="results-count">
    {filteredCount} of {totalOutputs}
  </span>
</div>

<style>
  .filter-bar {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-sm) var(--space-lg);
    background: var(--bg-tertiary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    flex-wrap: wrap;
  }

  .filter-group {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
  }

  .filter-label {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
  }

  .filter-select {
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 6px 10px;
    font-size: 12px;
    color: var(--text-primary);
    cursor: pointer;
  }

  .filter-select:hover {
    border-color: var(--accent);
  }

  .search-wrapper {
    flex: 1;
    min-width: 150px;
    max-width: 300px;
    position: relative;
    display: flex;
    align-items: center;
  }

  .search-icon {
    position: absolute;
    left: 10px;
    width: 16px;
    height: 16px;
    color: var(--text-muted);
    pointer-events: none;
  }

  .search-input {
    width: 100%;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 6px 30px 6px 32px;
    font-size: 12px;
    color: var(--text-primary);
  }

  .search-input:focus {
    outline: none;
    border-color: var(--accent);
  }

  .clear-search {
    position: absolute;
    right: 8px;
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: 16px;
    cursor: pointer;
    padding: 0;
    line-height: 1;
  }

  .clear-search:hover {
    color: var(--text-primary);
  }

  .clear-filters-btn {
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 6px 12px;
    font-size: 12px;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .clear-filters-btn:hover {
    border-color: var(--accent);
    color: var(--accent);
  }

  .results-count {
    font-size: 12px;
    color: var(--text-muted);
    margin-left: auto;
  }
</style>
