<script lang="ts">
  import { onMount, onDestroy } from "svelte";

  export let duration: number | undefined = undefined;
  export let reason: string | undefined = undefined;

  let remainingMs = duration || 0;
  let interval: ReturnType<typeof setInterval> | undefined;

  $: minutes = Math.floor(remainingMs / 60000);
  $: seconds = Math.floor((remainingMs % 60000) / 1000);
  $: timeDisplay = `${minutes}:${seconds.toString().padStart(2, '0')}`;

  onMount(() => {
    if (duration && duration > 0) {
      interval = setInterval(() => {
        remainingMs = Math.max(0, remainingMs - 1000);
      }, 1000);
    }
  });

  onDestroy(() => {
    if (interval) clearInterval(interval);
  });
</script>

<div class="sleep-indicator">
  <div class="sleep-icon">
    <span class="moon">&#127769;</span>
    <span class="zzz">z<span class="z2">z</span><span class="z3">z</span></span>
  </div>
  <div class="sleep-content">
    <span class="sleep-text">{reason || "Sleeping..."}</span>
    {#if duration && duration > 0}
      <span class="countdown">{timeDisplay} remaining</span>
    {/if}
  </div>
  <span class="hint">Type a message to wake</span>
</div>

<style>
  .sleep-indicator {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    margin: 8px 0;
    background: var(--bg-tertiary, #2a2a2a);
    border-radius: 8px;
    border: 1px solid var(--border-color, #3a3a3a);
  }

  .sleep-icon {
    position: relative;
    font-size: 20px;
  }

  .moon {
    animation: pulse 3s ease-in-out infinite;
  }

  .zzz {
    position: absolute;
    top: -8px;
    right: -12px;
    font-size: 10px;
    font-weight: bold;
    color: var(--accent-color, #4a9eff);
  }

  .z2 {
    animation: float 1.5s ease-in-out infinite;
    animation-delay: 0.2s;
    display: inline-block;
  }

  .z3 {
    animation: float 1.5s ease-in-out infinite;
    animation-delay: 0.4s;
    display: inline-block;
  }

  @keyframes float {
    0%, 100% {
      transform: translateY(0);
      opacity: 1;
    }
    50% {
      transform: translateY(-3px);
      opacity: 0.6;
    }
  }

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
      transform: scale(1);
    }
    50% {
      opacity: 0.7;
      transform: scale(0.95);
    }
  }

  .sleep-content {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
  }

  .sleep-text {
    font-size: 13px;
    color: var(--text-primary, #fff);
  }

  .countdown {
    font-family: monospace;
    font-size: 12px;
    color: var(--text-secondary, #888);
  }

  .hint {
    font-size: 11px;
    color: var(--text-secondary, #888);
    font-style: italic;
  }
</style>
