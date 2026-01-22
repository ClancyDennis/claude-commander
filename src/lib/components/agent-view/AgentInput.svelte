<script lang="ts">
  import { onMount } from "svelte";

  let { status, onSend }: { status: string; onSend: (text: string) => void } = $props();

  let promptInput = $state("");
  let textarea: HTMLTextAreaElement | null = $state(null);

  function adjustHeight() {
    if (textarea) {
      textarea.style.height = "auto";
      textarea.style.height = Math.min(textarea.scrollHeight, 200) + "px";
    }
  }

  function handleSend() {
    if (status !== "running" || !promptInput.trim()) return;
    onSend(promptInput);
    promptInput = "";
    // Reset height
    if (textarea) {
      textarea.style.height = "auto";
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  }
  
  $effect(() => {
    // Re-adjust if content changes programmatically (e.g. paste or clear)
    if (promptInput !== undefined) {
      adjustHeight();
    }
  });
</script>

<div class="input-area">
  <div class="input-wrapper">
    <textarea
      bind:this={textarea}
      bind:value={promptInput}
      placeholder="Type your prompt here..."
      onkeydown={handleKeydown}
      oninput={adjustHeight}
      disabled={status !== "running"}
      rows="1"
    ></textarea>
    <button
      class="primary send-btn"
      onclick={handleSend}
      disabled={status !== "running" || !promptInput.trim()}
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="22" y1="2" x2="11" y2="13"/>
        <polygon points="22,2 15,22 11,13 2,9"/>
      </svg>
      Send
    </button>
  </div>
</div>

<style>
  .input-area {
    padding: var(--space-lg);
    border-top: 1px solid var(--border);
    background-color: var(--bg-secondary);
  }

  .input-wrapper {
    display: flex;
    gap: var(--space-md);
    align-items: flex-end;
  }

  textarea {
    flex: 1;
    resize: none;
    min-height: 52px; /* Start smaller (single line + padding) */
    max-height: 200px;
    font-family: inherit;
    border-radius: 16px;
    padding: var(--space-md) var(--space-lg);
    overflow-y: hidden; /* Hide scrollbar when small */
    line-height: 1.5;
  }
  
  textarea:focus {
    overflow-y: auto; /* Show scrollbar if max-height reached */
  }

  .send-btn {
    padding: 16px 28px;
    flex-shrink: 0;
    height: 52px; /* Match initial input height */
  }

  .send-btn svg {
    width: 20px;
    height: 20px;
  }

  .send-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
