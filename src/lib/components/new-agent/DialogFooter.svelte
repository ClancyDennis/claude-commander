<script lang="ts">
  import type { SkillGenerationState } from "../../hooks/useSkillGeneration.svelte";

  let {
    creationType,
    isCreating,
    canCreate,
    skillGenState,
    onCreate,
    onCancel,
  }: {
    creationType: 'agent' | 'pipeline' | 'auto-pipeline';
    isCreating: boolean;
    canCreate: boolean;
    skillGenState: SkillGenerationState;
    onCreate: () => void;
    onCancel: () => void;
  } = $props();
</script>

<footer>
  {#if skillGenState.active}
    <div class="skill-generation-progress">
      <div class="skill-gen-header">
        <span class="skill-gen-spinner"></span>
        <span>Generating skills...</span>
      </div>
      <div class="skill-gen-details">
        <span class="skill-gen-count">{skillGenState.completed + skillGenState.skipped}/{skillGenState.total}</span>
        {#if skillGenState.currentFile}
          <span class="skill-gen-current" title={skillGenState.currentFile}>
            {skillGenState.currentFile.length > 30
              ? '...' + skillGenState.currentFile.slice(-27)
              : skillGenState.currentFile}
          </span>
        {/if}
      </div>
      <div class="skill-gen-bar">
        <div
          class="skill-gen-bar-fill"
          style="width: {((skillGenState.completed + skillGenState.skipped) / skillGenState.total) * 100}%"
        ></div>
      </div>
    </div>
  {:else}
    <button class="secondary" onclick={onCancel} disabled={isCreating}>
      Cancel
    </button>
    <button
      class="primary"
      onclick={onCreate}
      disabled={isCreating || !canCreate}
    >
      {#if isCreating}
        <span class="spinner"></span>
        {creationType === 'pipeline' ? 'Starting...' : creationType === 'auto-pipeline' ? 'Starting...' : 'Creating...'}
      {:else}
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          {#if creationType === 'pipeline'}
            <polygon points="5 3 19 12 5 21 5 3"/>
          {:else}
            <line x1="12" y1="5" x2="12" y2="19"/>
            <line x1="5" y1="12" x2="19" y2="12"/>
          {/if}
        </svg>
        {creationType === 'pipeline' ? 'Start Pipeline' : creationType === 'auto-pipeline' ? 'Start Ralphline' : 'Create Agent'}
      {/if}
    </button>
  {/if}
</footer>

<style>
  footer {
    flex-shrink: 0;
    padding: var(--space-4) var(--space-5);
    display: flex;
    justify-content: flex-end;
    align-items: center;
    gap: var(--space-3);
    border-top: 1px solid var(--border);
    background-color: rgba(0, 0, 0, 0.2);
  }

  footer button {
    min-width: 100px;
    padding: var(--space-2) var(--space-4);
    border-radius: var(--radius-md);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    cursor: pointer;
    transition: all var(--transition-fast);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    border: none;
  }

  footer button svg {
    width: 14px;
    height: 14px;
    flex-shrink: 0;
  }

  .secondary {
    background: var(--bg-tertiary);
    color: var(--text-secondary);
  }

  .secondary:hover:not(:disabled) {
    background: var(--bg-elevated);
    color: var(--text-primary);
  }

  .primary {
    background: var(--accent-hex);
    color: white;
  }

  .primary:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .spinner {
    width: 14px;
    height: 14px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  /* Skill generation progress styles */
  .skill-generation-progress {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    background: rgba(232, 102, 77, 0.08);
    border: 1px solid rgba(232, 102, 77, 0.2);
    border-radius: var(--radius-md);
    min-width: 0;
  }

  .skill-gen-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--accent-hex);
  }

  .skill-gen-spinner {
    width: 14px;
    height: 14px;
    border: 2px solid rgba(232, 102, 77, 0.3);
    border-top-color: var(--accent-hex);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    flex-shrink: 0;
  }

  .skill-gen-details {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    font-size: var(--text-xs);
    color: var(--text-secondary);
    min-width: 0;
  }

  .skill-gen-count {
    font-weight: var(--font-medium);
    color: var(--text-primary);
    font-variant-numeric: tabular-nums;
    flex-shrink: 0;
  }

  .skill-gen-current {
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: 10px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }

  .skill-gen-bar {
    height: 3px;
    background: rgba(232, 102, 77, 0.15);
    border-radius: 2px;
    overflow: hidden;
  }

  .skill-gen-bar-fill {
    height: 100%;
    background: var(--accent-hex);
    border-radius: 2px;
    transition: width 0.3s ease;
  }
</style>
