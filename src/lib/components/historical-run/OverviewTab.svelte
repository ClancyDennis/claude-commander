<script lang="ts">
  import type { AgentRun, AgentOutputRecord } from "$lib/types";
  import type { PromptData } from './dataLoader';
  import MarkdownRenderer from '../MarkdownRenderer.svelte';

  interface Props {
    run: AgentRun;
    prompts: PromptData[];
    outputs: AgentOutputRecord[];
    onResume?: (autoStart: boolean) => void;
  }

  let { run, prompts, outputs, onResume }: Props = $props();

  // Whether to start automatically when resuming (default: true)
  let autoStart = $state(true);

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
      <button class="primary resume-btn" onclick={() => onResume?.(autoStart)}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polygon points="5 3 19 12 5 21 5 3"/>
        </svg>
        Resume This Run
      </button>
      <label class="auto-start-label">
        <input type="checkbox" bind:checked={autoStart} />
        <span>Start automatically</span>
      </label>
      <p class="resume-hint">This run can be resumed from where it left off</p>
    </div>
  {/if}
</div>

<style>
  .overview-content {
    padding: var(--space-4);
  }

  .section {
    margin-bottom: var(--space-5);
  }

  h3 {
    font-size: var(--text-base);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
    margin: 0 0 var(--space-2) 0;
  }

  .prompt-content,
  .error-content,
  .result-content {
    background-color: var(--bg-tertiary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-md);
    padding: var(--space-4);
    color: var(--text-primary);
  }

  .result-content {
    max-height: 400px;
    overflow-y: auto;
  }

  .error-content {
    border-color: var(--error);
    background-color: rgba(255, 59, 48, 0.1);
  }

  .empty-content {
    background-color: var(--bg-tertiary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-md);
    padding: var(--space-5);
    text-align: center;
  }

  .empty-content .muted {
    color: var(--text-muted);
    margin: 0;
    font-style: italic;
    font-size: var(--text-sm);
  }

  .resume-section {
    padding: var(--space-5);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
  }

  .resume-btn {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-5);
    font-size: var(--text-sm);
    background: var(--accent-hex);
    color: white;
    border: none;
    border-radius: var(--radius-md);
    font-weight: var(--font-medium);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .resume-btn:hover {
    filter: brightness(1.1);
  }

  .resume-btn svg {
    width: 16px;
    height: 16px;
  }

  .resume-hint {
    font-size: var(--text-xs);
    color: var(--text-muted);
    margin: 0;
  }

  .auto-start-label {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-sm);
    color: var(--text-secondary);
    cursor: pointer;
  }

  .auto-start-label input[type="checkbox"] {
    width: 16px;
    height: 16px;
    accent-color: var(--accent-hex);
    cursor: pointer;
  }

  .auto-start-label span {
    user-select: none;
  }
</style>
