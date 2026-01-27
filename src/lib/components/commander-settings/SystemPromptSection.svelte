<script lang="ts">
  import type { SystemPromptInfo } from "$lib/types";

  let {
    systemPrompt,
    openSystemPrompt,
    resetSystemPrompt,
    resetting = false,
  }: {
    systemPrompt: SystemPromptInfo | null;
    openSystemPrompt: () => void;
    resetSystemPrompt: () => void;
    resetting?: boolean;
  } = $props();

  const isPersonalized = $derived(systemPrompt?.source === "personalized");
</script>

<section class="config-section">
  <h3>System Prompt</h3>
  <p class="section-description">
    View the exact prompt the System Commander sends to the meta agent, including your saved settings.
  </p>
  <div class="prompt-actions">
    <button class="view-btn" onclick={openSystemPrompt}>View Prompt</button>
    {#if isPersonalized}
      <button
        class="reset-btn"
        onclick={resetSystemPrompt}
        disabled={resetting}
        title="Reset to base prompt (removes personalization)"
      >
        {resetting ? "Resetting..." : "Reset to Base"}
      </button>
    {/if}
    {#if systemPrompt}
      <span class="prompt-meta">
        Source: {isPersonalized ? "Personalized" : "Base"}
      </span>
    {/if}
  </div>
</section>
