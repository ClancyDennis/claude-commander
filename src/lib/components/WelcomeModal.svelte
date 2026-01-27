<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { createEventDispatcher, onMount } from "svelte";
  import logoIcon from "$lib/assets/claude-commander-icon.png";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import {
    Check,
    ArrowRight,
    Terminal,
    Key,
    Loader2,
    ExternalLink,
    Rocket,
    RefreshCw,
    Copy,
    ChevronDown,
    ChevronUp,
    AlertCircle
  } from "lucide-svelte";

  interface Props {
    show: boolean;
    onClose: () => void;
  }

  let { show = $bindable(), onClose }: Props = $props();

  const dispatch = createEventDispatcher<{ startTutorial: void }>();

  // Wizard state
  let currentStep = $state(1);
  const TOTAL_STEPS = 3;
  const STORAGE_KEY = "claude-commander-onboarding-complete";

  // Step 1: Claude Code Status
  let claudeStatus = $state<{
    checking: boolean;
    installed: boolean;
    path: string | null;
    version: string | null;
    authenticated: boolean;
    authType: string | null;
    subscriptionType: string | null;
  }>({
    checking: true,
    installed: false,
    path: null,
    version: null,
    authenticated: false,
    authType: null,
    subscriptionType: null
  });

  // Step 2: Authentication Options
  let authConfig = $state<{
    claudeAuth: { expanded: boolean; status: "unchecked" | "ready" };
    anthropicKey: {
      expanded: boolean;
      value: string;
      status: "unchecked" | "validating" | "valid" | "invalid";
      message: string;
    };
    openaiKey: {
      expanded: boolean;
      value: string;
      status: "unchecked" | "validating" | "valid" | "invalid";
      message: string;
    };
  }>({
    claudeAuth: { expanded: false, status: "unchecked" },
    anthropicKey: { expanded: false, value: "", status: "unchecked", message: "" },
    openaiKey: { expanded: false, value: "", status: "unchecked", message: "" }
  });

  // Command copied state
  let commandCopied = $state(false);

  // Derived: Can proceed from step 2 (at least one valid auth method)
  let hasValidAuth = $derived(
    authConfig.claudeAuth.status === "ready" ||
      authConfig.anthropicKey.status === "valid" ||
      authConfig.openaiKey.status === "valid"
  );

  // Check Claude Code on mount and when step 1 is shown
  $effect(() => {
    if (show && currentStep === 1 && claudeStatus.checking) {
      checkClaudeCode();
    }
  });

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

  async function copyInstallCommand() {
    try {
      await navigator.clipboard.writeText("npm install -g @anthropic-ai/claude-code");
      commandCopied = true;
      setTimeout(() => (commandCopied = false), 2000);
    } catch (e) {
      console.error("Failed to copy:", e);
    }
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

  function goToStep(step: number) {
    currentStep = step;
  }

  async function handleContinueToAuth() {
    goToStep(2);
  }

  async function handleContinueToReady() {
    await saveApiKeys();
    goToStep(3);
  }

  function handleStartTutorial() {
    completeOnboarding();
    onClose();
    // Delay tour start to avoid click propagation to backdrop
    setTimeout(() => {
      dispatch("startTutorial");
    }, 100);
  }

  function handleSkip() {
    completeOnboarding();
    onClose();
  }

  function completeOnboarding() {
    try {
      localStorage.setItem(STORAGE_KEY, "true");
    } catch (e) {
      console.error("Failed to save onboarding status:", e);
    }
  }

  // Keyboard shortcuts data
  const shortcuts = [
    { keys: ["Ctrl", "K"], label: "Command palette" },
    { keys: ["Ctrl", "Enter"], label: "Send message" },
    { keys: ["Ctrl", "Shift", "N"], label: "New agent" }
  ];
</script>

<Dialog.Root bind:open={show}>
  <Dialog.Content class="welcome-modal">
    <!-- Progress Indicator -->
    <div class="progress-dots">
      {#each Array(TOTAL_STEPS) as _, i}
        <div
          class="progress-dot"
          class:active={i + 1 === currentStep}
          class:completed={i + 1 < currentStep}
        ></div>
      {/each}
    </div>

    <!-- Step Content -->
    <div class="step-container">
      <!-- Step 1: Claude Code Check -->
      {#if currentStep === 1}
        <div class="step-content animate-scale-in">
          <!-- Welcome Header -->
          <div class="welcome-header">
            <img src={logoIcon} alt="Claude Commander" class="logo" />
            <h1>Welcome to Claude Commander</h1>
            <p class="subtitle">Let's get you set up in just a few steps</p>
          </div>

          <!-- Claude Code Status Card -->
          <div class="status-card" class:success={claudeStatus.installed}>
            <div
              class="status-icon"
              class:success={claudeStatus.installed}
              class:checking={claudeStatus.checking}
            >
              {#if claudeStatus.checking}
                <Loader2 class="icon-spin" />
              {:else if claudeStatus.installed}
                <Check />
              {:else}
                <Terminal />
              {/if}
            </div>

            <div class="status-content">
              <h3>Claude Code CLI</h3>
              {#if claudeStatus.checking}
                <p class="status-message">Checking installation...</p>
              {:else if claudeStatus.installed}
                <p class="status-message success">
                  Installed
                  {#if claudeStatus.version}
                    <span class="version">({claudeStatus.version})</span>
                  {/if}
                  {#if claudeStatus.authenticated}
                    <span class="auth-indicator"> · Authenticated</span>
                    {#if claudeStatus.subscriptionType}
                      <span class="subscription-badge">{claudeStatus.subscriptionType}</span>
                    {/if}
                  {/if}
                </p>
              {:else}
                <p class="status-message warning">Not installed</p>
              {/if}
            </div>

            {#if !claudeStatus.checking && claudeStatus.installed}
              <div class="status-badge success">
                <Check class="badge-icon" />
                Ready
              </div>
            {/if}
          </div>

          <!-- Install Instructions (shown when not installed) -->
          {#if !claudeStatus.installed && !claudeStatus.checking}
            <div class="install-section">
              <h4>Install with npm:</h4>
              <div class="install-command-wrapper">
                <code class="install-command">npm install -g @anthropic-ai/claude-code</code>
                <button class="copy-btn" onclick={copyInstallCommand} title="Copy to clipboard">
                  {#if commandCopied}
                    <Check class="copy-icon success" />
                  {:else}
                    <Copy class="copy-icon" />
                  {/if}
                </button>
              </div>

              <div class="install-actions">
                <Button variant="outline" onclick={recheckClaudeCode} class="recheck-btn">
                  <RefreshCw class="btn-icon" />
                  Re-check
                </Button>
                <Button
                  variant="ghost"
                  onclick={() =>
                    window.open("https://docs.anthropic.com/en/docs/claude-code", "_blank")}
                >
                  <ExternalLink class="btn-icon" />
                  Documentation
                </Button>
              </div>
            </div>
          {/if}

          <!-- Footer Actions -->
          <div class="step-footer">
            <div></div>
            <div class="footer-right">
              {#if claudeStatus.installed}
                <Button onclick={handleContinueToAuth} size="lg" class="continue-btn">
                  Continue
                  <ArrowRight class="btn-icon-right" />
                </Button>
              {:else}
                <Button variant="ghost" onclick={handleContinueToAuth}>Skip for now</Button>
              {/if}
            </div>
          </div>
        </div>
      {/if}

      <!-- Step 2: Authentication Setup -->
      {#if currentStep === 2}
        <div class="step-content animate-slide-up">
          <div class="step-header">
            <h2>Authentication</h2>
            <p class="subtitle">Choose one or more authentication methods</p>
          </div>

          <div class="auth-options">
            <!-- Claude Code Authentication -->
            <div class="auth-card" class:expanded={authConfig.claudeAuth.expanded}>
              <button class="auth-toggle" onclick={() => toggleAuthCard("claudeAuth")}>
                <div class="auth-icon claude">
                  <Terminal />
                </div>
                <div class="auth-info">
                  <h4>Claude Code Authentication</h4>
                  <p>Use existing Claude Code login (run <code>claude</code> to authenticate)</p>
                </div>
                <div class="auth-toggle-icon">
                  {#if authConfig.claudeAuth.status === "ready"}
                    <div class="status-badge success small">
                      <Check class="badge-icon" />
                    </div>
                  {:else if authConfig.claudeAuth.expanded}
                    <ChevronUp />
                  {:else}
                    <ChevronDown />
                  {/if}
                </div>
              </button>

              {#if authConfig.claudeAuth.expanded}
                <div class="auth-expand">
                  <p class="auth-instructions">
                    If you've already authenticated Claude Code in your terminal, click "I'm Ready"
                    below. Otherwise, open a terminal and run <code>claude</code> to start the
                    authentication process.
                  </p>
                  <Button
                    onclick={markClaudeAuthReady}
                    disabled={authConfig.claudeAuth.status === "ready"}
                  >
                    {#if authConfig.claudeAuth.status === "ready"}
                      <Check class="btn-icon" />
                      Authenticated
                    {:else}
                      I'm Ready
                    {/if}
                  </Button>
                </div>
              {/if}
            </div>

            <!-- Anthropic API Key -->
            <div class="auth-card" class:expanded={authConfig.anthropicKey.expanded}>
              <button class="auth-toggle" onclick={() => toggleAuthCard("anthropicKey")}>
                <div class="auth-icon anthropic">
                  <Key />
                </div>
                <div class="auth-info">
                  <h4>Anthropic API Key</h4>
                  <p>Use your own Anthropic API key for direct access</p>
                </div>
                <div class="auth-toggle-icon">
                  {#if authConfig.anthropicKey.status === "valid"}
                    <div class="status-badge success small">
                      <Check class="badge-icon" />
                    </div>
                  {:else if authConfig.anthropicKey.status === "invalid"}
                    <div class="status-badge error small">
                      <AlertCircle class="badge-icon" />
                    </div>
                  {:else if authConfig.anthropicKey.expanded}
                    <ChevronUp />
                  {:else}
                    <ChevronDown />
                  {/if}
                </div>
              </button>

              {#if authConfig.anthropicKey.expanded}
                <div class="auth-expand">
                  <div class="api-key-input-group">
                    <input
                      type="password"
                      placeholder="sk-ant-..."
                      bind:value={authConfig.anthropicKey.value}
                      class="api-key-input"
                    />
                    <Button
                      variant="outline"
                      onclick={() => validateApiKey("anthropic")}
                      disabled={authConfig.anthropicKey.status === "validating" ||
                        !authConfig.anthropicKey.value}
                    >
                      {#if authConfig.anthropicKey.status === "validating"}
                        <Loader2 class="btn-icon icon-spin" />
                      {:else}
                        Validate
                      {/if}
                    </Button>
                  </div>
                  {#if authConfig.anthropicKey.message}
                    <p
                      class="validation-message"
                      class:success={authConfig.anthropicKey.status === "valid"}
                      class:error={authConfig.anthropicKey.status === "invalid"}
                    >
                      {authConfig.anthropicKey.message}
                    </p>
                  {/if}
                </div>
              {/if}
            </div>

            <!-- OpenAI API Key -->
            <div class="auth-card" class:expanded={authConfig.openaiKey.expanded}>
              <button class="auth-toggle" onclick={() => toggleAuthCard("openaiKey")}>
                <div class="auth-icon openai">
                  <Key />
                </div>
                <div class="auth-info">
                  <h4>OpenAI API Key</h4>
                  <p>Use OpenAI models as an alternative provider</p>
                </div>
                <div class="auth-toggle-icon">
                  {#if authConfig.openaiKey.status === "valid"}
                    <div class="status-badge success small">
                      <Check class="badge-icon" />
                    </div>
                  {:else if authConfig.openaiKey.status === "invalid"}
                    <div class="status-badge error small">
                      <AlertCircle class="badge-icon" />
                    </div>
                  {:else if authConfig.openaiKey.expanded}
                    <ChevronUp />
                  {:else}
                    <ChevronDown />
                  {/if}
                </div>
              </button>

              {#if authConfig.openaiKey.expanded}
                <div class="auth-expand">
                  <div class="api-key-input-group">
                    <input
                      type="password"
                      placeholder="sk-..."
                      bind:value={authConfig.openaiKey.value}
                      class="api-key-input"
                    />
                    <Button
                      variant="outline"
                      onclick={() => validateApiKey("openai")}
                      disabled={authConfig.openaiKey.status === "validating" ||
                        !authConfig.openaiKey.value}
                    >
                      {#if authConfig.openaiKey.status === "validating"}
                        <Loader2 class="btn-icon icon-spin" />
                      {:else}
                        Validate
                      {/if}
                    </Button>
                  </div>
                  {#if authConfig.openaiKey.message}
                    <p
                      class="validation-message"
                      class:success={authConfig.openaiKey.status === "valid"}
                      class:error={authConfig.openaiKey.status === "invalid"}
                    >
                      {authConfig.openaiKey.message}
                    </p>
                  {/if}
                </div>
              {/if}
            </div>
          </div>

          <!-- Footer Actions -->
          <div class="step-footer">
            <Button variant="ghost" onclick={() => goToStep(1)}>Back</Button>
            <div class="footer-right">
              <Button variant="ghost" onclick={handleSkip}>Skip</Button>
              <Button onclick={handleContinueToReady} disabled={!hasValidAuth} size="lg">
                Continue
                <ArrowRight class="btn-icon-right" />
              </Button>
            </div>
          </div>
        </div>
      {/if}

      <!-- Step 3: Ready -->
      {#if currentStep === 3}
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

          <!-- Footer Actions -->
          <div class="step-footer final">
            <Button onclick={handleStartTutorial} size="lg" class="tour-btn">
              <Rocket class="btn-icon" />
              Take Interactive Tour
            </Button>
            <Button variant="ghost" onclick={handleSkip}>Skip and get started</Button>
          </div>
        </div>
      {/if}
    </div>
  </Dialog.Content>
</Dialog.Root>

<style>
  /* ============================================
     MODAL CONTAINER - Apple HIG
     ============================================ */

  :global(.welcome-modal) {
    max-width: 560px !important;
    padding: 0 !important;
    overflow: hidden;
    border-radius: var(--radius-xl) !important;
    background: var(--bg-secondary) !important;
    border: 1px solid var(--border-hex) !important;
    box-shadow: var(--shadow-xl) !important;
  }

  /* ============================================
     PROGRESS INDICATOR
     ============================================ */

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

  /* ============================================
     STEP CONTAINER
     ============================================ */

  .step-container {
    padding: 0 var(--space-8) var(--space-8);
  }

  .step-content {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }

  /* ============================================
     WELCOME HEADER
     ============================================ */

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

  h2 {
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

  .step-header {
    text-align: center;
  }

  .step-header .subtitle {
    margin-top: var(--space-2);
  }

  /* ============================================
     STATUS CARD
     ============================================ */

  .status-card {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-5) var(--space-6);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-lg);
    transition: all var(--transition-normal);
  }

  .status-card.success {
    border-color: rgba(52, 199, 89, 0.3);
    background: rgba(52, 199, 89, 0.08);
  }

  .status-icon {
    width: 48px;
    height: 48px;
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-elevated);
    color: var(--text-muted);
    transition: all var(--transition-normal);
  }

  .status-icon :global(svg) {
    width: 24px;
    height: 24px;
  }

  .status-icon.success {
    background: var(--success-glow);
    color: var(--success-hex);
  }

  .status-icon.checking {
    background: var(--accent-glow);
    color: var(--accent-hex);
  }

  .status-content {
    flex: 1;
  }

  .status-content h3 {
    font-size: var(--text-lg);
    font-weight: var(--font-medium);
    margin: 0 0 var(--space-1);
    color: var(--text-primary);
  }

  .status-message {
    font-size: var(--text-sm);
    color: var(--text-muted);
    margin: 0;
  }

  .status-message.success {
    color: var(--success-hex);
  }

  .status-message.warning {
    color: var(--warning-hex);
  }

  .version {
    opacity: 0.8;
    font-size: var(--text-xs);
  }

  .auth-indicator {
    color: var(--success-hex);
  }

  .subscription-badge {
    display: inline-block;
    margin-left: var(--space-2);
    padding: 2px 8px;
    font-size: var(--text-xs);
    font-weight: var(--font-medium);
    background: var(--accent-glow);
    color: var(--accent-hex);
    border-radius: var(--radius-full);
    text-transform: capitalize;
  }

  .status-badge {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    font-size: var(--text-xs);
    font-weight: var(--font-medium);
    padding: var(--space-1) var(--space-3);
    border-radius: var(--radius-full);
  }

  .status-badge.success {
    background: var(--success-glow);
    color: var(--success-hex);
  }

  .status-badge.error {
    background: var(--error-glow);
    color: var(--error);
  }

  .status-badge.small {
    padding: var(--space-1) var(--space-2);
  }

  .badge-icon {
    width: 12px;
    height: 12px;
  }

  /* ============================================
     INSTALL SECTION
     ============================================ */

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
  }

  .copy-btn:hover {
    background: var(--border-light);
    color: var(--text-primary);
  }

  .copy-icon {
    width: 18px;
    height: 18px;
  }

  .copy-icon.success {
    color: var(--success-hex);
  }

  .install-actions {
    display: flex;
    justify-content: center;
    gap: var(--space-3);
  }

  /* ============================================
     AUTH CARDS
     ============================================ */

  .auth-options {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .auth-card {
    background: var(--bg-tertiary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-lg);
    overflow: hidden;
    transition: all var(--transition-normal);
  }

  .auth-card.expanded {
    border-color: var(--accent-hex);
    box-shadow: 0 0 0 3px var(--accent-glow);
  }

  .auth-toggle {
    width: 100%;
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-4) var(--space-5);
    background: transparent;
    border: none;
    cursor: pointer;
    text-align: left;
  }

  .auth-toggle:hover {
    background: rgba(255, 255, 255, 0.03);
  }

  .auth-icon {
    width: 40px;
    height: 40px;
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-elevated);
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .auth-icon :global(svg) {
    width: 20px;
    height: 20px;
  }

  .auth-card.expanded .auth-icon {
    background: var(--accent-glow);
    color: var(--accent-hex);
  }

  .auth-icon.claude {
    background: rgba(232, 102, 77, 0.15);
    color: var(--accent-hex);
  }

  .auth-icon.anthropic {
    background: rgba(232, 102, 77, 0.15);
    color: var(--accent-hex);
  }

  .auth-icon.openai {
    background: rgba(16, 163, 127, 0.15);
    color: #10a37f;
  }

  .auth-info {
    flex: 1;
    min-width: 0;
  }

  .auth-info h4 {
    font-size: var(--text-base);
    font-weight: var(--font-medium);
    margin: 0 0 var(--space-1);
    color: var(--text-primary);
  }

  .auth-info p {
    font-size: var(--text-sm);
    color: var(--text-muted);
    margin: 0;
  }

  .auth-info code {
    font-size: var(--text-xs);
    padding: 2px 6px;
    background: var(--bg-elevated);
    border-radius: var(--radius-sm);
    font-family: "SF Mono", ui-monospace, monospace;
  }

  .auth-toggle-icon {
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .auth-toggle-icon :global(svg) {
    width: 18px;
    height: 18px;
  }

  .auth-expand {
    padding: 0 var(--space-5) var(--space-5);
    animation: slideUp 0.2s var(--spring-bounce);
  }

  .auth-instructions {
    font-size: var(--text-sm);
    color: var(--text-secondary);
    margin: 0 0 var(--space-4);
    line-height: var(--leading-relaxed);
  }

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

  /* ============================================
     READY STEP
     ============================================ */

  .ready-step {
    text-align: center;
    align-items: center;
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

  /* ============================================
     FOOTER
     ============================================ */

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

  /* ============================================
     UTILITIES
     ============================================ */

  .btn-icon {
    width: 16px;
    height: 16px;
    margin-right: var(--space-2);
  }

  .btn-icon-right {
    width: 16px;
    height: 16px;
    margin-left: var(--space-2);
  }

  .icon-spin {
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
