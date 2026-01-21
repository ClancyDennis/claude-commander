<script lang="ts">
  import { rateLimitState, clearRateLimit } from "../stores/rateLimit";

  let timeRemaining = $state("");
  let intervalId: ReturnType<typeof setInterval> | null = null;

  function formatTimeRemaining(ms: number): string {
    if (ms <= 0) return "0:00";

    const totalSeconds = Math.floor(ms / 1000);
    const hours = Math.floor(totalSeconds / 3600);
    const minutes = Math.floor((totalSeconds % 3600) / 60);
    const seconds = totalSeconds % 60;

    if (hours > 0) {
      return `${hours}:${minutes.toString().padStart(2, "0")}:${seconds.toString().padStart(2, "0")}`;
    }
    return `${minutes}:${seconds.toString().padStart(2, "0")}`;
  }

  function updateCountdown() {
    const state = $rateLimitState;
    if (!state.isLimited || !state.resetTime) {
      timeRemaining = "";
      return;
    }

    const now = Date.now();
    const remaining = state.resetTime.getTime() - now;

    if (remaining <= 0) {
      clearRateLimit();
      timeRemaining = "";
      return;
    }

    timeRemaining = formatTimeRemaining(remaining);
  }

  $effect(() => {
    if ($rateLimitState.isLimited) {
      updateCountdown();
      intervalId = setInterval(updateCountdown, 1000);
    }

    return () => {
      if (intervalId) {
        clearInterval(intervalId);
        intervalId = null;
      }
    };
  });
</script>

{#if $rateLimitState.isLimited}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="overlay animate-fade-in">
    <div class="dialog animate-slide-up" role="dialog" aria-modal="true" tabindex="-1">
      <div class="icon-container">
        <div class="icon-bg">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <polyline points="12 6 12 12 16 14"/>
          </svg>
        </div>
      </div>

      <h2>Rate Limit Reached</h2>
      <p class="subtitle">You've reached your usage limit</p>

      <div class="countdown-container">
        <div class="countdown-label">Access resumes in</div>
        <div class="countdown-time">{timeRemaining}</div>
        {#if $rateLimitState.resetTime}
          <div class="reset-time">
            at {$rateLimitState.resetTime.toLocaleTimeString([], { hour: 'numeric', minute: '2-digit' })}
            {#if $rateLimitState.timezone}
              <span class="timezone">({$rateLimitState.timezone})</span>
            {/if}
          </div>
        {/if}
      </div>

      <div class="switch-section">
        <div class="switch-label">Switch to</div>
        <div class="switch-buttons">
          <button class="switch-btn" disabled>
            <svg viewBox="0 0 24 24" fill="currentColor" width="16" height="16">
              <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z"/>
            </svg>
            Gemini
          </button>
          <button class="switch-btn" disabled>
            <svg viewBox="0 0 24 24" fill="currentColor" width="16" height="16">
              <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8zm-1-13h2v6h-2zm0 8h2v2h-2z"/>
            </svg>
            Codex
          </button>
          <button class="switch-btn" disabled>
            <svg viewBox="0 0 24 24" fill="currentColor" width="16" height="16">
              <path d="M9.4 16.6L4.8 12l4.6-4.6L8 6l-6 6 6 6 1.4-1.4zm5.2 0l4.6-4.6-4.6-4.6L16 6l6 6-6 6-1.4-1.4z"/>
            </svg>
            Cline
          </button>
        </div>
      </div>

      <div class="info-box">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"/>
          <line x1="12" y1="16" x2="12" y2="12"/>
          <line x1="12" y1="8" x2="12.01" y2="8"/>
        </svg>
        <span>This modal will automatically close when your access is restored.</span>
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
    width: 420px;
    max-width: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    padding: var(--space-xl, 32px);
    background-color: var(--bg-secondary);
    border-radius: 24px;
    border: 1px solid var(--border);
    box-shadow: var(--shadow-lg), 0 0 80px rgba(239, 68, 68, 0.15);
  }

  .icon-container {
    margin-bottom: var(--space-lg);
  }

  .icon-bg {
    width: 72px;
    height: 72px;
    border-radius: 50%;
    background: linear-gradient(135deg, rgba(239, 68, 68, 0.2) 0%, rgba(239, 68, 68, 0.1) 100%);
    border: 2px solid var(--error);
    display: flex;
    align-items: center;
    justify-content: center;
    animation: pulse 2s ease-in-out infinite;
  }

  .icon-bg svg {
    width: 36px;
    height: 36px;
    color: var(--error);
  }

  @keyframes pulse {
    0%, 100% {
      transform: scale(1);
      opacity: 1;
    }
    50% {
      transform: scale(1.05);
      opacity: 0.9;
    }
  }

  h2 {
    font-size: 24px;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0 0 var(--space-xs) 0;
  }

  .subtitle {
    font-size: 15px;
    color: var(--text-muted);
    margin: 0 0 var(--space-xl, 32px) 0;
  }

  .countdown-container {
    width: 100%;
    padding: var(--space-lg);
    background: var(--bg-elevated);
    border-radius: 16px;
    border: 1px solid var(--border);
    margin-bottom: var(--space-lg);
  }

  .countdown-label {
    font-size: 13px;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: var(--space-sm);
  }

  .countdown-time {
    font-size: 48px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    color: var(--error);
    line-height: 1;
    margin-bottom: var(--space-sm);
  }

  .reset-time {
    font-size: 14px;
    color: var(--text-secondary);
  }

  .timezone {
    color: var(--text-muted);
  }

  .switch-section {
    width: 100%;
    margin-bottom: var(--space-lg);
  }

  .switch-label {
    font-size: 13px;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: var(--space-sm);
  }

  .switch-buttons {
    display: flex;
    gap: var(--space-sm);
    justify-content: center;
  }

  .switch-btn {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: 10px 16px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 10px;
    color: var(--text-muted);
    font-size: 14px;
    font-weight: 500;
    cursor: not-allowed;
    opacity: 0.5;
    transition: all 0.2s ease;
  }

  .switch-btn svg {
    width: 16px;
    height: 16px;
  }

  .switch-btn:disabled {
    opacity: 0.4;
  }

  .info-box {
    width: 100%;
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
