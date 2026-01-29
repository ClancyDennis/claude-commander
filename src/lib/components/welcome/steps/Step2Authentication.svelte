<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import ClaudeAuthCard from "../auth/ClaudeAuthCard.svelte";
  import ApiKeyAuthCard from "../auth/ApiKeyAuthCard.svelte";
  import WizardFooter from "../ui/WizardFooter.svelte";
  import type { AuthConfig } from "../types";

  interface Props {
    authConfig: AuthConfig;
    onNext: () => void;
    onBack: () => void;
    onSkip: () => void;
  }

  let { authConfig = $bindable(), onNext, onBack, onSkip }: Props = $props();

  // Derived: Can proceed from step 2 (at least one valid auth method)
  let hasValidAuth = $derived(
    authConfig.claudeAuth.status === "ready" ||
      authConfig.anthropicKey.status === "valid" ||
      authConfig.openaiKey.status === "valid"
  );

  function toggleAuthCard(card: "claudeAuth" | "anthropicKey" | "openaiKey") {
    if (card === "claudeAuth") {
      authConfig.claudeAuth.expanded = !authConfig.claudeAuth.expanded;
    } else if (card === "anthropicKey") {
      authConfig.anthropicKey.expanded = !authConfig.anthropicKey.expanded;
    } else {
      authConfig.openaiKey.expanded = !authConfig.openaiKey.expanded;
    }
  }

  function markClaudeAuthReady() {
    authConfig.claudeAuth.status = "ready";
  }

  async function validateApiKey(provider: "anthropic" | "openai") {
    const config = provider === "anthropic" ? authConfig.anthropicKey : authConfig.openaiKey;
    if (!config.value.trim()) return;

    if (provider === "anthropic") {
      authConfig.anthropicKey.status = "validating";
    } else {
      authConfig.openaiKey.status = "validating";
    }

    try {
      const result = await invoke<{ valid: boolean; message: string }>("validate_api_key", {
        provider,
        apiKey: config.value
      });

      if (provider === "anthropic") {
        authConfig.anthropicKey.status = result.valid ? "valid" : "invalid";
        authConfig.anthropicKey.message = result.message;
      } else {
        authConfig.openaiKey.status = result.valid ? "valid" : "invalid";
        authConfig.openaiKey.message = result.message;
      }
    } catch (e) {
      if (provider === "anthropic") {
        authConfig.anthropicKey.status = "invalid";
        authConfig.anthropicKey.message = String(e);
      } else {
        authConfig.openaiKey.status = "invalid";
        authConfig.openaiKey.message = String(e);
      }
    }
  }

  async function saveApiKeys() {
    const updates: { key: string; value: string }[] = [];

    if (authConfig.anthropicKey.status === "valid" && authConfig.anthropicKey.value) {
      updates.push({ key: "ANTHROPIC_API_KEY", value: authConfig.anthropicKey.value });
    }
    if (authConfig.openaiKey.status === "valid" && authConfig.openaiKey.value) {
      updates.push({ key: "OPENAI_API_KEY", value: authConfig.openaiKey.value });
    }

    if (updates.length > 0) {
      try {
        await invoke("update_config_batch", { updates });
      } catch (e) {
        console.error("Failed to save API keys:", e);
      }
    }
  }

  async function handleContinue() {
    await saveApiKeys();
    onNext();
  }
</script>

<div class="step-content animate-slide-up">
  <div class="step-header">
    <h2>Authentication</h2>
    <p class="subtitle">Choose one or more authentication methods</p>
  </div>

  <div class="auth-options">
    <ClaudeAuthCard
      bind:config={authConfig.claudeAuth}
      onToggle={() => toggleAuthCard("claudeAuth")}
      onMarkReady={markClaudeAuthReady}
    />

    <ApiKeyAuthCard
      provider="anthropic"
      bind:config={authConfig.anthropicKey}
      onToggle={() => toggleAuthCard("anthropicKey")}
      onValidate={() => validateApiKey("anthropic")}
    />

    <ApiKeyAuthCard
      provider="openai"
      bind:config={authConfig.openaiKey}
      onToggle={() => toggleAuthCard("openaiKey")}
      onValidate={() => validateApiKey("openai")}
    />
  </div>

  <WizardFooter
    showBack={true}
    showSkip={true}
    showContinue={true}
    continueDisabled={!hasValidAuth}
    continueLabel="Continue"
    {onBack}
    {onSkip}
    onContinue={handleContinue}
  />
</div>

<style>
  .step-content {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }

  .step-header {
    text-align: center;
  }

  .step-header h2 {
    font-size: var(--text-xl);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
    margin: 0;
    letter-spacing: -0.01em;
  }

  .subtitle {
    font-size: var(--text-base);
    color: var(--text-secondary);
    margin: var(--space-2) 0 0;
    line-height: var(--leading-relaxed);
  }

  .auth-options {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .animate-slide-up {
    animation: slideUp 0.3s var(--spring-bounce);
  }

  @keyframes slideUp {
    from {
      opacity: 0;
      transform: translateY(12px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
