<script lang="ts">
  import type { AgentRun, AgentOutputRecord } from "$lib/types";
  import type { PromptData } from './dataLoader';
  import MarkdownRenderer from '../MarkdownRenderer.svelte';

  interface Props {
    run: AgentRun;
    prompts: PromptData[];
    outputs: AgentOutputRecord[];
    onResume?: () => void;
  }

  let { run, prompts, outputs, onResume }: Props = $props();

  // Get the first output for the final result section
  let finalOutput = $derived(outputs.length > 0 ? outputs[0] : null);
</script>

<div class="overview-content">
  <!-- First Prompt Section -->
  <div class="section">
    <h3>First Prompt</h3>
    {#if run.initial_prompt}
      <div class="prompt-content">
        <MarkdownRenderer content={run.initial_prompt} />
      </div>
    {:else if prompts.length > 0}
      <div class="prompt-content">
        <MarkdownRenderer content={prompts[0].prompt} />
      </div>
    {:else}
      <div class="empty-content">
        <p class="muted">No prompts recorded</p>
      </div>
    {/if}
  </div>

  <!-- Final Output/Result Section -->
  <div class="section">
    <h3>Final Result</h3>
    {#if finalOutput}
      <div class="result-content">
        <MarkdownRenderer content={finalOutput.content} />
      </div>
    {:else}
      <div class="empty-content">
        <p class="muted">No outputs recorded</p>
      </div>
    {/if}
  </div>

  <!-- Error Message Section (if any) -->
  {#if run.error_message}
    <div class="section">
      <h3>Error</h3>
      <div class="error-content">
        <MarkdownRenderer content={run.error_message} />
      </div>
    </div>
  {/if}

  <!-- Resume Section -->
  {#if run.can_resume}
    <div class="resume-section">
      <button class="primary resume-btn" onclick={onResume}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polygon points="5 3 19 12 5 21 5 3"/>
        </svg>
        Resume This Run
      </button>
      <p class="resume-hint">This run can be resumed from where it left off</p>
    </div>
  {/if}
</div>

<style>
  .overview-content {
    padding: var(--space-md);
  }

  .section {
    margin-bottom: var(--space-lg);
  }

  h3 {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0 0 var(--space-sm) 0;
  }

  .prompt-content,
  .error-content,
  .result-content {
    background-color: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: var(--space-md);
    color: var(--text-primary);
  }

  .result-content {
    max-height: 400px;
    overflow-y: auto;
  }

  .error-content {
    border-color: var(--error);
    background-color: rgba(239, 68, 68, 0.1);
  }

  .empty-content {
    background-color: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: var(--space-lg);
    text-align: center;
  }

  .empty-content .muted {
    color: var(--text-muted);
    margin: 0;
    font-style: italic;
  }

  .resume-section {
    padding: var(--space-lg);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-sm);
  }

  .resume-btn {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: 12px 24px;
    font-size: 14px;
  }

  .resume-btn svg {
    width: 16px;
    height: 16px;
  }

  .resume-hint {
    font-size: 12px;
    color: var(--text-muted);
    margin: 0;
  }
</style>
