<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    step: number;
    currentStep: number;
    children: Snippet;
    animation?: "fade" | "slide";
  }

  let {
    step,
    currentStep,
    children,
    animation = "slide",
  }: Props = $props();

  let isActive = $derived(step === currentStep);
  let animationClass = $derived(
    animation === "fade" ? "wizard-animate-fade-in" : "wizard-animate-slide-in"
  );
</script>

{#if isActive}
  <div class={animationClass}>
    {@render children()}
  </div>
{/if}
