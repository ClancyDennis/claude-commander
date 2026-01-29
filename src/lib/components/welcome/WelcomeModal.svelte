<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import * as Dialog from "$lib/components/ui/dialog";
  import ProgressDots from "./ui/ProgressDots.svelte";
  import Step1ClaudeStatus from "./steps/Step1ClaudeStatus.svelte";
  import Step2Authentication from "./steps/Step2Authentication.svelte";
  import Step3Ready from "./steps/Step3Ready.svelte";
  import {
    TOTAL_STEPS,
    STORAGE_KEY,
    createInitialClaudeStatus,
    createInitialAuthConfig,
    type ClaudeCodeStatus,
    type AuthConfig
  } from "./types";

  interface Props {
    show: boolean;
    onClose: () => void;
  }

  let { show = $bindable(), onClose }: Props = $props();

  const dispatch = createEventDispatcher<{ startTutorial: void }>();

  // Wizard state
  let currentStep = $state(1);
  let claudeStatus = $state<ClaudeCodeStatus>(createInitialClaudeStatus());
  let authConfig = $state<AuthConfig>(createInitialAuthConfig());

  function goToStep(step: number) {
    currentStep = step;
  }

  function handleStartTutorial() {
    completeOnboarding();
    onClose();
    // Delay tour start to avoid click propagation to backdrop
    setTimeout(() => {
      dispatch("startTutorial");
    }, 100);
  }

  function handleSkip() {
    completeOnboarding();
    onClose();
  }

  function completeOnboarding() {
    try {
      localStorage.setItem(STORAGE_KEY, "true");
    } catch (e) {
      console.error("Failed to save onboarding status:", e);
    }
  }
</script>

<Dialog.Root bind:open={show}>
  <Dialog.Content class="welcome-modal">
    <ProgressDots {currentStep} totalSteps={TOTAL_STEPS} />

    <div class="step-container">
      {#if currentStep === 1}
        <Step1ClaudeStatus
          bind:claudeStatus
          bind:authConfig
          onNext={() => goToStep(2)}
          onSkip={handleSkip}
        />
      {:else if currentStep === 2}
        <Step2Authentication
          bind:authConfig
          onNext={() => goToStep(3)}
          onBack={() => goToStep(1)}
          onSkip={handleSkip}
        />
      {:else if currentStep === 3}
        <Step3Ready onStartTutorial={handleStartTutorial} onSkip={handleSkip} />
      {/if}
    </div>
  </Dialog.Content>
</Dialog.Root>

<style>
  :global(.welcome-modal) {
    max-width: 560px !important;
    padding: 0 !important;
    overflow: hidden;
    border-radius: var(--radius-xl) !important;
    background: var(--bg-secondary) !important;
    border: 1px solid var(--border-hex) !important;
    box-shadow: var(--shadow-xl) !important;
  }

  .step-container {
    padding: 0 var(--space-8) var(--space-8);
  }
</style>
