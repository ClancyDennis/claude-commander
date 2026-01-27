<script lang="ts">
  import HelpTip from "$lib/components/new-agent/HelpTip.svelte";
  import type { CommanderPersonality } from "$lib/stores/commanderPersonality";

  let { settings }: { settings: CommanderPersonality } = $props();
</script>

<section class="config-section">
  <h3>Voice</h3>

  <div class="setting-row two-col voice-row">
    <div class="inline-toggle">
      <label class="toggle">
        <input type="checkbox" bind:checked={settings.attentionEnabled} />
        <span class="toggle-slider"></span>
      </label>
      <div class="toggle-text">
        <span class="toggle-title">Listen for Follow-up</span>
        <HelpTip text="Keep listening after agent completes for your response." placement="top" />
      </div>
    </div>
    <div class="inline-field">
      <label for="listen-timeout">Timeout</label>
      <select id="listen-timeout" bind:value={settings.listenTimeout}>
        <option value={5}>5s</option>
        <option value={10}>10s</option>
        <option value={15}>15s</option>
        <option value={30}>30s</option>
      </select>
    </div>
  </div>

  <div class="setting-row two-col">
    <div class="inline-field">
      <label for="openai-voice">Voice</label>
      <select id="openai-voice" bind:value={settings.openaiVoice}>
        <option value="alloy">Alloy</option>
        <option value="ash">Ash</option>
        <option value="ballad">Ballad</option>
        <option value="coral">Coral</option>
        <option value="echo">Echo</option>
        <option value="sage">Sage</option>
        <option value="shimmer">Shimmer</option>
        <option value="verse">Verse</option>
      </select>
    </div>
    <div class="inline-field">
      <label for="voice-persona">Persona</label>
      <select id="voice-persona" bind:value={settings.voicePersona}>
        <option value="professional">Professional</option>
        <option value="casual">Casual</option>
        <option value="brief">Brief</option>
      </select>
    </div>
  </div>
</section>
