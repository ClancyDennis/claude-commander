<script lang="ts">
  import type { Snippet } from "svelte";
  import * as Dialog from "$lib/components/ui/dialog";

  interface Props {
    open: boolean;
    totalSteps: number;
    currentStep: number;
    title?: string;
    onClose: () => void;
    children: Snippet;
    wide?: boolean;
  }

  let {
    open = $bindable(),
    totalSteps,
    currentStep,
    title = "",
    onClose,
    children,
    wide = false,
  }: Props = $props();

  // Animation key for step transitions
  let animationKey = $state(0);

  // Track step changes for animations
  $effect(() => {
    // Increment animation key when step changes
    animationKey = currentStep;
  });

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<Dialog.Root bind:open>
  <Dialog.Content class="max-w-2xl p-0 overflow-hidden {wide ? 'max-w-4xl' : ''}">
    <!-- Progress Indicator -->
    <div class="flex items-center justify-center gap-2 pt-6 pb-2">
      {#each Array(totalSteps) as _, i}
        <div
          class="w-2 h-2 rounded-full transition-all duration-300 {i + 1 === currentStep
            ? 'bg-primary'
            : i + 1 < currentStep
              ? 'bg-primary/60'
              : 'bg-muted'}"
        ></div>
      {/each}
    </div>

    <!-- Step Content Container -->
    <div class="p-6 pt-2" key={animationKey}>
      {@render children()}
    </div>
  </Dialog.Content>
</Dialog.Root>

<style>
  :global(.wizard-animate-fade-in) {
    animation: wizardFadeIn 0.3s ease-out;
  }

  :global(.wizard-animate-slide-in) {
    animation: wizardSlideIn 0.3s ease-out;
  }

  :global(.wizard-animate-slide-out) {
    animation: wizardSlideOut 0.2s ease-in;
  }

  @keyframes wizardFadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  @keyframes wizardSlideIn {
    from {
      opacity: 0;
      transform: translateX(20px);
    }
    to {
      opacity: 1;
      transform: translateX(0);
    }
  }

  @keyframes wizardSlideOut {
    from {
      opacity: 1;
      transform: translateX(0);
    }
    to {
      opacity: 0;
      transform: translateX(-20px);
    }
  }
</style>
