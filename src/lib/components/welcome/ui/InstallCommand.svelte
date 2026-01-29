<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Check, Copy, RefreshCw, ExternalLink } from "lucide-svelte";

  interface Props {
    command: string;
    onRecheck: () => void;
    onOpenDocs?: () => void;
  }

  let { command, onRecheck, onOpenDocs }: Props = $props();

  let commandCopied = $state(false);

  async function copyInstallCommand() {
    try {
      await navigator.clipboard.writeText(command);
      commandCopied = true;
      setTimeout(() => (commandCopied = false), 2000);
    } catch (e) {
      console.error("Failed to copy:", e);
    }
  }

  function handleOpenDocs() {
    if (onOpenDocs) {
      onOpenDocs();
    } else {
      window.open("https://docs.anthropic.com/en/docs/claude-code", "_blank");
    }
  }
</script>

<div class="install-section">
  <h4>Install with npm:</h4>
  <div class="install-command-wrapper">
    <code class="install-command">{command}</code>
    <button class="copy-btn" onclick={copyInstallCommand} title="Copy to clipboard">
      {#if commandCopied}
        <Check class="copy-icon success" />
      {:else}
        <Copy class="copy-icon" />
      {/if}
    </button>
  </div>

  <div class="install-actions">
    <Button variant="outline" onclick={onRecheck} class="recheck-btn">
      <RefreshCw class="btn-icon" />
      Re-check
    </Button>
    <Button variant="ghost" onclick={handleOpenDocs}>
      <ExternalLink class="btn-icon" />
      Documentation
    </Button>
  </div>
</div>

<style>
  .install-section {
    padding: var(--space-5);
    background: var(--bg-tertiary);
    border-radius: var(--radius-lg);
    text-align: center;
  }

  .install-section h4 {
    font-size: var(--text-sm);
    color: var(--text-muted);
    margin: 0 0 var(--space-3);
    font-weight: var(--font-medium);
  }

  .install-command-wrapper {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-4);
  }

  .install-command {
    flex: 1;
    display: block;
    padding: var(--space-4);
    background: var(--bg-primary);
    border-radius: var(--radius-md);
    font-family: "SF Mono", ui-monospace, monospace;
    font-size: var(--text-sm);
    color: var(--text-accent);
    text-align: left;
    user-select: all;
    border: 1px solid var(--border-hex);
  }

  .copy-btn {
    padding: var(--space-3);
    background: var(--bg-elevated);
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    transition: all var(--transition-fast);
    border: none;
    cursor: pointer;
  }

  .copy-btn:hover {
    background: var(--border-light);
    color: var(--text-primary);
  }

  :global(.copy-icon) {
    width: 18px;
    height: 18px;
  }

  :global(.copy-icon.success) {
    color: var(--success-hex);
  }

  .install-actions {
    display: flex;
    justify-content: center;
    gap: var(--space-3);
  }

  :global(.btn-icon) {
    width: 16px;
    height: 16px;
    margin-right: var(--space-2);
  }
</style>
