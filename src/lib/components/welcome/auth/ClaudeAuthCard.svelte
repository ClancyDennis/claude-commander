<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Check } from "lucide-svelte";
  import AuthCard from "./AuthCard.svelte";
  import type { ClaudeAuthConfig } from "../types";

  interface Props {
    config: ClaudeAuthConfig;
    onToggle: () => void;
    onMarkReady: () => void;
  }

  let { config = $bindable(), onToggle, onMarkReady }: Props = $props();
</script>

<AuthCard
  title="Claude Code Authentication"
  description="Use existing Claude Code login (run <code>claude</code> to authenticate)"
  iconType="claude"
  expanded={config.expanded}
  status={config.status}
  {onToggle}
>
  <p class="auth-instructions">
    If you've already authenticated Claude Code in your terminal, click "I'm Ready" below.
    Otherwise, open a terminal and run <code>claude</code> to start the authentication process.
  </p>
  <Button onclick={onMarkReady} disabled={config.status === "ready"}>
    {#if config.status === "ready"}
      <Check class="btn-icon" />
      Authenticated
    {:else}
      I'm Ready
    {/if}
  </Button>
</AuthCard>

<style>
  .auth-instructions {
    font-size: var(--text-sm);
    color: var(--text-secondary);
    margin: 0 0 var(--space-4);
    line-height: var(--leading-relaxed);
  }

  .auth-instructions :global(code) {
    font-size: var(--text-xs);
    padding: 2px 6px;
    background: var(--bg-elevated);
    border-radius: var(--radius-sm);
    font-family: "SF Mono", ui-monospace, monospace;
  }

  :global(.btn-icon) {
    width: 16px;
    height: 16px;
    margin-right: var(--space-2);
  }
</style>
