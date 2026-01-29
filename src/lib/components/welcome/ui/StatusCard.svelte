<script lang="ts">
  import { Check, Terminal, Loader2 } from "lucide-svelte";

  interface Props {
    title: string;
    status: "checking" | "success" | "warning";
    message?: string;
    version?: string;
    authenticated?: boolean;
    subscriptionType?: string | null;
    showBadge?: boolean;
    badgeText?: string;
  }

  let {
    title,
    status,
    message = "",
    version,
    authenticated = false,
    subscriptionType = null,
    showBadge = false,
    badgeText = "Ready"
  }: Props = $props();
</script>

<div class="status-card" class:success={status === "success"}>
  <div
    class="status-icon"
    class:success={status === "success"}
    class:checking={status === "checking"}
  >
    {#if status === "checking"}
      <Loader2 class="icon-spin" />
    {:else if status === "success"}
      <Check />
    {:else}
      <Terminal />
    {/if}
  </div>

  <div class="status-content">
    <h3>{title}</h3>
    {#if status === "checking"}
      <p class="status-message">Checking installation...</p>
    {:else if status === "success"}
      <p class="status-message success">
        {message}
        {#if version}
          <span class="version">({version})</span>
        {/if}
        {#if authenticated}
          <span class="auth-indicator"> · Authenticated</span>
          {#if subscriptionType}
            <span class="subscription-badge">{subscriptionType}</span>
          {/if}
        {/if}
      </p>
    {:else}
      <p class="status-message warning">{message}</p>
    {/if}
  </div>

  {#if showBadge && status === "success"}
    <div class="status-badge success">
      <Check class="badge-icon" />
      {badgeText}
    </div>
  {/if}
</div>

<style>
  .status-card {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-5) var(--space-6);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-lg);
    transition: all var(--transition-normal);
  }

  .status-card.success {
    border-color: rgba(52, 199, 89, 0.3);
    background: rgba(52, 199, 89, 0.08);
  }

  .status-icon {
    width: 48px;
    height: 48px;
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-elevated);
    color: var(--text-muted);
    transition: all var(--transition-normal);
  }

  .status-icon :global(svg) {
    width: 24px;
    height: 24px;
  }

  .status-icon.success {
    background: var(--success-glow);
    color: var(--success-hex);
  }

  .status-icon.checking {
    background: var(--accent-glow);
    color: var(--accent-hex);
  }

  .status-content {
    flex: 1;
  }

  .status-content h3 {
    font-size: var(--text-lg);
    font-weight: var(--font-medium);
    margin: 0 0 var(--space-1);
    color: var(--text-primary);
  }

  .status-message {
    font-size: var(--text-sm);
    color: var(--text-muted);
    margin: 0;
  }

  .status-message.success {
    color: var(--success-hex);
  }

  .status-message.warning {
    color: var(--warning-hex);
  }

  .version {
    opacity: 0.8;
    font-size: var(--text-xs);
  }

  .auth-indicator {
    color: var(--success-hex);
  }

  .subscription-badge {
    display: inline-block;
    margin-left: var(--space-2);
    padding: 2px 8px;
    font-size: var(--text-xs);
    font-weight: var(--font-medium);
    background: var(--accent-glow);
    color: var(--accent-hex);
    border-radius: var(--radius-full);
    text-transform: capitalize;
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

  :global(.badge-icon) {
    width: 12px;
    height: 12px;
  }

  :global(.icon-spin) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }
</style>
