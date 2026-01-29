<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Loader2 } from "lucide-svelte";
  import AuthCard from "./AuthCard.svelte";
  import type { ApiKeyConfig } from "../types";

  type Provider = "anthropic" | "openai";

  interface Props {
    provider: Provider;
    config: ApiKeyConfig;
    onToggle: () => void;
    onValidate: () => void;
  }

  let { provider, config = $bindable(), onToggle, onValidate }: Props = $props();

  const titles: Record<Provider, string> = {
    anthropic: "Anthropic API Key",
    openai: "OpenAI API Key"
  };

  const descriptions: Record<Provider, string> = {
    anthropic: "Use your own Anthropic API key for direct access",
    openai: "Use OpenAI models as an alternative provider"
  };

  const placeholders: Record<Provider, string> = {
    anthropic: "sk-ant-...",
    openai: "sk-..."
  };

  const iconType = $derived(provider === "openai" ? "openai" : "anthropic");
</script>

<AuthCard
  title={titles[provider]}
  description={descriptions[provider]}
  {iconType}
  expanded={config.expanded}
  status={config.status}
  {onToggle}
>
  <div class="api-key-input-group">
    <input
      type="password"
      placeholder={placeholders[provider]}
      bind:value={config.value}
      class="api-key-input"
    />
    <Button
      variant="outline"
      onclick={onValidate}
      disabled={config.status === "validating" || !config.value}
    >
      {#if config.status === "validating"}
        <Loader2 class="btn-icon icon-spin" />
      {:else}
        Validate
      {/if}
    </Button>
  </div>
  {#if config.message}
    <p
      class="validation-message"
      class:success={config.status === "valid"}
      class:error={config.status === "invalid"}
    >
      {config.message}
    </p>
  {/if}
</AuthCard>

<style>
  .api-key-input-group {
    display: flex;
    gap: var(--space-3);
  }

  .api-key-input {
    flex: 1;
    padding: var(--space-3) var(--space-4);
    background: var(--bg-primary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-md);
    font-family: "SF Mono", ui-monospace, monospace;
    font-size: var(--text-sm);
    color: var(--text-primary);
  }

  .api-key-input:focus {
    border-color: var(--accent-hex);
    box-shadow: 0 0 0 3px var(--accent-glow);
    outline: none;
  }

  .validation-message {
    font-size: var(--text-xs);
    margin: var(--space-2) 0 0;
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm);
  }

  .validation-message.success {
    background: var(--success-glow);
    color: var(--success-hex);
  }

  .validation-message.error {
    background: var(--error-glow);
    color: var(--error);
  }

  :global(.btn-icon) {
    width: 16px;
    height: 16px;
    margin-right: var(--space-2);
  }

  :global(.icon-spin) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }
</style>
