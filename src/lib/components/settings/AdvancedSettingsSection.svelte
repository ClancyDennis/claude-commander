<script lang="ts">
  import HelpTip from "../new-agent/HelpTip.svelte";

  let {
    isEditing,
    editedApiKeyMode,
    currentApiKeyMode,
    onApiKeyModeChange,
  }: {
    isEditing: boolean;
    editedApiKeyMode: string;
    currentApiKeyMode: string | undefined;
    onApiKeyModeChange: (value: string) => void;
  } = $props();
</script>

<section class="config-section">
  <h3>Advanced Settings</h3>
  <div class="advanced-setting-item">
    <div class="advanced-setting-header">
      <span class="advanced-setting-name">Claude Code Authentication</span>
      <HelpTip text="How Claude Code worker agents authenticate with Anthropic." placement="top" />
    </div>
    <p class="advanced-setting-description">
      Choose how worker agents authenticate. "Claude Code Configured" uses OAuth credentials
      from ~/.claude/.credentials.json. "API Key" passes your Anthropic API key to workers.
    </p>
    {#if isEditing}
      <div class="auth-options">
        <label class="auth-option" class:selected={editedApiKeyMode !== "passthrough"}>
          <input
            type="radio"
            name="auth-mode"
            value="blocked"
            checked={editedApiKeyMode !== "passthrough"}
            onchange={() => onApiKeyModeChange("blocked")}
          />
          <span class="option-label">Claude Code Configured</span>
          <span class="option-hint">(OAuth, default)</span>
        </label>
        <label class="auth-option" class:selected={editedApiKeyMode === "passthrough"}>
          <input
            type="radio"
            name="auth-mode"
            value="passthrough"
            checked={editedApiKeyMode === "passthrough"}
            onchange={() => onApiKeyModeChange("passthrough")}
          />
          <span class="option-label">API Key</span>
          <span class="option-hint">(uses your Anthropic key)</span>
        </label>
      </div>
    {:else}
      <div class="advanced-setting-value">
        <span class="status-badge" class:oauth={currentApiKeyMode !== "passthrough"}>
          {currentApiKeyMode === "passthrough" ? "API Key" : "Claude Code Configured"}
        </span>
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

  .config-section h3 {
    margin: 0 0 var(--space-4) 0;
    font-size: var(--text-lg);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
  }

  .advanced-setting-item {
    padding: var(--space-4);
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-hex);
  }

  .advanced-setting-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-1);
  }

  .advanced-setting-name {
    font-weight: var(--font-semibold);
    color: var(--text-primary);
  }

  .advanced-setting-description {
    font-size: var(--text-sm);
    color: var(--text-muted);
    margin: 0 0 var(--space-4) 0;
    line-height: var(--leading-relaxed);
  }

  .advanced-setting-value {
    display: flex;
    align-items: center;
  }

  .status-badge {
    font-size: var(--text-xs);
    padding: var(--space-1) var(--space-3);
    background: var(--bg-primary);
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-hex);
  }

  .status-badge.oauth {
    background: var(--success-glow);
    color: var(--success-hex);
    border-color: rgba(52, 199, 89, 0.3);
  }

  /* Radio Button Options */
  .auth-options {
    display: flex;
    gap: var(--space-3);
    flex-wrap: wrap;
  }

  .auth-option {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    background: var(--bg-primary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .auth-option:hover {
    border-color: var(--text-muted);
  }

  .auth-option.selected {
    border-color: var(--accent-hex);
    background: var(--accent-glow);
  }

  .auth-option input[type="radio"] {
    accent-color: var(--accent-hex);
    width: 16px;
    height: 16px;
    margin: 0;
  }

  .option-label {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--text-primary);
  }

  .option-hint {
    font-size: var(--text-xs);
    color: var(--text-muted);
  }
</style>
