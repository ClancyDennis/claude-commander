<script lang="ts">
  import type { Snippet } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog";
  import { ArrowLeft, ArrowRight } from "lucide-svelte";

  interface Props {
    onBack?: () => void;
    onNext?: () => void;
    onSkip?: () => void;
    backLabel?: string;
    nextLabel?: string;
    skipLabel?: string;
    showBack?: boolean;
    showNext?: boolean;
    showSkip?: boolean;
    nextDisabled?: boolean;
    nextLoading?: boolean;
    nextVariant?: "default" | "outline" | "ghost" | "destructive";
    layout?: "between" | "end" | "center";
    children?: Snippet;
  }

  let {
    onBack,
    onNext,
    onSkip,
    backLabel = "Back",
    nextLabel = "Continue",
    skipLabel = "Skip",
    showBack = true,
    showNext = true,
    showSkip = false,
    nextDisabled = false,
    nextLoading = false,
    nextVariant = "default",
    layout = "between",
    children,
  }: Props = $props();

  let justifyClass = $derived(
    layout === "between" ? "justify-between" :
    layout === "end" ? "justify-end" :
    "justify-center"
  );
</script>

<Dialog.Footer class={justifyClass}>
  {#if showBack && onBack}
    <Button variant="ghost" onclick={onBack} class="gap-2">
      <ArrowLeft class="w-4 h-4" />
      {backLabel}
    </Button>
  {:else if layout === "between"}
    <div></div>
  {/if}

  <div class="flex gap-2">
    {#if showSkip && onSkip}
      <Button variant="ghost" onclick={onSkip}>
        {skipLabel}
      </Button>
    {/if}

    {#if children}
      {@render children()}
    {/if}

    {#if showNext && onNext}
      <Button
        variant={nextVariant}
        onclick={onNext}
        disabled={nextDisabled || nextLoading}
        class="gap-2"
      >
        {#if nextLoading}
          <span class="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin"></span>
        {/if}
        {nextLabel}
        {#if !nextLoading}
          <ArrowRight class="w-4 h-4" />
        {/if}
      </Button>
    {/if}
  </div>
</Dialog.Footer>
