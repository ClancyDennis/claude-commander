<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import HelpTip from "./new-agent/HelpTip.svelte";
  import type { ConfigStatus, ConfigUpdate, ConfigUpdateResult, ApiKeyValidationResult } from "$lib/types";

  let { onClose }: { onClose?: () => void } = $props();

  let config = $state<ConfigStatus | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  // Edit mode state
  let isEditing = $state(false);
  let isSaving = $state(false);
  let saveError = $state<string | null>(null);
  let showRestartNotice = $state(false);

  // Form state for API keys
  let editedApiKeys = $state<Record<string, string>>({
    ANTHROPIC_API_KEY: "",
    OPENAI_API_KEY: "",
    GITHUB_TOKEN: "",
  });

  // Form state for models
  let editedModels = $state<Record<string, string>>({
    ANTHROPIC_MODEL: "",
    SECURITY_MODEL: "",
    LIGHT_TASK_MODEL: "",
    OPENAI_MODEL: "",
  });

  // Form state for advanced settings
  let editedApiKeyMode = $state<string>("");

  // Validation state
  let validationStatus = $state<Record<string, { valid: boolean | null; message: string; loading: boolean }>>({});

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
      "LIGHT_TASK_MODEL": "Model for lightweight tasks like title generation (faster/cheaper)",
      "OPENAI_MODEL": "OpenAI model for tasks using GPT (requires OpenAI API key)"
    };
    return descriptions[name] || "";
  }

  function getStatusColor(isConfigured: boolean): string {
    return isConfigured ? "var(--success)" : "var(--warning)";
  }

  function getProviderKeyName(provider: string): string {
    const mapping: Record<string, string> = {
      "Anthropic": "ANTHROPIC_API_KEY",
      "OpenAI": "OPENAI_API_KEY",
      "GitHub": "GITHUB_TOKEN",
    };
    return mapping[provider] || "";
  }

  function startEditing() {
    isEditing = true;
    saveError = null;
    showRestartNotice = false;

    // Initialize with empty values (empty means keep existing)
    editedApiKeys = {
      ANTHROPIC_API_KEY: "",
      OPENAI_API_KEY: "",
      GITHUB_TOKEN: "",
    };

    // Initialize models with current values
    if (config) {
      editedModels = {
        ANTHROPIC_MODEL: config.models.find(m => m.name === "ANTHROPIC_MODEL")?.value || "",
        SECURITY_MODEL: config.models.find(m => m.name === "SECURITY_MODEL")?.value || "",
        LIGHT_TASK_MODEL: config.models.find(m => m.name === "LIGHT_TASK_MODEL")?.value || "",
        OPENAI_MODEL: config.models.find(m => m.name === "OPENAI_MODEL")?.value || "",
      };
      // Initialize API key mode
      editedApiKeyMode = config.models.find(m => m.name === "CLAUDE_CODE_API_KEY_MODE")?.value || "";
    }

    // Reset validation state
    validationStatus = {};
  }

  function cancelEditing() {
    isEditing = false;
    editedApiKeys = { ANTHROPIC_API_KEY: "", OPENAI_API_KEY: "", GITHUB_TOKEN: "" };
    editedModels = { ANTHROPIC_MODEL: "", SECURITY_MODEL: "", LIGHT_TASK_MODEL: "", OPENAI_MODEL: "" };
    editedApiKeyMode = "";
    validationStatus = {};
    saveError = null;
  }

  async function validateApiKey(provider: string) {
    const keyName = getProviderKeyName(provider);
    const keyValue = editedApiKeys[keyName];

    if (!keyValue.trim()) {
      validationStatus[keyName] = { valid: null, message: "Enter a key to validate", loading: false };
      return;
    }

    validationStatus[keyName] = { valid: null, message: "", loading: true };

    try {
      const result = await invoke<ApiKeyValidationResult>("validate_api_key", {
        provider: provider.toLowerCase(),
        apiKey: keyValue,
      });

      validationStatus[keyName] = {
        valid: result.valid,
        message: result.message,
        loading: false,
      };
    } catch (err) {
      validationStatus[keyName] = {
        valid: false,
        message: String(err),
        loading: false,
      };
    }
  }

  async function saveChanges() {
    isSaving = true;
    saveError = null;

    try {
      const updates: ConfigUpdate[] = [];

      // Collect API key changes (only if not empty)
      for (const [key, value] of Object.entries(editedApiKeys)) {
        if (value.trim()) {
          updates.push({ key, value: value.trim() });
        }
      }

      // Collect model changes
      for (const [key, value] of Object.entries(editedModels)) {
        // Always include model changes (empty string removes the override)
        const currentValue = config?.models.find(m => m.name === key)?.value || "";
        if (value !== currentValue) {
          updates.push({ key, value: value.trim() });
        }
      }

      // Collect API key mode change
      const currentApiKeyMode = config?.models.find(m => m.name === "CLAUDE_CODE_API_KEY_MODE")?.value || "";
      if (editedApiKeyMode !== currentApiKeyMode) {
        updates.push({ key: "CLAUDE_CODE_API_KEY_MODE", value: editedApiKeyMode.trim() });
      }

      if (updates.length === 0) {
        // No changes to save
        isEditing = false;
        return;
      }

      const result = await invoke<ConfigUpdateResult>("update_config_batch", { updates });

      if (result.success) {
        if (result.requires_restart) {
          showRestartNotice = true;
        }
        await fetchConfig();
        isEditing = false;
      } else {
        saveError = result.message;
      }
    } catch (err) {
      saveError = String(err);
    } finally {
      isSaving = false;
    }
  }

  async function openConfigDirectory() {
    try {
      await invoke("open_config_directory");
    } catch (err) {
      console.error("Failed to open config directory:", err);
    }
  }

  function getAvailableModels(): string[] {
    if (!config) return [];
    return [...config.available_claude_models, ...config.available_openai_models];
  }
</script>

<div class="settings">
  <header class="settings-header">
    <h2>Settings</h2>
    <div class="header-actions">
      <button class="refresh-btn" onclick={fetchConfig} disabled={loading || isEditing} title="Refresh">
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

  {#if showRestartNotice}
    <div class="restart-notice">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <line x1="12" y1="8" x2="12" y2="12"/>
        <line x1="12" y1="16" x2="12.01" y2="16"/>
      </svg>
      <span>Configuration saved. Restart the app for API key changes to take full effect.</span>
      <button class="dismiss-btn" onclick={() => showRestartNotice = false}>Dismiss</button>
    </div>
  {/if}

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
      {#if config.provider !== "None"}
        <div class="active-models">
          <span class="active-models-label">Active Models:</span>
          <div class="active-models-list">
            {#each config.models.filter(m => m.value && m.name !== "CLAUDE_CODE_API_KEY_MODE") as model}
              <span class="active-model-chip" title={model.name}>
                <span class="model-role">{model.name.replace(/_/g, " ").toLowerCase()}:</span>
                {model.value}
              </span>
            {/each}
            {#if config.models.filter(m => m.value && m.name !== "CLAUDE_CODE_API_KEY_MODE").length === 0}
              <span class="active-model-chip default">Using defaults</span>
            {/if}
          </div>
        </div>
      {/if}
    </section>

    <!-- API Keys -->
    <section class="config-section">
      <div class="section-header">
        <h3>API Keys</h3>
        {#if !isEditing}
          <button class="edit-btn" onclick={startEditing}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7"/>
              <path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z"/>
            </svg>
            Edit
          </button>
        {/if}
      </div>
      <div class="api-keys-list">
        {#each config.api_keys as apiKey}
          {@const keyName = getProviderKeyName(apiKey.provider)}
          <div class="api-key-item">
            <div class="api-key-header">
              <span class="api-key-provider">{apiKey.provider}</span>
              {#if !isEditing}
                <span
                  class="api-key-status"
                  style="color: {getStatusColor(apiKey.is_configured)}"
                >
                  {apiKey.is_configured ? "Configured" : "Not Configured"}
                </span>
              {/if}
            </div>
            {#if isEditing}
              <div class="api-key-edit">
                <input
                  type="password"
                  bind:value={editedApiKeys[keyName]}
                  placeholder={apiKey.is_configured ? "Leave empty to keep current" : "Enter API key"}
                  class="api-key-input"
                />
                <button
                  class="validate-btn"
                  onclick={() => validateApiKey(apiKey.provider)}
                  disabled={validationStatus[keyName]?.loading}
                >
                  {validationStatus[keyName]?.loading ? "..." : "Test"}
                </button>
              </div>
              {#if validationStatus[keyName]?.message}
                <div class="validation-result" class:valid={validationStatus[keyName]?.valid} class:invalid={validationStatus[keyName]?.valid === false}>
                  {validationStatus[keyName]?.message}
                </div>
              {/if}
            {:else}
              <div class="api-key-preview">
                <code>{apiKey.key_preview}</code>
              </div>
            {/if}
          </div>
        {/each}

        <!-- GitHub Token (not shown in api_keys by default) -->
        {#if isEditing}
          <div class="api-key-item">
            <div class="api-key-header">
              <span class="api-key-provider">GitHub</span>
            </div>
            <div class="api-key-edit">
              <input
                type="password"
                bind:value={editedApiKeys.GITHUB_TOKEN}
                placeholder="Enter GitHub token (optional)"
                class="api-key-input"
              />
              <button
                class="validate-btn"
                onclick={() => validateApiKey("GitHub")}
                disabled={validationStatus.GITHUB_TOKEN?.loading}
              >
                {validationStatus.GITHUB_TOKEN?.loading ? "..." : "Test"}
              </button>
            </div>
            {#if validationStatus.GITHUB_TOKEN?.message}
              <div class="validation-result" class:valid={validationStatus.GITHUB_TOKEN?.valid} class:invalid={validationStatus.GITHUB_TOKEN?.valid === false}>
                {validationStatus.GITHUB_TOKEN?.message}
              </div>
            {/if}
          </div>
        {/if}
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
              {#if !isEditing && model.is_default}
                <span class="default-badge">Default</span>
              {/if}
              <HelpTip text={getModelDescription(model.name)} placement="top" />
            </div>
            {#if isEditing}
              <select
                bind:value={editedModels[model.name]}
                class="model-select"
              >
                <option value="">Use default</option>
                {#each getAvailableModels() as modelOption}
                  <option value={modelOption}>{modelOption}</option>
                {/each}
              </select>
            {:else}
              <div class="model-value">
                <code>{model.value ?? "(using default)"}</code>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </section>

    <!-- Advanced Settings (always visible, toggleable when editing) -->
    <section class="config-section">
      <h3>Advanced Settings</h3>
      <div class="advanced-setting-item">
        <div class="advanced-setting-header">
          <span class="advanced-setting-name">Claude Code Local Authentication</span>
          <HelpTip text="Allows Claude Code agent instances to use their own authentication (OAuth login or separate API key) instead of requiring the Anthropic API key configured above." placement="top" />
        </div>
        <p class="advanced-setting-description">
          Allow Claude Code agent instances to use their local authentication (OAuth or separate API key) instead of the Anthropic API key configured here. The meta-agent still uses the API key above, but spawned Claude Code agents can authenticate independently.
        </p>
        {#if isEditing}
          <div class="toggle-wrapper">
            <label class="toggle">
              <input
                type="checkbox"
                checked={editedApiKeyMode === "local"}
                onchange={(e) => editedApiKeyMode = e.currentTarget.checked ? "local" : ""}
              />
              <span class="toggle-slider"></span>
            </label>
            <span class="toggle-label">{editedApiKeyMode === "local" ? "Enabled" : "Disabled"}</span>
          </div>
        {:else}
          {@const currentMode = config.models.find(m => m.name === "CLAUDE_CODE_API_KEY_MODE")?.value}
          <div class="advanced-setting-value">
            <span class="status-badge" class:enabled={currentMode === "local"}>
              {currentMode === "local" ? "Enabled" : "Disabled"}
            </span>
          </div>
        {/if}
      </div>
    </section>

    <!-- Edit Actions -->
    {#if isEditing}
      {#if saveError}
        <div class="save-error">
          {saveError}
        </div>
      {/if}
      <div class="edit-actions">
        <button class="btn-primary" onclick={saveChanges} disabled={isSaving}>
          {isSaving ? "Saving..." : "Save Changes"}
        </button>
        <button class="btn-secondary" onclick={cancelEditing} disabled={isSaving}>
          Cancel
        </button>
      </div>
    {:else}
      <!-- Configuration Actions -->
      <section class="config-section">
        <h3>Configuration</h3>
        <p class="config-path-info">
          Settings are stored at: <code>{config.config_path}</code>
        </p>
        <div class="config-actions">
          <button class="btn-secondary" onclick={openConfigDirectory}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/>
            </svg>
            Open Config Folder
          </button>
        </div>
      </section>
    {/if}
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

  .restart-notice {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-md);
    background: rgba(245, 158, 11, 0.15);
    border: 1px solid var(--warning);
    border-radius: 8px;
    color: var(--warning);
    font-size: 13px;
    margin-bottom: var(--space-lg);
  }

  .restart-notice svg {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
  }

  .restart-notice span {
    flex: 1;
  }

  .dismiss-btn {
    padding: 4px 12px;
    background: transparent;
    border: 1px solid var(--warning);
    border-radius: 4px;
    color: var(--warning);
    cursor: pointer;
    font-size: 12px;
  }

  .dismiss-btn:hover {
    background: rgba(245, 158, 11, 0.2);
  }

  .config-section {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: var(--space-lg);
    margin-bottom: var(--space-lg);
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-md);
  }

  .section-header h3 {
    margin: 0;
  }

  .config-section h3 {
    margin: 0 0 var(--space-md) 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .edit-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 13px;
    transition: all 0.2s ease;
  }

  .edit-btn:hover {
    background: var(--accent-glow);
    border-color: var(--accent);
    color: var(--text-primary);
  }

  .edit-btn svg {
    width: 14px;
    height: 14px;
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

  .api-key-edit {
    display: flex;
    gap: var(--space-sm);
  }

  .api-key-input {
    flex: 1;
    padding: 8px 12px;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-primary);
    font-family: monospace;
    font-size: 13px;
  }

  .api-key-input:focus {
    outline: none;
    border-color: var(--accent);
  }

  .api-key-input::placeholder {
    color: var(--text-muted);
    font-family: inherit;
  }

  .validate-btn {
    padding: 8px 16px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 13px;
    transition: all 0.2s ease;
  }

  .validate-btn:hover:not(:disabled) {
    background: var(--accent-glow);
    border-color: var(--accent);
    color: var(--text-primary);
  }

  .validate-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .validation-result {
    margin-top: var(--space-sm);
    font-size: 12px;
    padding: 4px 8px;
    border-radius: 4px;
  }

  .validation-result.valid {
    color: var(--success);
    background: rgba(34, 197, 94, 0.1);
  }

  .validation-result.invalid {
    color: var(--error);
    background: rgba(239, 68, 68, 0.1);
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

  .model-select {
    width: 100%;
    padding: 8px 12px;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-primary);
    font-size: 13px;
    cursor: pointer;
  }

  .model-select:focus {
    outline: none;
    border-color: var(--accent);
  }

  /* Active Models in Provider Card */
  .active-models {
    margin-top: var(--space-md);
    padding-top: var(--space-md);
    border-top: 1px solid var(--border);
  }

  .active-models-label {
    font-size: 12px;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    display: block;
    margin-bottom: var(--space-sm);
  }

  .active-models-list {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-sm);
  }

  .active-model-chip {
    font-size: 12px;
    padding: 4px 10px;
    background: var(--bg-tertiary);
    color: var(--text-primary);
    border-radius: 6px;
    border: 1px solid var(--border);
    font-family: monospace;
  }

  .active-model-chip .model-role {
    color: var(--text-muted);
    font-family: inherit;
    text-transform: capitalize;
  }

  .active-model-chip.default {
    color: var(--text-muted);
    font-style: italic;
  }

  /* Advanced Settings */
  .advanced-setting-item {
    padding: var(--space-md);
    background: var(--bg-tertiary);
    border-radius: 8px;
    border: 1px solid var(--border);
  }

  .advanced-setting-header {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    margin-bottom: var(--space-xs);
  }

  .advanced-setting-name {
    font-weight: 600;
    color: var(--text-primary);
  }

  .advanced-setting-description {
    font-size: 13px;
    color: var(--text-muted);
    margin: 0 0 var(--space-md) 0;
    line-height: 1.5;
  }

  .advanced-setting-value {
    display: flex;
    align-items: center;
  }

  .status-badge {
    font-size: 12px;
    padding: 4px 10px;
    background: var(--bg-primary);
    color: var(--text-muted);
    border-radius: 4px;
    border: 1px solid var(--border);
  }

  .status-badge.enabled {
    background: rgba(34, 197, 94, 0.1);
    color: var(--success);
    border-color: rgba(34, 197, 94, 0.3);
  }

  /* Toggle Switch */
  .toggle-wrapper {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .toggle {
    position: relative;
    display: inline-block;
    width: 44px;
    height: 24px;
  }

  .toggle input {
    opacity: 0;
    width: 0;
    height: 0;
  }

  .toggle-slider {
    position: absolute;
    cursor: pointer;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 24px;
    transition: all 0.2s ease;
  }

  .toggle-slider:before {
    position: absolute;
    content: "";
    height: 18px;
    width: 18px;
    left: 2px;
    bottom: 2px;
    background-color: var(--text-muted);
    border-radius: 50%;
    transition: all 0.2s ease;
  }

  .toggle input:checked + .toggle-slider {
    background-color: var(--accent);
    border-color: var(--accent);
  }

  .toggle input:checked + .toggle-slider:before {
    transform: translateX(20px);
    background-color: white;
  }

  .toggle-label {
    font-size: 13px;
    color: var(--text-secondary);
  }

  .config-path-info {
    color: var(--text-secondary);
    margin: 0 0 var(--space-md) 0;
    font-size: 13px;
  }

  .config-path-info code {
    background: var(--bg-tertiary);
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 12px;
  }

  .config-actions {
    display: flex;
    gap: var(--space-md);
  }

  .edit-actions {
    display: flex;
    gap: var(--space-md);
    padding: var(--space-lg);
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 12px;
    margin-bottom: var(--space-lg);
  }

  .save-error {
    padding: var(--space-md);
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid var(--error);
    border-radius: 8px;
    color: var(--error);
    font-size: 13px;
    margin-bottom: var(--space-md);
  }

  .btn-primary {
    padding: 10px 20px;
    background: var(--accent);
    color: white;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-size: 14px;
    font-weight: 500;
    transition: all 0.2s ease;
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-secondary {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 20px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 8px;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 14px;
    transition: all 0.2s ease;
  }

  .btn-secondary:hover:not(:disabled) {
    background: var(--accent-glow);
    border-color: var(--accent);
    color: var(--text-primary);
  }

  .btn-secondary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-secondary svg {
    width: 16px;
    height: 16px;
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
