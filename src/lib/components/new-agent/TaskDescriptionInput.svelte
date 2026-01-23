<script lang="ts">
  import HelpTip from "./HelpTip.svelte";

  let {
    pipelineTask = $bindable(),
    creationType,
    isCreating
  }: {
    pipelineTask: string;
    creationType: 'agent' | 'pipeline' | 'auto-pipeline';
    isCreating: boolean;
  } = $props();
</script>

{#if creationType === 'pipeline' || creationType === 'auto-pipeline'}
  <label>
    <span class="label-text">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
      </svg>
      Task Description
      <HelpTip
        text="Describe the outcome you want. More context (stack, constraints, files) helps the pipeline do the right thing."
      />
    </span>
    <textarea
      bind:value={pipelineTask}
      placeholder={creationType === 'auto-pipeline'
        ? "What should Ralph build?"
        : "Describe the pipeline objective..."}
      rows="4"
      disabled={isCreating}
    ></textarea>
  </label>
{/if}

<style>
  label {
    display: block;
  }

  .label-text {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-2);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--text-primary);
  }

  .label-text svg {
    width: 16px;
    height: 16px;
    color: var(--accent-hex);
    flex-shrink: 0;
  }

  textarea {
    width: 100%;
    padding: var(--space-3);
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text-primary);
    font-size: var(--text-sm);
    font-family: inherit;
    line-height: 1.5;
    resize: vertical;
    transition: all var(--transition-fast);
  }

  textarea:focus {
    outline: none;
    border-color: var(--accent-hex);
    box-shadow: 0 0 0 3px rgba(232, 102, 77, 0.15);
  }

  textarea:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
