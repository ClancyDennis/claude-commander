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
    padding: 40px;
    color: #999;
  }

  .locked-icon {
    font-size: 64px;
    margin-bottom: 16px;
    opacity: 0.6;
  }

  .locked-title {
    font-size: 24px;
    font-weight: 600;
    color: #e0e0e0;
    margin-bottom: 8px;
  }

  .locked-description {
    font-size: 14px;
    margin-bottom: 24px;
    max-width: 400px;
    color: #999;
  }

  .locked-hint {
    display: flex;
    flex-direction: column;
    gap: 8px;
    background: rgba(124, 58, 237, 0.05);
    border: 1px solid rgba(124, 58, 237, 0.2);
    border-radius: 12px;
    padding: 16px;
  }

  .locked-hint .config-link {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .locked-hint .config-link:hover {
    transform: translateY(-1px);
  }

  .locked-hint .config-link:hover code {
    background: rgba(124, 58, 237, 0.2);
    border-color: rgba(124, 58, 237, 0.5);
  }

  .locked-hint code {
    font-family: monospace;
    font-size: 13px;
    color: #7c3aed;
    background: rgba(124, 58, 237, 0.1);
    padding: 4px 8px;
    border-radius: 4px;
    border: 1px solid rgba(124, 58, 237, 0.2);
    transition: all 0.2s ease;
  }

  .locked-hint .open-icon {
    font-size: 14px;
    color: #7c3aed;
    opacity: 0.7;
  }

  .locked-hint .config-link:hover .open-icon {
    opacity: 1;
  }

  .locked-hint > span {
    font-size: 12px;
    color: #666;
  }

  .provider-links {
    display: flex;
    gap: 12px;
    margin-top: 8px;
  }

  .provider-link {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 8px 14px;
    font-size: 13px;
    font-weight: 500;
    color: var(--text-secondary, #999);
    background: var(--bg-tertiary, #1a1a1f);
    border: 1px solid var(--border, rgba(124, 58, 237, 0.2));
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .provider-link:hover {
    color: #7c3aed;
    border-color: rgba(124, 58, 237, 0.4);
    background: rgba(124, 58, 237, 0.1);
    transform: translateY(-1px);
  }

  .provider-link .open-icon {
    font-size: 12px;
    opacity: 0.6;
  }

  .provider-link:hover .open-icon {
    opacity: 1;
  }
</style>
