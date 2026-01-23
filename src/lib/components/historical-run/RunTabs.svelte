<script lang="ts">
  import type { TabType } from './types';

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
    border-bottom: 1px solid var(--border-hex);
    background: var(--bg-secondary);
    flex-shrink: 0;
    padding: 0;
  }

  .main-tab {
    /* Override global button styles */
    flex: 1;
    padding: var(--space-3) var(--space-4);
    background: transparent;
    border: none;
    border-radius: 0;
    border-bottom: 2px solid transparent;
    color: var(--text-secondary);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    line-height: var(--leading-normal);
    cursor: pointer;
    transition: color var(--transition-fast), background var(--transition-fast), border-color var(--transition-fast);
    box-shadow: none;
  }

  .main-tab:hover {
    color: var(--text-primary);
    background: rgba(255, 255, 255, 0.04);
    box-shadow: none;
  }

  .main-tab:active {
    transform: none;
  }

  .main-tab.active {
    color: var(--accent-hex);
    border-bottom-color: var(--accent-hex);
    background: transparent;
  }
</style>
