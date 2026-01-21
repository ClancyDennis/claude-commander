<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import logoIcon from "$lib/assets/claude-commander-icon.png";

  interface Props {
    show: boolean;
    onClose: () => void;
  }

  let { show, onClose }: Props = $props();

  async function handleOpenConfigFolder() {
    try {
      await invoke("open_config_directory");
    } catch (e) {
      console.error("Failed to open config directory:", e);
    }
  }

  async function handleSkip() {
    try {
      await invoke("create_env_placeholder");
      onClose();
    } catch (e) {
      console.error("Failed to create placeholder:", e);
      onClose();
    }
  }

  const features = [
    {
      icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
        <circle cx="9" cy="7" r="4"/>
        <path d="M23 21v-2a4 4 0 0 0-3-3.87"/>
        <path d="M16 3.13a4 4 0 0 1 0 7.75"/>
      </svg>`,
      title: "Multi-Agent Orchestration",
      description: "Spawn and manage multiple Claude Code agents working in parallel"
    },
    {
      icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/>
      </svg>`,
      title: "Pipeline Workflows",
      description: "Automated development with planning, building, and verification stages"
    },
    {
      icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
      </svg>`,
      title: "Security Monitoring",
      description: "Real-time threat detection and command analysis for agent actions"
    },
    {
      icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="12" y1="1" x2="12" y2="23"/>
        <path d="M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"/>
      </svg>`,
      title: "Cost Tracking",
      description: "Monitor API usage and costs across all agents in real-time"
    },
    {
      icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
        <polyline points="14 2 14 8 20 8"/>
        <line x1="16" y1="13" x2="8" y2="13"/>
        <line x1="16" y1="17" x2="8" y2="17"/>
        <polyline points="10 9 9 9 8 9"/>
      </svg>`,
      title: "Instruction Library",
      description: "Reusable skill templates for common development tasks"
    }
  ];
</script>

{#if show}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="overlay animate-fade-in" onclick={(e) => e.target === e.currentTarget && handleSkip()}>
    <div class="dialog animate-slide-up" role="dialog" aria-modal="true" tabindex="-1">
      <div class="header">
        <div class="icon-container">
          <img src={logoIcon} alt="Claude Commander" class="logo-icon" />
        </div>
        <div class="header-text">
          <h2>Welcome to Claude Commander</h2>
          <p class="subtitle">Orchestrate AI agents for powerful development workflows</p>
        </div>
      </div>

      <div class="content-row">
        <div class="features-grid">
          {#each features as feature}
            <div class="feature-card">
              <div class="feature-icon">
                {@html feature.icon}
              </div>
              <div class="feature-content">
                <h3>{feature.title}</h3>
                <p>{feature.description}</p>
              </div>
            </div>
          {/each}
        </div>

        <div class="setup-section">
          <h3>Authentication Required</h3>
          <div class="auth-explanation">
            <div class="auth-item">
              <strong>Claude Code CLI</strong>
              <span>Sign in with your Claude.ai account (Pro/Team) or Anthropic API key for the agents</span>
            </div>
            <div class="auth-item">
              <strong>API Key</strong>
              <span>Anthropic or OpenAI key in <code>.env</code> for orchestration and chat</span>
            </div>
          </div>
          <h3>Setup Steps</h3>
          <ol class="setup-steps">
            <li>
              <span class="step-number">1</span>
              <span>Click <strong>Open Config Folder</strong> below</span>
            </li>
            <li>
              <span class="step-number">2</span>
              <span>Copy <code>env.example</code> to <code>.env</code></span>
            </li>
            <li>
              <span class="step-number">3</span>
              <span>Add your API key and restart the app</span>
            </li>
          </ol>

              <div class="actions">
            <button class="btn btn-primary" onclick={handleOpenConfigFolder}>
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
              </svg>
              Open Config Folder
            </button>
            <button class="btn btn-secondary" onclick={handleSkip}>
              Skip for now
            </button>
          </div>
        </div>
      </div>

      <div class="info-box">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"/>
          <line x1="12" y1="16" x2="12" y2="12"/>
          <line x1="12" y1="8" x2="12.01" y2="8"/>
        </svg>
        <span>This dialog won't appear again once you configure your API key.</span>
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.85);
    backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
    padding: var(--space-lg);
  }

  .dialog {
    width: 900px;
    max-width: 95%;
    max-height: 90vh;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    padding: var(--space-lg, 24px);
    background-color: var(--bg-secondary);
    border-radius: 24px;
    border: 1px solid var(--border);
    box-shadow: var(--shadow-lg), 0 0 80px rgba(99, 102, 241, 0.15);
  }

  .header {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    margin-bottom: var(--space-md);
  }

  .icon-container {
    flex-shrink: 0;
  }

  .logo-icon {
    width: 56px;
    height: 56px;
    border-radius: 12px;
  }

  .header-text {
    flex: 1;
  }

  h2 {
    font-size: 22px;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0 0 2px 0;
  }

  .subtitle {
    font-size: 14px;
    color: var(--text-muted);
    margin: 0;
  }

  .content-row {
    display: flex;
    gap: var(--space-lg);
    margin-bottom: var(--space-md);
  }

  .features-grid {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .feature-card {
    display: flex;
    gap: var(--space-sm);
    padding: var(--space-sm) var(--space-md);
    background: var(--bg-elevated);
    border-radius: 10px;
    border: 1px solid var(--border);
  }

  .feature-icon {
    flex-shrink: 0;
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 6px;
    background: var(--bg-tertiary);
  }

  .feature-icon :global(svg) {
    width: 16px;
    height: 16px;
    color: var(--primary, #6366f1);
  }

  .feature-content h3 {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0 0 2px 0;
  }

  .feature-content p {
    font-size: 11px;
    color: var(--text-muted);
    margin: 0;
    line-height: 1.3;
  }

  .setup-section {
    flex-shrink: 0;
    width: 320px;
    padding: var(--space-md);
    background: var(--bg-elevated);
    border-radius: 12px;
    border: 1px solid var(--border);
    display: flex;
    flex-direction: column;
  }

  .setup-section h3 {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0 0 var(--space-xs) 0;
  }

  .auth-explanation {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: var(--space-sm);
    padding-bottom: var(--space-sm);
    border-bottom: 1px solid var(--border);
  }

  .auth-item {
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 11px;
  }

  .auth-item strong {
    color: var(--text-primary);
    font-size: 12px;
  }

  .auth-item span {
    color: var(--text-muted);
    line-height: 1.3;
  }

  .auth-item code {
    background: var(--bg-tertiary);
    padding: 1px 4px;
    border-radius: 3px;
    font-size: 10px;
    color: var(--text-primary);
  }

  .setup-steps {
    list-style: none;
    padding: 0;
    margin: 0 0 var(--space-md) 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
    flex: 1;
  }

  .setup-steps li {
    display: flex;
    align-items: flex-start;
    gap: var(--space-xs);
    font-size: 12px;
    color: var(--text-secondary);
    line-height: 1.4;
  }

  .step-number {
    flex-shrink: 0;
    width: 18px;
    height: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    background: var(--primary, #6366f1);
    color: white;
    font-size: 10px;
    font-weight: 600;
  }

  .setup-steps code {
    background: var(--bg-tertiary);
    padding: 1px 4px;
    border-radius: 3px;
    font-size: 11px;
    color: var(--text-primary);
  }

  .setup-steps strong {
    color: var(--text-primary);
  }

  .actions {
    display: flex;
    gap: var(--space-xs);
  }

  .btn {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-xs);
    padding: 12px 20px;
    border-radius: 12px;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
    border: none;
  }

  .btn svg {
    width: 18px;
    height: 18px;
  }

  .btn-primary {
    background: var(--primary, #6366f1);
    color: white;
  }

  .btn-primary:hover {
    background: var(--primary-hover, #4f46e5);
  }

  .btn-secondary {
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    color: var(--text-secondary);
  }

  .btn-secondary:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .info-box {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-md);
    background: var(--bg-tertiary);
    border-radius: 12px;
    font-size: 13px;
    color: var(--text-secondary);
  }

  .info-box svg {
    width: 18px;
    height: 18px;
    flex-shrink: 0;
    color: var(--text-muted);
  }

  /* Animation classes */
  :global(.animate-fade-in) {
    animation: fadeIn 0.3s ease-out;
  }

  :global(.animate-slide-up) {
    animation: slideUp 0.3s ease-out;
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  @keyframes slideUp {
    from {
      opacity: 0;
      transform: translateY(20px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
