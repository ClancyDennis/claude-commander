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
      <span class="advanced-setting-name">Claude Code Local Authentication</span>
      <HelpTip text="Allows Claude Code agent instances to use their own authentication (OAuth login or separate API key) instead of requiring the Anthropic API key configured above." placement="top" />
    </div>
    <p class="advanced-setting-description">
      Allow Claude Code agent instances to use their own authentication (OAuth or separate API key) instead of the Anthropic API key configured here. The meta-agent still uses the API key above, but spawned Claude Code agents can authenticate independently.
    </p>
    {#if isEditing}
      <div class="toggle-wrapper">
        <label class="toggle">
          <input
            type="checkbox"
            checked={editedApiKeyMode === "passthrough"}
            onchange={(e) => onApiKeyModeChange(e.currentTarget.checked ? "passthrough" : "blocked")}
          />
          <span class="toggle-slider"></span>
        </label>
        <span class="toggle-label">{editedApiKeyMode === "passthrough" ? "Enabled" : "Disabled"}</span>
      </div>
    {:else}
      <div class="advanced-setting-value">
        <span class="status-badge" class:enabled={currentApiKeyMode === "passthrough"}>
          {currentApiKeyMode === "passthrough" ? "Enabled" : "Disabled"}
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

  .status-badge.enabled {
    background: var(--success-glow);
    color: var(--success-hex);
    border-color: rgba(52, 199, 89, 0.3);
  }

  /* Toggle Switch */
  .toggle-wrapper {
    display: flex;
    align-items: center;
    gap: var(--space-2);
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
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-full);
    transition: all var(--transition-fast);
  }

  .toggle-slider:before {
    position: absolute;
    content: "";
    height: 18px;
    width: 18px;
    left: 2px;
    bottom: 2px;
    background-color: var(--text-muted);
    border-radius: var(--radius-full);
    transition: all var(--transition-fast);
  }

  .toggle input:checked + .toggle-slider {
    background-color: var(--accent-hex);
    border-color: var(--accent-hex);
  }

  .toggle input:checked + .toggle-slider:before {
    transform: translateX(20px);
    background-color: white;
  }

  .toggle-label {
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }
</style>
