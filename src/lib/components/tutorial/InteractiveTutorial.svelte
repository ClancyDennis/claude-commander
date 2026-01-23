<script lang="ts">
  import { cn } from "$lib/utils";
  import { tutorialStore } from "$lib/stores/tutorial.svelte";
  import { Button } from "$lib/components/ui/button";
  import * as Card from "$lib/components/ui/card";

  interface SpotlightPosition {
    top: number;
    left: number;
    width: number;
    height: number;
  }

  let spotlightPosition = $state<SpotlightPosition | null>(null);
  let tooltipPosition = $state<{ top: number; left: number }>({ top: 0, left: 0 });

  // Update spotlight position when step changes
  $effect(() => {
    if (!tutorialStore.isActive) {
      spotlightPosition = null;
      return;
    }

    const step = tutorialStore.currentStepData;
    if (!step) return;

    const targetElement = document.querySelector(step.target);
    if (targetElement) {
      const rect = targetElement.getBoundingClientRect();
      const padding = 8;

      spotlightPosition = {
        top: rect.top - padding,
        left: rect.left - padding,
        width: rect.width + padding * 2,
        height: rect.height + padding * 2,
      };

      // Calculate tooltip position (below the spotlight, centered)
      const tooltipWidth = 320;
      let tooltipLeft = rect.left + rect.width / 2 - tooltipWidth / 2;
      let tooltipTop = rect.bottom + 16;

      // Ensure tooltip stays within viewport
      const viewportWidth = window.innerWidth;
      const viewportHeight = window.innerHeight;

      if (tooltipLeft < 16) tooltipLeft = 16;
      if (tooltipLeft + tooltipWidth > viewportWidth - 16) {
        tooltipLeft = viewportWidth - tooltipWidth - 16;
      }

      // If tooltip would go below viewport, show it above the spotlight
      if (tooltipTop + 200 > viewportHeight) {
        tooltipTop = rect.top - 200 - 16;
      }

      tooltipPosition = { top: tooltipTop, left: tooltipLeft };
    } else {
      // Fallback: center the tooltip if target not found
      spotlightPosition = null;
      tooltipPosition = {
        top: window.innerHeight / 2 - 100,
        left: window.innerWidth / 2 - 160,
      };
    }
  });

  // Keyboard navigation
  $effect(() => {
    if (!tutorialStore.isActive) return;

    function handleKeydown(event: KeyboardEvent) {
      if (event.key === 'ArrowRight' || event.key === 'Enter') {
        event.preventDefault();
        tutorialStore.next();
      } else if (event.key === 'Escape') {
        event.preventDefault();
        tutorialStore.skip();
      }
    }

    window.addEventListener('keydown', handleKeydown);

    return () => {
      window.removeEventListener('keydown', handleKeydown);
    };
  });

  // Generate clip-path polygon for backdrop cutout
  function getClipPath(pos: SpotlightPosition | null): string {
    if (!pos) {
      return 'none';
    }

    const { top, left, width, height } = pos;
    const right = left + width;
    const bottom = top + height;

    // Create a polygon that covers the entire screen with a hole for the spotlight
    // Using polygon to create a frame around the cutout
    return `polygon(
      0% 0%,
      0% 100%,
      ${left}px 100%,
      ${left}px ${top}px,
      ${right}px ${top}px,
      ${right}px ${bottom}px,
      ${left}px ${bottom}px,
      ${left}px 100%,
      100% 100%,
      100% 0%
    )`;
  }
</script>

{#if tutorialStore.isActive}
  <!-- Backdrop with cutout -->
  <div
    class="fixed inset-0 z-[9998] bg-black/80 backdrop-blur-sm transition-all duration-300"
    style="clip-path: {getClipPath(spotlightPosition)};"
    onclick={() => tutorialStore.skip()}
    onkeydown={(e) => e.key === 'Enter' && tutorialStore.skip()}
    role="button"
    tabindex="-1"
    aria-label="Click to skip tutorial"
  ></div>

  <!-- Spotlight glow effect -->
  {#if spotlightPosition}
    <div
      class="fixed z-[9999] pointer-events-none rounded-lg ring-2 ring-primary ring-offset-2 ring-offset-transparent transition-all duration-300"
      style="
        top: {spotlightPosition.top}px;
        left: {spotlightPosition.left}px;
        width: {spotlightPosition.width}px;
        height: {spotlightPosition.height}px;
        box-shadow: 0 0 0 4px hsl(var(--primary) / 0.3), 0 0 20px 8px hsl(var(--primary) / 0.2);
      "
    ></div>
  {/if}

  <!-- Tooltip Card -->
  <div
    class="fixed z-[10000] w-80 animate-scale-in"
    style="top: {tooltipPosition.top}px; left: {tooltipPosition.left}px;"
  >
    <Card.Root class="border-primary/50 shadow-lg shadow-primary/20">
      <Card.Header class="pb-2">
        <Card.Title class="text-lg text-primary">
          {tutorialStore.currentStepData?.title}
        </Card.Title>
      </Card.Header>
      <Card.Content class="pb-4">
        <p class="text-sm text-muted-foreground">
          {tutorialStore.currentStepData?.message}
        </p>
      </Card.Content>
      <Card.Footer class="flex items-center justify-between pt-0">
        <!-- Progress dots -->
        <div class="flex gap-1.5">
          {#each tutorialStore.steps as _, index}
            <div
              class={cn(
                "h-2 w-2 rounded-full transition-all duration-200",
                index === tutorialStore.currentStep
                  ? "bg-primary scale-125"
                  : index < tutorialStore.currentStep
                    ? "bg-primary/60"
                    : "bg-muted"
              )}
            ></div>
          {/each}
        </div>

        <!-- Navigation buttons -->
        <div class="flex gap-2">
          <Button
            variant="ghost"
            size="sm"
            onclick={() => tutorialStore.skip()}
          >
            Skip tour
          </Button>
          <Button
            size="sm"
            onclick={() => tutorialStore.next()}
          >
            {tutorialStore.currentStep === tutorialStore.steps.length - 1 ? 'Finish' : 'Next'}
          </Button>
        </div>
      </Card.Footer>
    </Card.Root>
  </div>
{/if}
