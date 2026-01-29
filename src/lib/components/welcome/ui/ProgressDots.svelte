<script lang="ts">
  interface Props {
    currentStep: number;
    totalSteps: number;
  }

  let { currentStep, totalSteps }: Props = $props();
</script>

<div class="progress-dots">
  {#each Array(totalSteps) as _, i}
    <div
      class="progress-dot"
      class:active={i + 1 === currentStep}
      class:completed={i + 1 < currentStep}
    ></div>
  {/each}
</div>

<style>
  .progress-dots {
    display: flex;
    justify-content: center;
    gap: var(--space-2);
    padding: var(--space-5) 0 var(--space-3);
  }

  .progress-dot {
    width: 8px;
    height: 8px;
    border-radius: var(--radius-full);
    background: var(--bg-elevated);
    transition: all var(--transition-normal);
  }

  .progress-dot.active {
    width: 24px;
    background: var(--accent-hex);
  }

  .progress-dot.completed {
    background: var(--accent-hex);
    opacity: 0.5;
  }
</style>
