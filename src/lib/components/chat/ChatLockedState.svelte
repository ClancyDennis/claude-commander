<script lang="ts">
  import { open } from "@tauri-apps/plugin-shell";

  interface Props {
    configPath: string;
    onOpenConfigDir: () => void;
  }

  let { configPath, onOpenConfigDir }: Props = $props();

  function openAnthropicKeys() {
    open("https://console.anthropic.com/settings/keys");
  }

  function openOpenAIKeys() {
    open("https://platform.openai.com/api-keys");
  }
</script>

<div class="locked-state">
  <div class="locked-icon">🔒</div>
  <div class="locked-title">API Key Required</div>
  <div class="locked-description">
    Enter an OpenAI or Anthropic API key to unlock the System Commander.
  </div>
  <div class="locked-hint">
    <button class="config-link" onclick={onOpenConfigDir} title="Click to open folder">
      <code>{configPath || "~/.config/claude-commander/.env"}</code>
      <span class="open-icon">↗</span>
    </button>
    <span>Add OPENAI_API_KEY or ANTHROPIC_API_KEY</span>
  </div>
  <div class="provider-links">
    <button class="provider-link" onclick={openAnthropicKeys}>
      Get Anthropic Key
      <span class="open-icon">↗</span>
    </button>
    <button class="provider-link" onclick={openOpenAIKeys}>
      Get OpenAI Key
      <span class="open-icon">↗</span>
    </button>
  </div>
</div>

<style>
  .locked-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: var(--space-10);
    color: var(--text-muted);
  }

  .locked-icon {
    font-size: 56px;
    margin-bottom: var(--space-4);
    opacity: 0.7;
  }

  .locked-title {
    font-size: var(--text-2xl);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
    margin-bottom: var(--space-2);
  }

  .locked-description {
    font-size: var(--text-sm);
    margin-bottom: var(--space-6);
    max-width: 360px;
    line-height: var(--leading-relaxed);
  }

  .locked-hint {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
  }

  .locked-hint .config-link {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .locked-hint .config-link:hover {
    transform: translateY(-1px);
  }

  .locked-hint .config-link:hover code {
    background: rgba(232, 102, 77, 0.15);
    border-color: var(--accent-hex);
  }

  .locked-hint code {
    font-family: var(--font-mono, monospace);
    font-size: var(--text-sm);
    color: var(--accent-hex);
    background: var(--bg-elevated);
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-hex);
    transition: all var(--transition-fast);
  }

  .locked-hint .open-icon {
    font-size: var(--text-sm);
    color: var(--accent-hex);
    opacity: 0.7;
  }

  .locked-hint .config-link:hover .open-icon {
    opacity: 1;
  }

  .locked-hint > span {
    font-size: var(--text-xs);
    color: var(--text-muted);
  }

  .provider-links {
    display: flex;
    gap: var(--space-3);
    margin-top: var(--space-2);
    flex-wrap: wrap;
    justify-content: center;
  }

  .provider-link {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--text-secondary);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .provider-link:hover {
    color: var(--accent-hex);
    border-color: var(--accent-hex);
    background: rgba(232, 102, 77, 0.1);
  }

  .provider-link .open-icon {
    font-size: var(--text-xs);
    opacity: 0.6;
  }

  .provider-link:hover .open-icon {
    opacity: 1;
  }
</style>
