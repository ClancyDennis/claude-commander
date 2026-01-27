<script lang="ts">
  import HelpTip from "$lib/components/new-agent/HelpTip.svelte";
  import type { CommanderPersonality } from "$lib/stores/commanderPersonality";

  let {
    settings,
    commonLanguages,
    commonFrameworks,
  }: {
    settings: CommanderPersonality;
    commonLanguages: string[];
    commonFrameworks: string[];
  } = $props();

  let newLanguage = $state("");
  let newFramework = $state("");

  function addLanguage(lang: string) {
    if (lang && !settings.preferredLanguages.includes(lang)) {
      settings.preferredLanguages = [...settings.preferredLanguages, lang];
    }
    newLanguage = "";
  }

  function removeLanguage(lang: string) {
    settings.preferredLanguages = settings.preferredLanguages.filter((l) => l !== lang);
  }

  function addFramework(fw: string) {
    if (fw && !settings.preferredFrameworks.includes(fw)) {
      settings.preferredFrameworks = [...settings.preferredFrameworks, fw];
    }
    newFramework = "";
  }

  function removeFramework(fw: string) {
    settings.preferredFrameworks = settings.preferredFrameworks.filter((f) => f !== fw);
  }
</script>

<section class="config-section">
  <h3>Technology Preferences</h3>

  <div class="setting-row two-col tag-grid">
    <div class="tag-field">
      <div class="setting-inline">
        <label>Languages</label>
        <HelpTip text="Languages you prefer. The commander will favor these when suggesting code." placement="top" />
      </div>
      <div class="tag-list compact">
        {#each settings.preferredLanguages as lang}
          <span class="tag">{lang}<button class="tag-remove" onclick={() => removeLanguage(lang)}>×</button></span>
        {/each}
      </div>
      <div class="tag-input-row">
        <input
          type="text"
          placeholder="Add..."
          bind:value={newLanguage}
          onkeydown={(e) => e.key === "Enter" && addLanguage(newLanguage)}
          list="languages-list"
        />
        <datalist id="languages-list">
          {#each commonLanguages.filter((l) => !settings.preferredLanguages.includes(l)) as lang}<option value={lang} />{/each}
        </datalist>
        <button class="add-btn" onclick={() => addLanguage(newLanguage)}>+</button>
      </div>
    </div>
    <div class="tag-field">
      <div class="setting-inline"><label>Frameworks</label></div>
      <div class="tag-list compact">
        {#each settings.preferredFrameworks as fw}
          <span class="tag">{fw}<button class="tag-remove" onclick={() => removeFramework(fw)}>×</button></span>
        {/each}
      </div>
      <div class="tag-input-row">
        <input
          type="text"
          placeholder="Add..."
          bind:value={newFramework}
          onkeydown={(e) => e.key === "Enter" && addFramework(newFramework)}
          list="frameworks-list"
        />
        <datalist id="frameworks-list">
          {#each commonFrameworks.filter((f) => !settings.preferredFrameworks.includes(f)) as fw}<option value={fw} />{/each}
        </datalist>
        <button class="add-btn" onclick={() => addFramework(newFramework)}>+</button>
      </div>
    </div>
  </div>

  <div class="setting-row two-col">
    <div class="inline-field stacked">
      <label for="patterns-favor">Patterns to Favor</label>
      <input type="text" id="patterns-favor" placeholder="e.g., functional, composition" bind:value={settings.patternsToFavor} />
    </div>
    <div class="inline-field stacked">
      <label for="patterns-avoid">Patterns to Avoid</label>
      <input type="text" id="patterns-avoid" placeholder="e.g., singletons, deep nesting" bind:value={settings.patternsToAvoid} />
    </div>
  </div>
</section>
