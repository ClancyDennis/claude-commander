<script lang="ts">
  import type { SystemPromptInfo } from "$lib/types";
  import MarkdownRenderer from "$lib/components/MarkdownRenderer.svelte";

  let showRaw = $state(false);

  let {
    systemPrompt,
    systemPromptLoading,
    systemPromptError,
    systemPromptCopied,
    onRefresh,
    onCopy,
    onClose,
    onKeydown,
  }: {
    systemPrompt: SystemPromptInfo | null;
    systemPromptLoading: boolean;
    systemPromptError: string | null;
    systemPromptCopied: boolean;
    onRefresh: () => void;
    onCopy: () => void;
    onClose: () => void;
    onKeydown: (e: KeyboardEvent) => void;
  } = $props();
</script>

<svelte:window onkeydown={onKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="prompt-overlay" onclick={onClose}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="prompt-modal" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true" tabindex="-1">
    <header class="prompt-header">
      <div class="prompt-title">
        <h2>System Prompt</h2>
        {#if systemPrompt}
          <span class="prompt-source">
            {systemPrompt.source === "personalized" ? "Personalized prompt" : "Base prompt"}
          </span>
        {/if}
      </div>
      <div class="prompt-modal-actions">
        <button
          class="prompt-btn ghost"
          onclick={() => showRaw = !showRaw}
          disabled={systemPromptLoading}
          title={showRaw ? "Show rendered" : "Show raw"}
        >
          {showRaw ? "Rendered" : "Raw"}
        </button>
        <button class="prompt-btn ghost" onclick={onRefresh} disabled={systemPromptLoading}>
          Refresh
        </button>
        <button class="prompt-btn" onclick={onCopy} disabled={!systemPrompt || systemPromptLoading}>
          {systemPromptCopied ? "Copied" : "Copy"}
        </button>
        <button class="prompt-btn ghost" onclick={onClose}>Close</button>
      </div>
    </header>
    <div class="prompt-body">
      {#if systemPromptLoading}
        <div class="prompt-loading">Loading system prompt...</div>
      {:else if systemPromptError}
        <div class="prompt-error">Failed to load prompt: {systemPromptError}</div>
      {:else if systemPrompt}
        {#if showRaw}
          <pre class="prompt-text">{systemPrompt.prompt}</pre>
        {:else}
          <div class="prompt-markdown">
            <MarkdownRenderer content={systemPrompt.prompt} />
          </div>
        {/if}
      {:else}
        <div class="prompt-empty">No prompt available.</div>
      {/if}
    </div>
  </div>
</div>
