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
    gap: var(--space-sm);
    margin-bottom: var(--space-sm);
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .label-text svg {
    width: 18px;
    height: 18px;
    color: var(--accent);
  }

  textarea {
    width: 100%;
    padding: var(--space-md);
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 10px;
    color: var(--text-primary);
    font-size: 14px;
    font-family: inherit;
    line-height: 1.5;
    resize: vertical;
    transition: all 0.2s ease;
  }

  textarea:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-glow);
  }

  textarea:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>
