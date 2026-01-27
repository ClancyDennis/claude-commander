<script lang="ts">
  import {
    commanderPersonality,
    updatePersonality,
    resetPersonality,
    hasCustomSettings,
    focusAreaOptions,
    commonLanguages,
    commonFrameworks,
    type CommanderPersonality,
  } from "$lib/stores/commanderPersonality";
  import { useSystemPrompt } from "$lib/hooks/useSystemPrompt.svelte";
  import {
    PersonalityToneSection,
    TechnologyPreferencesSection,
    CodeQualitySection,
    VoiceSection,
    CustomInstructionsSection,
    SystemPromptSection,
    SystemPromptModal,
  } from "$lib/components/commander-settings";

  let { onClose }: { onClose?: () => void } = $props();

  // Local reactive copy for form binding (not synced until Save)
  let settings = $state<CommanderPersonality>({ ...$commanderPersonality });
  let isSaving = $state(false);
  let savedSettings = $state<string>(JSON.stringify($commanderPersonality));
  const systemPrompt = useSystemPrompt();

  // Check if there are unsaved changes
  let hasChanges = $derived(JSON.stringify(settings) !== savedSettings);

  // Sync from store when it changes externally (e.g., reset)
  $effect(() => {
    const storeValue = $commanderPersonality;
    savedSettings = JSON.stringify(storeValue);
    settings = { ...storeValue };
  });

  // Save and trigger generation
  async function handleSave() {
    isSaving = true;
    try {
      await updatePersonality(settings);
      savedSettings = JSON.stringify(settings);
    } finally {
      isSaving = false;
    }
  }

  function handleReset() {
    if (confirm("Reset all Commander settings to defaults?")) {
      resetPersonality();
    }
  }

  function handleDiscard() {
    settings = { ...$commanderPersonality };
  }
</script>

<div class="commander-settings">
  <header class="settings-header">
    <h2>Commander Settings</h2>
    <div class="header-actions">
      {#if hasChanges}
        <button class="discard-btn" onclick={handleDiscard} disabled={isSaving}>Discard</button>
        <button class="save-btn" onclick={handleSave} disabled={isSaving}>
          {#if isSaving}Generating...{:else}Save{/if}
        </button>
      {/if}
      {#if hasCustomSettings()}
        <button class="reset-btn" onclick={handleReset} title="Reset to Defaults" disabled={isSaving}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M1 4v6h6M23 20v-6h-6"/>
            <path d="M20.49 9A9 9 0 005.64 5.64L1 10m22 4l-4.64 4.36A9 9 0 014.51 15"/>
          </svg>
        </button>
      {/if}
      {#if onClose}
        <button class="close-btn" onclick={onClose} title="Close">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      {/if}
    </div>
  </header>

  <p class="settings-description">
    Configure how the System Commander manages agents and communicates with you.
    These preferences are woven into the commander's personality.
  </p>

  <PersonalityToneSection settings={settings} />

  <TechnologyPreferencesSection
    settings={settings}
    commonLanguages={commonLanguages}
    commonFrameworks={commonFrameworks}
  />

  <CodeQualitySection settings={settings} focusAreaOptions={focusAreaOptions} />

  <VoiceSection settings={settings} />

  <CustomInstructionsSection settings={settings} />

  <SystemPromptSection
    systemPrompt={systemPrompt.state.systemPrompt}
    openSystemPrompt={systemPrompt.openSystemPrompt}
    resetSystemPrompt={systemPrompt.resetSystemPrompt}
    resetting={systemPrompt.state.resetting}
  />

  {#if systemPrompt.state.showSystemPrompt}
    <SystemPromptModal
      systemPrompt={systemPrompt.state.systemPrompt}
      systemPromptLoading={systemPrompt.state.systemPromptLoading}
      systemPromptError={systemPrompt.state.systemPromptError}
      systemPromptCopied={systemPrompt.state.systemPromptCopied}
      onRefresh={systemPrompt.loadSystemPrompt}
      onCopy={systemPrompt.copySystemPrompt}
      onClose={systemPrompt.closeSystemPrompt}
      onKeydown={systemPrompt.handleSystemPromptKeydown}
    />
  {/if}
</div>
