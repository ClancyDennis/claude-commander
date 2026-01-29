<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { ArrowRight, Rocket } from "lucide-svelte";

  interface Props {
    showBack?: boolean;
    showSkip?: boolean;
    showContinue?: boolean;
    continueDisabled?: boolean;
    continueLabel?: string;
    variant?: "default" | "final";
    onBack?: () => void;
    onSkip?: () => void;
    onContinue?: () => void;
  }

  let {
    showBack = false,
    showSkip = false,
    showContinue = true,
    continueDisabled = false,
    continueLabel = "Continue",
    variant = "default",
    onBack,
    onSkip,
    onContinue
  }: Props = $props();
</script>

{#if variant === "final"}
  <div class="step-footer final">
    <Button onclick={onContinue} size="lg" class="tour-btn">
      <Rocket class="btn-icon" />
      {continueLabel}
    </Button>
    {#if showSkip && onSkip}
      <Button variant="ghost" onclick={onSkip}>Skip and get started</Button>
    {/if}
  </div>
{:else}
  <div class="step-footer">
    <div>
      {#if showBack && onBack}
        <Button variant="ghost" onclick={onBack}>Back</Button>
      {/if}
    </div>
    <div class="footer-right">
      {#if showSkip && onSkip}
        <Button variant="ghost" onclick={onSkip}>Skip</Button>
      {/if}
      {#if showContinue && onContinue}
        <Button onclick={onContinue} disabled={continueDisabled} size="lg">
          {continueLabel}
          <ArrowRight class="btn-icon-right" />
        </Button>
      {/if}
    </div>
  </div>
{/if}

<style>
  .step-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-top: var(--space-4);
    border-top: 1px solid var(--border-hex);
  }

  .step-footer.final {
    flex-direction: column;
    gap: var(--space-3);
    border-top: none;
    padding-top: var(--space-2);
  }

  .footer-right {
    display: flex;
    gap: var(--space-3);
    align-items: center;
  }

  :global(.tour-btn) {
    width: 100%;
  }

  :global(.btn-icon) {
    width: 16px;
    height: 16px;
    margin-right: var(--space-2);
  }

  :global(.btn-icon-right) {
    width: 16px;
    height: 16px;
    margin-left: var(--space-2);
  }
</style>
