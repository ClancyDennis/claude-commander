<script lang="ts">
  import HelpTip from "$lib/components/new-agent/HelpTip.svelte";
  import type { CommanderPersonality } from "$lib/stores/commanderPersonality";

  let {
    settings,
    focusAreaOptions,
  }: {
    settings: CommanderPersonality;
    focusAreaOptions: Array<{ value: string; label: string }>;
  } = $props();

  function toggleFocusArea(area: string) {
    if (settings.focusAreas.includes(area)) {
      settings.focusAreas = settings.focusAreas.filter((a) => a !== area);
    } else {
      settings.focusAreas = [...settings.focusAreas, area];
    }
  }
</script>

<section class="config-section">
  <h3>Code Quality & Agent</h3>
  <div class="checkbox-grid compact">
    {#each focusAreaOptions as option}
      <label class="checkbox-item">
        <input type="checkbox" checked={settings.focusAreas.includes(option.value)} onchange={() => toggleFocusArea(option.value)} />
        <span class="checkbox-label">{option.label}</span>
      </label>
    {/each}
  </div>
  <div class="setting-row compact-slider" style="margin-top: var(--space-3);">
    <div class="setting-inline">
      <label for="autonomy">Autonomy</label>
      <HelpTip text="How much guidance vs independence agents get." placement="top" />
    </div>
    <div class="slider-compact">
      <span class="slider-label-sm">Guided</span>
      <input type="range" id="autonomy" min="1" max="10" bind:value={settings.autonomyLevel} />
      <span class="slider-label-sm">Autonomous</span>
      <span class="slider-value">{settings.autonomyLevel}</span>
    </div>
  </div>
</section>
