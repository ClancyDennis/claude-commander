<script lang="ts">
  import type { CommanderPersonality } from "$lib/stores/commanderPersonality";

  let { settings }: { settings: CommanderPersonality } = $props();
</script>

<section class="config-section">
  <h3>Custom Instructions</h3>
  <textarea
    placeholder="e.g., Always suggest tests. Prefer Svelte 5 runes. Be careful with migrations."
    bind:value={settings.customInstructions}
    rows="3"
  ></textarea>
</section>
