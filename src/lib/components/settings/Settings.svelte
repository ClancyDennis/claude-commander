<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import type { ConfigStatus, ConfigUpdate, ConfigUpdateResult, ApiKeyValidationResult } from "$lib/types";
  import { useAsyncData } from "$lib/hooks/useAsyncData.svelte";

  import ApiKeysSection from "./ApiKeysSection.svelte";
  import ModelConfigSection from "./ModelConfigSection.svelte";
  import AdvancedSettingsSection from "./AdvancedSettingsSection.svelte";
  import EditActionsFooter from "./EditActionsFooter.svelte";

  let { onClose }: { onClose?: () => void } = $props();

  // Config data fetching
  const configData = useAsyncData(() => invoke<ConfigStatus>("get_config_status"));

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

  onMount(() => {
    configData.fetch();
  });

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
    if (configData.data) {
      editedModels = {
        ANTHROPIC_MODEL: configData.data.models.find(m => m.name === "ANTHROPIC_MODEL")?.value || "",
        SECURITY_MODEL: configData.data.models.find(m => m.name === "SECURITY_MODEL")?.value || "",
        LIGHT_TASK_MODEL: configData.data.models.find(m => m.name === "LIGHT_TASK_MODEL")?.value || "",
        OPENAI_MODEL: configData.data.models.find(m => m.name === "OPENAI_MODEL")?.value || "",
      };
      // Initialize API key mode
      editedApiKeyMode = configData.data.models.find(m => m.name === "CLAUDE_CODE_API_KEY_MODE")?.value || "";
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
        const currentValue = configData.data?.models.find(m => m.name === key)?.value || "";
        if (value !== currentValue) {
          updates.push({ key, value: value.trim() });
        }
      }

      // Collect API key mode change
      const currentApiKeyMode = configData.data?.models.find(m => m.name === "CLAUDE_CODE_API_KEY_MODE")?.value || "";
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
        await configData.fetch();
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
    if (!configData.data) return [];
    return [...configData.data.available_claude_models, ...configData.data.available_openai_models];
  }

  function handleApiKeyChange(keyName: string, value: string) {
    editedApiKeys[keyName] = value;
  }

  function handleModelChange(modelName: string, value: string) {
    editedModels[modelName] = value;
  }

  function handleApiKeyModeChange(value: string) {
    editedApiKeyMode = value;
  }
</script>

<div class="settings">
  <header class="settings-header">
    <h2>Settings</h2>
    <div class="header-actions">
      <button class="refresh-btn" onclick={() => configData.fetch()} disabled={configData.loading || isEditing} title="Refresh">
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

  {#if configData.loading && !configData.data}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Loading configuration...</p>
    </div>
  {:else if configData.error}
    <div class="error-state">
      <p>Failed to load configuration: {configData.error}</p>
      <button onclick={() => configData.fetch()}>Retry</button>
    </div>
  {:else if configData.data}
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
          <span class="provider-name">{configData.data.provider}</span>
          <span class="provider-status">
            {configData.data.provider === "None" ? "No API key configured" : "Active"}
          </span>
        </div>
      </div>
      {#if configData.data.provider !== "None"}
        <div class="active-models">
          <span class="active-models-label">Active Models:</span>
          <div class="active-models-list">
            {#each configData.data.models.filter(m => m.name !== "CLAUDE_CODE_API_KEY_MODE" && (m.value || m.default_value)) as model}
              <span class="active-model-chip" class:is-default={model.is_default} title={model.name}>
                <span class="model-role">{model.name.replace(/_/g, " ").toLowerCase()}:</span>
                {model.value ?? model.default_value}
              </span>
            {/each}
          </div>
        </div>
      {/if}
    </section>

    <!-- API Keys -->
    <ApiKeysSection
      apiKeys={configData.data.api_keys}
      {isEditing}
      {editedApiKeys}
      {validationStatus}
      onStartEditing={startEditing}
      onApiKeyChange={handleApiKeyChange}
      onValidateApiKey={validateApiKey}
    />

    <!-- Model Configuration -->
    <ModelConfigSection
      models={configData.data.models}
      {isEditing}
      {editedModels}
      availableModels={getAvailableModels()}
      onModelChange={handleModelChange}
    />

    <!-- Advanced Settings -->
    <AdvancedSettingsSection
      {isEditing}
      {editedApiKeyMode}
      currentApiKeyMode={configData.data?.models.find(m => m.name === "CLAUDE_CODE_API_KEY_MODE")?.value ?? undefined}
      onApiKeyModeChange={handleApiKeyModeChange}
    />

    <!-- Edit Actions / Configuration Footer -->
    <EditActionsFooter
      {isEditing}
      {isSaving}
      {saveError}
      configPath={configData.data.config_path}
      onSave={saveChanges}
      onCancel={cancelEditing}
      onOpenConfigDirectory={openConfigDirectory}
    />
  {/if}
</div>

<style>
  .settings {
    padding: var(--space-6);
    height: 100%;
    overflow-y: auto;
  }

  .settings-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-6);
    padding-bottom: var(--space-4);
    border-bottom: 1px solid var(--border-hex);
  }

  .settings-header h2 {
    margin: 0;
    font-size: var(--text-2xl);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .refresh-btn,
  .close-btn {
    width: 36px;
    height: 36px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-hex);
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .refresh-btn:hover:not(:disabled),
  .close-btn:hover {
    background: var(--accent-glow);
    border-color: var(--accent-hex);
    color: var(--text-primary);
  }

  .refresh-btn:disabled {
    opacity: 0.4;
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
    gap: var(--space-4);
    padding: var(--space-4);
    background: var(--warning-glow);
    border: 1px solid var(--warning-hex);
    border-radius: var(--radius-md);
    color: var(--warning-hex);
    font-size: var(--text-sm);
    margin-bottom: var(--space-6);
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
    padding: var(--space-1) var(--space-3);
    background: transparent;
    border: 1px solid var(--warning-hex);
    border-radius: var(--radius-sm);
    color: var(--warning-hex);
    cursor: pointer;
    font-size: var(--text-xs);
    transition: background var(--transition-fast);
  }

  .dismiss-btn:hover {
    background: rgba(255, 149, 0, 0.2);
  }

  .config-section {
    background: var(--bg-secondary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-lg);
    padding: var(--space-6);
    margin-bottom: var(--space-6);
  }

  .config-section h3 {
    margin: 0 0 var(--space-4) 0;
    font-size: var(--text-lg);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
  }

  .provider-card {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-4);
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
  }

  .provider-icon {
    width: 48px;
    height: 48px;
    border-radius: var(--radius-lg);
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--accent-hex);
  }

  .provider-icon svg {
    width: 24px;
    height: 24px;
  }

  .provider-info {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .provider-name {
    font-size: var(--text-lg);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
  }

  .provider-status {
    font-size: var(--text-sm);
    color: var(--text-muted);
  }

  /* Active Models in Provider Card */
  .active-models {
    margin-top: var(--space-4);
    padding-top: var(--space-4);
    border-top: 1px solid var(--border-hex);
  }

  .active-models-label {
    font-size: var(--text-xs);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    display: block;
    margin-bottom: var(--space-2);
  }

  .active-models-list {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .active-model-chip {
    font-size: var(--text-xs);
    padding: var(--space-1) var(--space-3);
    background: var(--bg-tertiary);
    color: var(--text-primary);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-hex);
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

  .active-model-chip.is-default {
    opacity: 0.8;
  }

  .loading-state, .error-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-8);
    gap: var(--space-4);
    color: var(--text-secondary);
  }

  .error-state button {
    padding: var(--space-2) var(--space-4);
    background: var(--accent-hex);
    color: white;
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .error-state button:hover {
    background: var(--accent-hover);
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid var(--border-hex);
    border-top-color: var(--accent-hex);
    border-radius: var(--radius-full);
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
