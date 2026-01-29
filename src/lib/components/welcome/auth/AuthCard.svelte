<script lang="ts">
  import { ChevronDown, ChevronUp, Check, AlertCircle, Terminal, Key } from "lucide-svelte";
  import type { Snippet } from "svelte";

  type IconType = "claude" | "anthropic" | "openai";
  type StatusType = "unchecked" | "ready" | "valid" | "invalid" | "validating";

  interface Props {
    title: string;
    description: string;
    iconType: IconType;
    expanded: boolean;
    status: StatusType;
    onToggle: () => void;
    children?: Snippet;
  }

  let { title, description, iconType, expanded, status, onToggle, children }: Props = $props();

  const showSuccessBadge = $derived(status === "ready" || status === "valid");
  const showErrorBadge = $derived(status === "invalid");
</script>

<div class="auth-card" class:expanded>
  <button class="auth-toggle" onclick={onToggle}>
    <div class="auth-icon {iconType}">
      {#if iconType === "claude"}
        <Terminal />
      {:else}
        <Key />
      {/if}
    </div>
    <div class="auth-info">
      <h4>{title}</h4>
      <p>{@html description}</p>
    </div>
    <div class="auth-toggle-icon">
      {#if showSuccessBadge}
        <div class="status-badge success small">
          <Check class="badge-icon" />
        </div>
      {:else if showErrorBadge}
        <div class="status-badge error small">
          <AlertCircle class="badge-icon" />
        </div>
      {:else if expanded}
        <ChevronUp />
      {:else}
        <ChevronDown />
      {/if}
    </div>
  </button>

  {#if expanded}
    <div class="auth-expand">
      {#if children}
        {@render children()}
      {/if}
    </div>
  {/if}
</div>

<style>
  .auth-card {
    background: var(--bg-tertiary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-lg);
    overflow: hidden;
    transition: all var(--transition-normal);
  }

  .auth-card.expanded {
    border-color: var(--accent-hex);
    box-shadow: 0 0 0 3px var(--accent-glow);
  }

  .auth-toggle {
    width: 100%;
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-4) var(--space-5);
    background: transparent;
    border: none;
    cursor: pointer;
    text-align: left;
  }

  .auth-toggle:hover {
    background: rgba(255, 255, 255, 0.03);
  }

  .auth-icon {
    width: 40px;
    height: 40px;
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-elevated);
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .auth-icon :global(svg) {
    width: 20px;
    height: 20px;
  }

  .auth-card.expanded .auth-icon {
    background: var(--accent-glow);
    color: var(--accent-hex);
  }

  .auth-icon.claude {
    background: rgba(232, 102, 77, 0.15);
    color: var(--accent-hex);
  }

  .auth-icon.anthropic {
    background: rgba(232, 102, 77, 0.15);
    color: var(--accent-hex);
  }

  .auth-icon.openai {
    background: rgba(16, 163, 127, 0.15);
    color: #10a37f;
  }

  .auth-info {
    flex: 1;
    min-width: 0;
  }

  .auth-info h4 {
    font-size: var(--text-base);
    font-weight: var(--font-medium);
    margin: 0 0 var(--space-1);
    color: var(--text-primary);
  }

  .auth-info p {
    font-size: var(--text-sm);
    color: var(--text-muted);
    margin: 0;
  }

  .auth-info :global(code) {
    font-size: var(--text-xs);
    padding: 2px 6px;
    background: var(--bg-elevated);
    border-radius: var(--radius-sm);
    font-family: "SF Mono", ui-monospace, monospace;
  }

  .auth-toggle-icon {
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .auth-toggle-icon :global(svg) {
    width: 18px;
    height: 18px;
  }

  .auth-expand {
    padding: 0 var(--space-5) var(--space-5);
    animation: slideUp 0.2s var(--spring-bounce);
  }

  .status-badge {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    font-size: var(--text-xs);
    font-weight: var(--font-medium);
    padding: var(--space-1) var(--space-3);
    border-radius: var(--radius-full);
  }

  .status-badge.success {
    background: var(--success-glow);
    color: var(--success-hex);
  }

  .status-badge.error {
    background: var(--error-glow);
    color: var(--error);
  }

  .status-badge.small {
    padding: var(--space-1) var(--space-2);
  }

  :global(.badge-icon) {
    width: 12px;
    height: 12px;
  }

  @keyframes slideUp {
    from {
      opacity: 0;
      transform: translateY(-8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
