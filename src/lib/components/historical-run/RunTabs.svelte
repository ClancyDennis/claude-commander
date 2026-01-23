<script lang="ts">
  export type TabType = 'overview' | 'activity' | 'outputs' | 'prompts';

  interface Props {
    activeTab: TabType;
    hasPipeline: boolean;
    activityCount: number;
    outputsCount: number;
    promptsCount: number;
    onTabChange: (tab: TabType) => void;
  }

  let {
    activeTab,
    hasPipeline,
    activityCount,
    outputsCount,
    promptsCount,
    onTabChange
  }: Props = $props();
</script>

<div class="main-tabs">
  <button
    class="main-tab"
    class:active={activeTab === 'overview'}
    onclick={() => onTabChange('overview')}
  >
    Overview
  </button>
  {#if hasPipeline}
    <button
      class="main-tab"
      class:active={activeTab === 'activity'}
      onclick={() => onTabChange('activity')}
    >
      Activity ({activityCount})
    </button>
  {/if}
  <button
    class="main-tab"
    class:active={activeTab === 'outputs'}
    onclick={() => onTabChange('outputs')}
  >
    Outputs ({outputsCount})
  </button>
  <button
    class="main-tab"
    class:active={activeTab === 'prompts'}
    onclick={() => onTabChange('prompts')}
  >
    Prompts ({promptsCount})
  </button>
</div>

<style>
  .main-tabs {
    display: flex;
    border-bottom: 1px solid var(--border);
    background: var(--bg-secondary);
    flex-shrink: 0;
  }

  .main-tab {
    flex: 1;
    padding: 12px 16px;
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--text-secondary);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .main-tab:hover {
    color: var(--text-primary);
    background: var(--bg-tertiary);
  }

  .main-tab.active {
    color: var(--accent);
    border-bottom-color: var(--accent);
  }
</style>
