<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import logoIcon from "$lib/assets/claude-commander-icon.png";
  import StatusCard from "../ui/StatusCard.svelte";
  import InstallCommand from "../ui/InstallCommand.svelte";
  import WizardFooter from "../ui/WizardFooter.svelte";
  import type { ClaudeCodeStatus, AuthConfig } from "../types";

  interface Props {
    claudeStatus: ClaudeCodeStatus;
    authConfig: AuthConfig;
    onNext: () => void;
    onSkip: () => void;
  }

  let { claudeStatus = $bindable(), authConfig = $bindable(), onNext, onSkip }: Props = $props();

  async function checkClaudeCode() {
    claudeStatus.checking = true;
    try {
      const status = await invoke<{
        installed: boolean;
        path: string | null;
        version: string | null;
        authenticated: boolean;
        auth_type: string | null;
        subscription_type: string | null;
      }>("check_claude_code_installed");

      claudeStatus = {
        checking: false,
        installed: status.installed,
        path: status.path,
        version: status.version,
        authenticated: status.authenticated,
        authType: status.auth_type,
        subscriptionType: status.subscription_type
      };

      // If Claude Code is already authenticated, set the auth status
      if (status.authenticated) {
        authConfig.claudeAuth.status = "ready";
      }
    } catch (e) {
      console.error("Failed to check Claude Code:", e);
      claudeStatus = {
        checking: false,
        installed: false,
        path: null,
        version: null,
        authenticated: false,
        authType: null,
        subscriptionType: null
      };
    }
  }

  async function recheckClaudeCode() {
    claudeStatus.checking = true;
    await checkClaudeCode();
  }

  onMount(() => {
    if (claudeStatus.checking) {
      checkClaudeCode();
    }
  });

  const statusType = $derived.by(() => {
    if (claudeStatus.checking) return "checking";
    if (claudeStatus.installed) return "success";
    return "warning";
  });

  const statusMessage = $derived(claudeStatus.installed ? "Installed" : "Not installed");
</script>

<div class="step-content animate-scale-in">
  <!-- Welcome Header -->
  <div class="welcome-header">
    <img src={logoIcon} alt="Claude Commander" class="logo" />
    <h1>Welcome to Claude Commander</h1>
    <p class="subtitle">Let's get you set up in just a few steps</p>
  </div>

  <!-- Claude Code Status Card -->
  <StatusCard
    title="Claude Code CLI"
    status={statusType}
    message={statusMessage}
    version={claudeStatus.version}
    authenticated={claudeStatus.authenticated}
    subscriptionType={claudeStatus.subscriptionType}
    showBadge={claudeStatus.installed && !claudeStatus.checking}
    badgeText="Ready"
  />

  <!-- Install Instructions (shown when not installed) -->
  {#if !claudeStatus.installed && !claudeStatus.checking}
    <InstallCommand command="npm install -g @anthropic-ai/claude-code" onRecheck={recheckClaudeCode} />
  {/if}

  <!-- Footer Actions -->
  <WizardFooter
    showContinue={claudeStatus.installed}
    showSkip={!claudeStatus.installed}
    continueLabel={claudeStatus.installed ? "Continue" : "Skip for now"}
    onContinue={claudeStatus.installed ? onNext : onNext}
    onSkip={claudeStatus.installed ? undefined : onNext}
  />
</div>

<style>
  .step-content {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }

  .welcome-header {
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-3);
  }

  .logo {
    width: 72px;
    height: 72px;
    border-radius: 18px;
    box-shadow: var(--shadow-lg);
  }

  h1 {
    font-size: var(--text-2xl);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
    margin: 0;
    letter-spacing: -0.02em;
  }

  .subtitle {
    font-size: var(--text-base);
    color: var(--text-secondary);
    margin: 0;
    line-height: var(--leading-relaxed);
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
