<script lang="ts">
  import { Check } from "lucide-svelte";
  import WizardFooter from "../ui/WizardFooter.svelte";
  import type { KeyboardShortcut } from "../types";

  interface Props {
    onStartTutorial: () => void;
    onSkip: () => void;
  }

  let { onStartTutorial, onSkip }: Props = $props();

  const shortcuts: KeyboardShortcut[] = [
    { keys: ["Ctrl", "K"], label: "Command palette" },
    { keys: ["Ctrl", "Enter"], label: "Send message" },
    { keys: ["Ctrl", "Shift", "N"], label: "New agent" }
  ];
</script>

<div class="step-content ready-step animate-scale-in">
  <div class="success-icon">
    <Check />
  </div>

  <h2>You're All Set!</h2>
  <p class="subtitle">Claude Commander is ready to help you build amazing things.</p>

  <!-- Quick Tips / Keyboard Shortcuts -->
  <div class="quick-tips">
    {#each shortcuts as shortcut}
      <div class="tip">
        <div class="tip-keys">
          {#each shortcut.keys as key, i}
            {#if i > 0}<span class="key-separator">+</span>{/if}
            <kbd>{key}</kbd>
          {/each}
        </div>
        <span class="tip-label">{shortcut.label}</span>
      </div>
    {/each}
  </div>

  <WizardFooter
    variant="final"
    showSkip={true}
    continueLabel="Take Interactive Tour"
    onContinue={onStartTutorial}
    {onSkip}
  />
</div>

<style>
  .step-content {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }

  .ready-step {
    text-align: center;
    align-items: center;
  }

  .ready-step h2 {
    font-size: var(--text-xl);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
    margin: 0;
    letter-spacing: -0.01em;
  }

  .subtitle {
    font-size: var(--text-base);
    color: var(--text-secondary);
    margin: 0;
    line-height: var(--leading-relaxed);
  }

  .success-icon {
    width: 64px;
    height: 64px;
    border-radius: var(--radius-full);
    background: var(--success-glow);
    color: var(--success-hex);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .success-icon :global(svg) {
    width: 32px;
    height: 32px;
  }

  .quick-tips {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding: var(--space-5);
    background: var(--bg-tertiary);
    border-radius: var(--radius-lg);
    width: 100%;
  }

  .tip {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
  }

  .tip-keys {
    display: flex;
    align-items: center;
    gap: var(--space-1);
  }

  kbd {
    padding: var(--space-1) var(--space-2);
    background: var(--bg-primary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-sm);
    font-family: "SF Mono", ui-monospace, monospace;
    font-size: var(--text-xs);
    color: var(--text-primary);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
  }

  .key-separator {
    font-size: var(--text-xs);
    color: var(--text-muted);
  }

  .tip-label {
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }

  .animate-scale-in {
    animation: scaleIn 0.3s var(--spring-bounce);
  }

  @keyframes scaleIn {
    from {
      opacity: 0;
      transform: scale(0.95);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }
</style>
