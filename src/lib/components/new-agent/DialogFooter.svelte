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
    padding: var(--space-lg);
    display: flex;
    justify-content: flex-end;
    gap: var(--space-md);
    border-top: 1px solid var(--border);
    background-color: var(--bg-tertiary);
  }

  footer button {
    min-width: 120px;
    padding: 10px 20px;
    border-radius: 10px;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    border: none;
  }

  .secondary {
    background: var(--bg-elevated);
    color: var(--text-secondary);
    border: 1px solid var(--border);
  }

  .secondary:hover:not(:disabled) {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .primary {
    background: var(--accent);
    color: white;
  }

  .primary:hover:not(:disabled) {
    background: var(--accent-hover, #7c3aed);
    transform: scale(1.02);
  }

  button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .spinner {
    width: 18px;
    height: 18px;
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
    gap: var(--space-sm);
    padding: var(--space-sm) var(--space-md);
    background: linear-gradient(135deg, rgba(124, 58, 237, 0.1) 0%, rgba(147, 51, 234, 0.05) 100%);
    border: 1px solid rgba(124, 58, 237, 0.3);
    border-radius: 12px;
    animation: pulse-border 2s ease-in-out infinite;
  }

  @keyframes pulse-border {
    0%, 100% {
      border-color: rgba(124, 58, 237, 0.3);
      box-shadow: 0 0 0 0 rgba(124, 58, 237, 0);
    }
    50% {
      border-color: rgba(124, 58, 237, 0.6);
      box-shadow: 0 0 12px 2px rgba(124, 58, 237, 0.15);
    }
  }

  .skill-gen-header {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    font-size: 14px;
    font-weight: 600;
    color: var(--accent);
  }

  .skill-gen-spinner {
    width: 16px;
    height: 16px;
    border: 2px solid rgba(124, 58, 237, 0.3);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  .skill-gen-details {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    font-size: 12px;
    color: var(--text-secondary);
  }

  .skill-gen-count {
    font-weight: 600;
    color: var(--text-primary);
    font-variant-numeric: tabular-nums;
  }

  .skill-gen-current {
    color: var(--text-muted);
    font-family: var(--font-mono, monospace);
    font-size: 11px;
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .skill-gen-bar {
    height: 4px;
    background: rgba(124, 58, 237, 0.2);
    border-radius: 2px;
    overflow: hidden;
  }

  .skill-gen-bar-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--accent) 0%, #9333ea 100%);
    border-radius: 2px;
    transition: width 0.3s ease;
    position: relative;
  }

  .skill-gen-bar-fill::after {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(
      90deg,
      transparent 0%,
      rgba(255, 255, 255, 0.3) 50%,
      transparent 100%
    );
    animation: shimmer 1.5s ease-in-out infinite;
  }

  @keyframes shimmer {
    0% { transform: translateX(-100%); }
    100% { transform: translateX(100%); }
  }
</style>
