<script lang="ts">
  import HelpTip from "$lib/components/new-agent/HelpTip.svelte";
  import type { CommanderPersonality } from "$lib/stores/commanderPersonality";

  let { settings }: { settings: CommanderPersonality } = $props();
</script>

<section class="config-section">
  <h3>Personality & Tone</h3>

  <div class="setting-row compact-slider">
    <div class="setting-inline">
      <label for="strictness">Strictness</label>
      <HelpTip text="How demanding the commander is about code quality and task completion." placement="top" />
    </div>
    <div class="slider-compact">
      <span class="slider-label-sm">Lenient</span>
      <input type="range" id="strictness" min="1" max="10" bind:value={settings.strictness} />
      <span class="slider-label-sm">Demanding</span>
      <span class="slider-value">{settings.strictness}</span>
    </div>
  </div>

  <div class="setting-row two-col">
    <div class="inline-field">
      <label for="communication-style">Style</label>
      <select id="communication-style" bind:value={settings.communicationStyle}>
        <option value="concise">Concise</option>
        <option value="balanced">Balanced</option>
        <option value="verbose">Verbose</option>
      </select>
    </div>
    <div class="inline-field">
      <label for="tone">Tone</label>
      <select id="tone" bind:value={settings.tone}>
        <option value="professional">Professional</option>
        <option value="friendly">Friendly</option>
        <option value="direct">Direct</option>
      </select>
    </div>
  </div>
</section>
