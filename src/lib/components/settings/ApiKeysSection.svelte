<script lang="ts">
  import type { ApiKeyStatus } from "$lib/types";
  import ApiKeyItem from "./ApiKeyItem.svelte";

  let {
    apiKeys,
    isEditing,
    editedApiKeys,
    validationStatus,
    onStartEditing,
    onApiKeyChange,
    onValidateApiKey,
  }: {
    apiKeys: ApiKeyStatus[];
    isEditing: boolean;
    editedApiKeys: Record<string, string>;
    validationStatus: Record<string, { valid: boolean | null; message: string; loading: boolean }>;
    onStartEditing: () => void;
    onApiKeyChange: (keyName: string, value: string) => void;
    onValidateApiKey: (provider: string) => void;
  } = $props();

  function getProviderKeyName(provider: string): string {
    const mapping: Record<string, string> = {
      "Anthropic": "ANTHROPIC_API_KEY",
      "OpenAI": "OPENAI_API_KEY",
      "Gemini": "GEMINI_API_KEY",
      "GitHub": "GITHUB_TOKEN",
    };
    return mapping[provider] || "";
  }
</script>

<section class="config-section">
  <div class="section-header">
    <h3>API Keys</h3>
    {#if !isEditing}
      <button class="edit-btn" onclick={onStartEditing}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7"/>
          <path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z"/>
        </svg>
        Edit
      </button>
    {/if}
  </div>
  <div class="api-keys-list">
    {#each apiKeys as apiKey}
      {@const keyName = getProviderKeyName(apiKey.provider)}
      <ApiKeyItem
        {apiKey}
        {keyName}
        {isEditing}
        editedValue={editedApiKeys[keyName] || ""}
        validationStatus={validationStatus[keyName]}
        onInputChange={(value) => onApiKeyChange(keyName, value)}
        onValidate={() => onValidateApiKey(apiKey.provider)}
      />
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
            value={editedApiKeys.GITHUB_TOKEN || ""}
            oninput={(e) => onApiKeyChange("GITHUB_TOKEN", e.currentTarget.value)}
            placeholder="Enter GitHub token (optional)"
            class="api-key-input"
          />
          <button
            class="validate-btn"
            onclick={() => onValidateApiKey("GitHub")}
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

<style>
  .config-section {
    background: var(--bg-secondary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-lg);
    padding: var(--space-6);
    margin-bottom: var(--space-6);
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-4);
  }

  .section-header h3 {
    margin: 0;
    font-size: var(--text-lg);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
  }

  .edit-btn {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    cursor: pointer;
    font-size: var(--text-sm);
    transition: all var(--transition-fast);
  }

  .edit-btn:hover {
    background: var(--accent-glow);
    border-color: var(--accent-hex);
    color: var(--text-primary);
  }

  .edit-btn svg {
    width: 14px;
    height: 14px;
  }

  .api-keys-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  /* Styles for inline GitHub token item */
  .api-key-item {
    padding: var(--space-4);
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-hex);
  }

  .api-key-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-2);
  }

  .api-key-provider {
    font-weight: var(--font-semibold);
    color: var(--text-primary);
  }

  .api-key-edit {
    display: flex;
    gap: var(--space-2);
  }

  .api-key-input {
    flex: 1;
    padding: var(--space-2) var(--space-3);
    background: var(--bg-primary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-family: monospace;
    font-size: var(--text-sm);
  }

  .api-key-input:focus {
    outline: none;
    border-color: var(--accent-hex);
    box-shadow: 0 0 0 3px var(--accent-glow);
  }

  .api-key-input::placeholder {
    color: var(--text-muted);
    font-family: inherit;
  }

  .validate-btn {
    padding: var(--space-2) var(--space-4);
    background: var(--bg-secondary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    cursor: pointer;
    font-size: var(--text-sm);
    transition: all var(--transition-fast);
  }

  .validate-btn:hover:not(:disabled) {
    background: var(--accent-glow);
    border-color: var(--accent-hex);
    color: var(--text-primary);
  }

  .validate-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .validation-result {
    margin-top: var(--space-2);
    font-size: var(--text-xs);
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-sm);
  }

  .validation-result.valid {
    color: var(--success-hex);
    background: var(--success-glow);
  }

  .validation-result.invalid {
    color: var(--error);
    background: var(--error-glow);
  }
</style>
