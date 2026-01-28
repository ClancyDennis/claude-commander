<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { MetaAgentQuestionEvent } from "$lib/types";

  export let question: MetaAgentQuestionEvent | null = null;
  export let onAnswered: () => void = () => {};

  let customAnswer = "";
  let isSubmitting = false;
  let error: string | null = null;

  async function submitAnswer(answer: string) {
    if (!question || isSubmitting) return;

    isSubmitting = true;
    error = null;

    try {
      await invoke("answer_meta_agent_question", {
        questionId: question.question_id,
        answer,
      });
      customAnswer = "";
      onAnswered();
    } catch (e) {
      error = String(e);
      console.error("[MetaAgentQuestion] Failed to submit answer:", e);
    } finally {
      isSubmitting = false;
    }
  }

  function handleOptionClick(option: string) {
    submitAnswer(option);
  }

  function handleCustomSubmit() {
    if (customAnswer.trim()) {
      submitAnswer(customAnswer.trim());
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleCustomSubmit();
    }
  }
</script>

{#if question}
  <div class="meta-agent-question">
    <div class="question-header">
      <span class="question-icon">?</span>
      <span class="question-label">System Commander needs your input</span>
    </div>

    <div class="question-text">
      {question.question}
    </div>

    {#if question.options && question.options.length > 0}
      <div class="options-container">
        {#each question.options as option}
          <button
            class="option-button"
            disabled={isSubmitting}
            on:click={() => handleOptionClick(option)}
          >
            {option}
          </button>
        {/each}
      </div>
    {/if}

    <div class="custom-answer-container">
      <input
        type="text"
        class="custom-answer-input"
        placeholder="Type a custom answer..."
        bind:value={customAnswer}
        disabled={isSubmitting}
        on:keydown={handleKeydown}
      />
      <button
        class="submit-button"
        disabled={isSubmitting || !customAnswer.trim()}
        on:click={handleCustomSubmit}
      >
        {isSubmitting ? "Sending..." : "Send"}
      </button>
    </div>

    {#if error}
      <div class="error-message">
        {error}
      </div>
    {/if}
  </div>
{/if}

<style>
  .meta-agent-question {
    background: var(--bg-secondary, #1e1e1e);
    border: 1px solid var(--border-color, #3a3a3a);
    border-radius: 8px;
    padding: 16px;
    margin: 12px 0;
    animation: slideIn 0.2s ease-out;
  }

  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateY(-10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .question-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 12px;
  }

  .question-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    background: var(--accent-color, #4a9eff);
    color: white;
    border-radius: 50%;
    font-weight: bold;
    font-size: 14px;
  }

  .question-label {
    font-size: 12px;
    color: var(--text-secondary, #888);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .question-text {
    font-size: 14px;
    color: var(--text-primary, #fff);
    line-height: 1.5;
    margin-bottom: 16px;
  }

  .options-container {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-bottom: 12px;
  }

  .option-button {
    padding: 8px 16px;
    background: var(--bg-tertiary, #2a2a2a);
    border: 1px solid var(--border-color, #3a3a3a);
    border-radius: 6px;
    color: var(--text-primary, #fff);
    cursor: pointer;
    font-size: 13px;
    transition: all 0.15s ease;
  }

  .option-button:hover:not(:disabled) {
    background: var(--accent-color, #4a9eff);
    border-color: var(--accent-color, #4a9eff);
  }

  .option-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .custom-answer-container {
    display: flex;
    gap: 8px;
  }

  .custom-answer-input {
    flex: 1;
    padding: 10px 12px;
    background: var(--bg-tertiary, #2a2a2a);
    border: 1px solid var(--border-color, #3a3a3a);
    border-radius: 6px;
    color: var(--text-primary, #fff);
    font-size: 13px;
  }

  .custom-answer-input:focus {
    outline: none;
    border-color: var(--accent-color, #4a9eff);
  }

  .custom-answer-input::placeholder {
    color: var(--text-secondary, #888);
  }

  .submit-button {
    padding: 10px 20px;
    background: var(--accent-color, #4a9eff);
    border: none;
    border-radius: 6px;
    color: white;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .submit-button:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .submit-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .error-message {
    margin-top: 12px;
    padding: 8px 12px;
    background: rgba(255, 82, 82, 0.1);
    border: 1px solid rgba(255, 82, 82, 0.3);
    border-radius: 4px;
    color: #ff5252;
    font-size: 12px;
  }
</style>
