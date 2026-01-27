<script lang="ts">
  import type { ApiKeyStatus } from "$lib/types";

  let {
    apiKey,
    keyName,
    editedValue,
    isEditing,
    validationStatus,
    onInputChange,
    onValidate,
  }: {
    apiKey: ApiKeyStatus;
    keyName: string;
    editedValue: string;
    isEditing: boolean;
    validationStatus: { valid: boolean | null; message: string; loading: boolean } | undefined;
    onInputChange: (value: string) => void;
    onValidate: () => void;
  } = $props();

  function getStatusColor(isConfigured: boolean): string {
    return isConfigured ? "var(--success)" : "var(--warning)";
  }
</script>

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
        value={editedValue}
        oninput={(e) => onInputChange(e.currentTarget.value)}
        placeholder={apiKey.is_configured ? "Leave empty to keep current" : "Enter API key"}
        class="api-key-input"
      />
      <button
        class="validate-btn"
        onclick={onValidate}
        disabled={validationStatus?.loading}
      >
        {validationStatus?.loading ? "..." : "Test"}
      </button>
    </div>
    {#if validationStatus?.message}
      <div class="validation-result" class:valid={validationStatus?.valid} class:invalid={validationStatus?.valid === false}>
        {validationStatus?.message}
      </div>
    {/if}
  {:else}
    <div class="api-key-preview">
      <code>{apiKey.key_preview}</code>
    </div>
  {/if}
</div>

<style>
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

  .api-key-status {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
  }

  .api-key-preview code {
    font-family: monospace;
    font-size: var(--text-sm);
    color: var(--text-muted);
    background: var(--bg-primary);
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-sm);
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
