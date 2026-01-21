<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import HelpTip from "./new-agent/HelpTip.svelte";
  import type { ConfigStatus } from "$lib/types";

  let { onClose }: { onClose?: () => void } = $props();

  let config = $state<ConfigStatus | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  async function fetchConfig() {
    try {
      loading = true;
      error = null;
      config = await invoke<ConfigStatus>("get_config_status");
    } catch (err) {
      console.error("Failed to fetch config:", err);
      error = String(err);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    fetchConfig();
  });

  function getModelDescription(name: string): string {
    const descriptions: Record<string, string> = {
      "ANTHROPIC_MODEL": "Primary model for agent tasks and conversations",
      "SECURITY_MODEL": "Model for security-critical analysis (higher capability recommended)",
      "LIGHT_TASK_MODEL": "Model for lightweight tasks like title generation (faster/cheaper)"
    };
    return descriptions[name] || "";
  }

  function getStatusColor(isConfigured: boolean): string {
    return isConfigured ? "var(--success)" : "var(--warning)";
  }
</script>

<div class="settings">
  <header class="settings-header">
    <h2>Settings</h2>
    <div class="header-actions">
      <button class="refresh-btn" onclick={fetchConfig} disabled={loading} title="Refresh">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M1 4v6h6M23 20v-6h-6"/>
          <path d="M20.49 9A9 9 0 005.64 5.64L1 10m22 4l-4.64 4.36A9 9 0 014.51 15"/>
        </svg>
      </button>
      {#if onClose}
        <button class="close-btn" onclick={onClose} title="Close">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      {/if}
    </div>
  </header>

  {#if loading && !config}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Loading configuration...</p>
    </div>
  {:else if error}
    <div class="error-state">
      <p>Failed to load configuration: {error}</p>
      <button onclick={fetchConfig}>Retry</button>
    </div>
  {:else if config}
    <!-- Active Provider -->
    <section class="config-section">
      <h3>Active Provider</h3>
      <div class="provider-card">
        <div class="provider-icon" style="background: var(--accent);">
          <svg viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <path d="M12 6v6l4 2"/>
          </svg>
        </div>
        <div class="provider-info">
          <span class="provider-name">{config.provider}</span>
          <span class="provider-status">
            {config.provider === "None" ? "No API key configured" : "Active"}
          </span>
        </div>
      </div>
    </section>

    <!-- API Keys -->
    <section class="config-section">
      <h3>API Keys</h3>
      <div class="api-keys-list">
        {#each config.api_keys as apiKey}
          <div class="api-key-item">
            <div class="api-key-header">
              <span class="api-key-provider">{apiKey.provider}</span>
              <span
                class="api-key-status"
                style="color: {getStatusColor(apiKey.is_configured)}"
              >
                {apiKey.is_configured ? "Configured" : "Not Configured"}
              </span>
            </div>
            <div class="api-key-preview">
              <code>{apiKey.key_preview}</code>
            </div>
          </div>
        {/each}
      </div>
    </section>

    <!-- Model Configuration -->
    <section class="config-section">
      <h3>Model Configuration</h3>
      <div class="models-list">
        {#each config.models as model}
          <div class="model-item">
            <div class="model-header">
              <span class="model-name">{model.name}</span>
              {#if model.is_default}
                <span class="default-badge">Default</span>
              {/if}
              <HelpTip text={getModelDescription(model.name)} placement="top" />
            </div>
            <div class="model-value">
              <code>{model.value ?? "(using default)"}</code>
            </div>
          </div>
        {/each}
      </div>
    </section>

    <!-- Available Models Reference -->
    <section class="config-section">
      <h3>Available Claude Models</h3>
      <div class="available-models">
        {#each config.available_claude_models as model}
          <span class="model-tag">{model}</span>
        {/each}
      </div>
    </section>

    <!-- Available OpenAI Models -->
    {#if config.available_openai_models.length > 0}
      <section class="config-section">
        <h3>Available OpenAI Models</h3>
        <div class="available-models">
          {#each config.available_openai_models as model}
            <span class="model-tag openai">{model}</span>
          {/each}
        </div>
      </section>
    {/if}

    <!-- Configuration Help -->
    <section class="config-section help-section">
      <h3>How to Configure</h3>
      <p>
        Edit the <code>.env</code> file in the project root to change configuration.
        Changes require restarting the application.
      </p>
      <div class="env-path">
        <code>tauri_server/.env</code>
      </div>
    </section>
  {/if}
</div>

<style>
  .settings {
    padding: var(--space-lg);
    max-width: 800px;
    margin: 0 auto;
    height: 100%;
    overflow-y: auto;
  }

  .settings-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-xl);
  }

  .settings-header h2 {
    margin: 0;
    font-size: 28px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .refresh-btn,
  .close-btn {
    width: 36px;
    height: 36px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .refresh-btn:hover:not(:disabled),
  .close-btn:hover {
    background: var(--accent-glow);
    border-color: var(--accent);
    color: var(--text-primary);
  }

  .refresh-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .refresh-btn svg,
  .close-btn svg {
    width: 18px;
    height: 18px;
  }

  .config-section {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: var(--space-lg);
    margin-bottom: var(--space-lg);
  }

  .config-section h3 {
    margin: 0 0 var(--space-md) 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .provider-card {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-md);
    background: var(--bg-tertiary);
    border-radius: 10px;
  }

  .provider-icon {
    width: 48px;
    height: 48px;
    border-radius: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .provider-icon svg {
    width: 24px;
    height: 24px;
  }

  .provider-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .provider-name {
    font-size: 18px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .provider-status {
    font-size: 13px;
    color: var(--text-muted);
  }

  .api-keys-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .api-key-item {
    padding: var(--space-md);
    background: var(--bg-tertiary);
    border-radius: 8px;
    border: 1px solid var(--border);
  }

  .api-key-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-sm);
  }

  .api-key-provider {
    font-weight: 600;
    color: var(--text-primary);
  }

  .api-key-status {
    font-size: 13px;
    font-weight: 500;
  }

  .api-key-preview code {
    font-family: monospace;
    font-size: 13px;
    color: var(--text-muted);
    background: var(--bg-primary);
    padding: 4px 8px;
    border-radius: 4px;
  }

  .models-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .model-item {
    padding: var(--space-md);
    background: var(--bg-tertiary);
    border-radius: 8px;
    border: 1px solid var(--border);
  }

  .model-header {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    margin-bottom: var(--space-sm);
  }

  .model-name {
    font-weight: 600;
    color: var(--text-primary);
    font-family: monospace;
  }

  .default-badge {
    font-size: 11px;
    padding: 2px 8px;
    background: rgba(124, 58, 237, 0.2);
    color: var(--accent);
    border-radius: 4px;
    font-weight: 500;
  }

  .model-value code {
    font-family: monospace;
    font-size: 13px;
    color: var(--text-secondary);
  }

  .available-models {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-sm);
  }

  .model-tag {
    font-size: 12px;
    padding: 4px 10px;
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    border-radius: 6px;
    border: 1px solid var(--border);
    font-family: monospace;
  }

  .model-tag.openai {
    border-color: rgba(16, 163, 127, 0.3);
    background: rgba(16, 163, 127, 0.1);
  }

  .help-section p {
    color: var(--text-secondary);
    margin: 0 0 var(--space-md) 0;
    line-height: 1.5;
  }

  .help-section code {
    background: var(--bg-tertiary);
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 13px;
  }

  .env-path code {
    display: inline-block;
    font-family: monospace;
    font-size: 13px;
    padding: 8px 12px;
    background: var(--bg-tertiary);
    border-radius: 6px;
    border: 1px solid var(--border);
    color: var(--text-primary);
  }

  .loading-state, .error-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-xl);
    gap: var(--space-md);
    color: var(--text-secondary);
  }

  .error-state button {
    padding: 8px 16px;
    background: var(--accent);
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
  }

  .error-state button:hover {
    background: var(--accent-hover);
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
