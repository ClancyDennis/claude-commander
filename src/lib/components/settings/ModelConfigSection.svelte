<script lang="ts">
  import type { ModelConfig, ApiKeyStatus } from "$lib/types";
  import HelpTip from "../new-agent/HelpTip.svelte";

  let {
    models,
    apiKeys,
    isEditing,
    editedModels,
    availableClaudeModels,
    claudeModelAliases,
    claudeCodeModelOptions,
    availableOpenaiModels,
    onModelChange,
  }: {
    models: ModelConfig[];
    apiKeys: ApiKeyStatus[];
    isEditing: boolean;
    editedModels: Record<string, string>;
    availableClaudeModels: string[];
    claudeModelAliases: string[];
    claudeCodeModelOptions: string[];
    availableOpenaiModels: string[];
    onModelChange: (modelName: string, value: string) => void;
  } = $props();

  // Check which providers are configured
  const anthropicConfigured = $derived(apiKeys.find(k => k.provider === "Anthropic")?.is_configured ?? false);
  const openaiConfigured = $derived(apiKeys.find(k => k.provider === "OpenAI")?.is_configured ?? false);

  // User-friendly labels for settings
  const labels: Record<string, string> = {
    PRIMARY_MODEL: "Model",
    SECURITY_MODEL: "Security Model",
    LIGHT_TASK_MODEL: "Light Model",
    ADVISOR_MODEL: "Advisor Model",
    ADVISOR_ENABLED: "Advisor",
    CLAUDE_CODE_MODEL: "Model",
  };

  // Descriptions for settings
  const descriptions: Record<string, string> = {
    PRIMARY_MODEL: "Main model for conversations and complex reasoning. Provider is inferred from model name.",
    SECURITY_MODEL: "Model for security-critical analysis (higher capability recommended)",
    LIGHT_TASK_MODEL: "Model for lightweight tasks like title generation (faster/cheaper)",
    ADVISOR_MODEL: "External AI advisor model that reviews each pipeline step (default: Gemini)",
    ADVISOR_ENABLED: "Enable the external advisor to provide guidance after each pipeline phase",
    CLAUDE_CODE_MODEL: "Model used by Claude Code worker agents",
  };

  // Claude Code model labels
  const claudeCodeLabels: Record<string, string> = {
    auto: "Auto (Latest Claude)",
    sonnet: "Sonnet (Recommended)",
    opus: "Opus (Premium)",
    haiku: "Haiku (Fast)",
  };

  function getModel(name: string): ModelConfig | undefined {
    return models.find(m => m.name === name);
  }

  function getDisplayValue(model: ModelConfig | undefined): string {
    if (!model) return "—";
    return model.value ?? model.default_value ?? "—";
  }

  function isClaudeModel(model: string): boolean {
    if (!model) return false;
    const lower = model.toLowerCase();
    return lower.startsWith("claude-") || claudeModelAliases.includes(model) || ["sonnet", "opus", "haiku"].includes(lower);
  }

  function isOpenAIModel(model: string): boolean {
    if (!model) return false;
    const lower = model.toLowerCase();
    return lower.startsWith("gpt-") || lower.startsWith("o1-") || lower.startsWith("o3-");
  }

  function isGeminiModel(model: string): boolean {
    if (!model) return false;
    return model.toLowerCase().startsWith("gemini-");
  }

  // Get provider badge for display
  function getProviderBadge(modelValue: string | null | undefined): string {
    if (!modelValue) return "";
    if (isOpenAIModel(modelValue)) return "OpenAI";
    if (isClaudeModel(modelValue)) return "Claude";
    if (isGeminiModel(modelValue)) return "Gemini";
    return "";
  }

  // Known Gemini model options
  const geminiModels = [
    "gemini-2.5-pro-preview-06-05",
    "gemini-2.5-flash-preview-05-20",
    "gemini-2.0-flash",
  ];

  // Check if Gemini is configured
  const geminiConfigured = $derived(apiKeys.find(k => k.provider === "Gemini")?.is_configured ?? false);
</script>

<!-- Meta Agent Section -->
<section class="config-section">
  <div class="section-header">
    <h3>Meta Agent</h3>
    <span class="section-subtitle">System Commander that orchestrates worker agents</span>
  </div>

  <div class="settings-grid">
    <!-- Primary Model (unified dropdown) -->
    <div class="setting-item">
      <div class="setting-header">
        <span class="setting-label">{labels.PRIMARY_MODEL}</span>
        <HelpTip text={descriptions.PRIMARY_MODEL} placement="top" />
      </div>
      {#if isEditing}
        <select
          value={editedModels.PRIMARY_MODEL}
          onchange={(e) => onModelChange("PRIMARY_MODEL", e.currentTarget.value)}
          class="setting-select"
        >
          <option value="">Use default</option>
          {#if anthropicConfigured}
            <optgroup label="Claude (Latest)">
              {#each claudeModelAliases as model}
                <option value={model}>{model}</option>
              {/each}
            </optgroup>
            <optgroup label="Claude (Pinned)">
              {#each availableClaudeModels.filter(m => !claudeModelAliases.includes(m)) as model}
                <option value={model}>{model}</option>
              {/each}
            </optgroup>
          {/if}
          {#if openaiConfigured}
            <optgroup label="OpenAI">
              {#each availableOpenaiModels as model}
                <option value={model}>{model}</option>
              {/each}
            </optgroup>
          {/if}
        </select>
      {:else}
        {@const model = getModel("PRIMARY_MODEL")}
        {@const badge = getProviderBadge(model?.value || model?.default_value)}
        <div class="setting-value">
          <code>{getDisplayValue(model)}</code>
          {#if badge}
            <span class="provider-badge" class:openai={badge === "OpenAI"} class:claude={badge === "Claude"}>{badge}</span>
          {/if}
          {#if model?.is_default}
            <span class="default-indicator">(default)</span>
          {/if}
        </div>
      {/if}
    </div>

    <!-- Security Model -->
    <div class="setting-item">
      <div class="setting-header">
        <span class="setting-label">{labels.SECURITY_MODEL}</span>
        <HelpTip text={descriptions.SECURITY_MODEL} placement="top" />
      </div>
      {#if isEditing}
        <select
          value={editedModels.SECURITY_MODEL}
          onchange={(e) => onModelChange("SECURITY_MODEL", e.currentTarget.value)}
          class="setting-select"
        >
          <option value="">Use default</option>
          {#if anthropicConfigured}
            <optgroup label="Claude (Latest)">
              {#each claudeModelAliases as model}
                <option value={model}>{model}</option>
              {/each}
            </optgroup>
            <optgroup label="Claude (Pinned)">
              {#each availableClaudeModels.filter(m => !claudeModelAliases.includes(m)) as model}
                <option value={model}>{model}</option>
              {/each}
            </optgroup>
          {/if}
          {#if openaiConfigured}
            <optgroup label="OpenAI">
              {#each availableOpenaiModels as model}
                <option value={model}>{model}</option>
              {/each}
            </optgroup>
          {/if}
        </select>
      {:else}
        {@const model = getModel("SECURITY_MODEL")}
        {@const badge = getProviderBadge(model?.value || model?.default_value)}
        <div class="setting-value">
          <code>{getDisplayValue(model)}</code>
          {#if badge}
            <span class="provider-badge" class:openai={badge === "OpenAI"} class:claude={badge === "Claude"}>{badge}</span>
          {/if}
          {#if model?.is_default}
            <span class="default-indicator">(default)</span>
          {/if}
        </div>
      {/if}
    </div>

    <!-- Light Task Model -->
    <div class="setting-item">
      <div class="setting-header">
        <span class="setting-label">{labels.LIGHT_TASK_MODEL}</span>
        <HelpTip text={descriptions.LIGHT_TASK_MODEL} placement="top" />
      </div>
      {#if isEditing}
        <select
          value={editedModels.LIGHT_TASK_MODEL}
          onchange={(e) => onModelChange("LIGHT_TASK_MODEL", e.currentTarget.value)}
          class="setting-select"
        >
          <option value="">Use default</option>
          {#if anthropicConfigured}
            <optgroup label="Claude (Latest)">
              {#each claudeModelAliases as model}
                <option value={model}>{model}</option>
              {/each}
            </optgroup>
            <optgroup label="Claude (Pinned)">
              {#each availableClaudeModels.filter(m => !claudeModelAliases.includes(m)) as model}
                <option value={model}>{model}</option>
              {/each}
            </optgroup>
          {/if}
          {#if openaiConfigured}
            <optgroup label="OpenAI">
              {#each availableOpenaiModels as model}
                <option value={model}>{model}</option>
              {/each}
            </optgroup>
          {/if}
        </select>
      {:else}
        {@const model = getModel("LIGHT_TASK_MODEL")}
        {@const badge = getProviderBadge(model?.value || model?.default_value)}
        <div class="setting-value">
          <code>{getDisplayValue(model)}</code>
          {#if badge}
            <span class="provider-badge" class:openai={badge === "OpenAI"} class:claude={badge === "Claude"}>{badge}</span>
          {/if}
          {#if model?.is_default}
            <span class="default-indicator">(default)</span>
          {/if}
        </div>
      {/if}
    </div>
  </div>
</section>

<!-- Advisor Section -->
<section class="config-section">
  <div class="section-header">
    <h3>Advisor</h3>
    <span class="section-subtitle">External AI that reviews each pipeline step and provides guidance</span>
  </div>

  <div class="settings-grid">
    <!-- Advisor Enabled Toggle -->
    <div class="setting-item">
      <div class="setting-header">
        <span class="setting-label">{labels.ADVISOR_ENABLED}</span>
        <HelpTip text={descriptions.ADVISOR_ENABLED} placement="top" />
      </div>
      {#if isEditing}
        <select
          value={editedModels.ADVISOR_ENABLED || "false"}
          onchange={(e) => onModelChange("ADVISOR_ENABLED", e.currentTarget.value)}
          class="setting-select"
        >
          <option value="false">Disabled</option>
          <option value="true">Enabled</option>
        </select>
      {:else}
        {@const model = getModel("ADVISOR_ENABLED")}
        <div class="setting-value">
          <code>{(model?.value || model?.default_value) === "true" ? "Enabled" : "Disabled"}</code>
          {#if model?.is_default}
            <span class="default-indicator">(default)</span>
          {/if}
        </div>
      {/if}
    </div>

    <!-- Advisor Model -->
    <div class="setting-item">
      <div class="setting-header">
        <span class="setting-label">{labels.ADVISOR_MODEL}</span>
        <HelpTip text={descriptions.ADVISOR_MODEL} placement="top" />
      </div>
      {#if isEditing}
        <select
          value={editedModels.ADVISOR_MODEL}
          onchange={(e) => onModelChange("ADVISOR_MODEL", e.currentTarget.value)}
          class="setting-select"
        >
          <option value="">Use default (gemini-2.5-pro-preview-06-05)</option>
          {#if geminiConfigured}
            <optgroup label="Gemini">
              {#each geminiModels as model}
                <option value={model}>{model}</option>
              {/each}
            </optgroup>
          {/if}
          {#if anthropicConfigured}
            <optgroup label="Claude">
              {#each claudeModelAliases as model}
                <option value={model}>{model}</option>
              {/each}
            </optgroup>
          {/if}
          {#if openaiConfigured}
            <optgroup label="OpenAI">
              {#each availableOpenaiModels as model}
                <option value={model}>{model}</option>
              {/each}
            </optgroup>
          {/if}
        </select>
      {:else}
        {@const model = getModel("ADVISOR_MODEL")}
        {@const badge = getProviderBadge(model?.value || model?.default_value)}
        <div class="setting-value">
          <code>{getDisplayValue(model)}</code>
          {#if badge}
            <span class="provider-badge" class:openai={badge === "OpenAI"} class:claude={badge === "Claude"} class:gemini={badge === "Gemini"}>{badge}</span>
          {/if}
          {#if model?.is_default}
            <span class="default-indicator">(default)</span>
          {/if}
        </div>
      {/if}
    </div>
  </div>
</section>

<!-- Claude Code (Worker Agents) Section -->
<section class="config-section">
  <div class="section-header">
    <h3>Claude Code</h3>
    <span class="section-subtitle">Worker agents that execute tasks</span>
  </div>

  <div class="settings-grid">
    <!-- Claude Code Model -->
    <div class="setting-item">
      <div class="setting-header">
        <span class="setting-label">{labels.CLAUDE_CODE_MODEL}</span>
        <HelpTip text={descriptions.CLAUDE_CODE_MODEL} placement="top" />
      </div>
      {#if isEditing}
        <select
          value={editedModels.CLAUDE_CODE_MODEL}
          onchange={(e) => onModelChange("CLAUDE_CODE_MODEL", e.currentTarget.value)}
          class="setting-select"
        >
          <option value="">Use default (auto)</option>
          {#each claudeCodeModelOptions as option}
            <option value={option}>{claudeCodeLabels[option] || option}</option>
          {/each}
        </select>
      {:else}
        {@const model = getModel("CLAUDE_CODE_MODEL")}
        <div class="setting-value">
          <code>{claudeCodeLabels[model?.value || "auto"] || getDisplayValue(model)}</code>
          {#if model?.is_default}
            <span class="default-indicator">(default)</span>
          {/if}
        </div>
      {/if}
    </div>

    <!-- Auth Status (read-only info) -->
    <div class="setting-item info-only">
      <div class="setting-header">
        <span class="setting-label">Authentication</span>
        <HelpTip text="Claude Code uses OAuth authentication via ~/.claude/.credentials.json" placement="top" />
      </div>
      <div class="setting-value">
        <code>OAuth</code>
        <span class="auth-path">~/.claude/.credentials.json</span>
      </div>
    </div>
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
    margin-bottom: var(--space-4);
  }

  .section-header h3 {
    margin: 0;
    font-size: var(--text-lg);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
  }

  .section-subtitle {
    font-size: var(--text-sm);
    color: var(--text-muted);
    display: block;
    margin-top: var(--space-1);
  }

  .settings-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: var(--space-4);
  }

  .setting-item {
    padding: var(--space-4);
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-hex);
  }

  .setting-item.info-only {
    background: var(--bg-secondary);
    border-style: dashed;
  }

  .setting-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-2);
  }

  .setting-label {
    font-weight: var(--font-semibold);
    color: var(--text-primary);
    font-size: var(--text-sm);
  }

  .setting-value {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-wrap: wrap;
  }

  .setting-value code {
    font-family: monospace;
    font-size: var(--text-sm);
    color: var(--text-secondary);
    background: var(--bg-primary);
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-sm);
  }

  .provider-badge {
    font-size: var(--text-xs);
    padding: 2px var(--space-2);
    border-radius: var(--radius-sm);
    font-weight: var(--font-medium);
  }

  .provider-badge.claude {
    background: rgba(232, 102, 77, 0.2);
    color: var(--accent-hex);
  }

  .provider-badge.openai {
    background: rgba(16, 163, 127, 0.2);
    color: rgb(16, 163, 127);
  }

  .provider-badge.gemini {
    background: rgba(66, 133, 244, 0.2);
    color: rgb(66, 133, 244);
  }

  .setting-select {
    width: 100%;
    padding: var(--space-2) var(--space-3);
    background: var(--bg-primary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: var(--text-sm);
    cursor: pointer;
  }

  .setting-select:focus {
    outline: none;
    border-color: var(--accent-hex);
    box-shadow: 0 0 0 3px var(--accent-glow);
  }

  .setting-select optgroup {
    font-weight: var(--font-semibold);
    color: var(--text-muted);
  }

  .default-indicator {
    font-size: var(--text-xs);
    color: var(--text-muted);
    font-style: italic;
  }

  .auth-path {
    font-size: var(--text-xs);
    color: var(--text-muted);
    font-family: monospace;
  }
</style>
