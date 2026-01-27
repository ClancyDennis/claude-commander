<script lang="ts">
  import { Mic } from "$lib/components/ui/icons";

  interface Props {
    transcript: string;
    isActive: boolean;
    error: string | null;
  }

  let { transcript, isActive, error }: Props = $props();
</script>

<div class="dictate-content">
  {#if transcript}
    <div class="transcript-box">
      <p class="transcript-text">{transcript}</p>
    </div>
  {:else if isActive}
    <div class="listening-state">
      <div class="pulse-ring"></div>
      <Mic size={32} />
      <p>Listening...</p>
    </div>
  {:else}
    <div class="empty-state">
      <p>Click Start to begin dictating.</p>
    </div>
  {/if}

  {#if error}
    <div class="error-message">{error}</div>
  {/if}
</div>

<style>
  .dictate-content {
    min-height: 200px;
    padding: 0 0.25rem;
  }

  .empty-state {
    text-align: center;
    padding: 2rem 1rem;
    color: var(--text-secondary);
  }

  .empty-state p {
    margin: 0 0 0.5rem;
    font-size: 0.875rem;
  }

  .listening-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    padding: 3rem 1rem;
    color: var(--accent-hex);
    position: relative;
  }

  .pulse-ring {
    position: absolute;
    width: 80px;
    height: 80px;
    border-radius: 50%;
    background: rgba(var(--accent-rgb), 0.1);
    animation: pulse-ring 1.5s ease-out infinite;
  }

  @keyframes pulse-ring {
    0% {
      transform: scale(0.8);
      opacity: 1;
    }
    100% {
      transform: scale(1.5);
      opacity: 0;
    }
  }

  .transcript-box {
    background: var(--bg-tertiary);
    border-radius: 0.5rem;
    padding: 1rem;
    margin: 1rem 0;
  }

  .transcript-text {
    margin: 0;
    font-size: 0.875rem;
    line-height: 1.6;
  }

  .error-message {
    margin-top: 1rem;
    padding: 0.5rem 0.75rem;
    background: var(--error);
    color: white;
    font-size: 0.75rem;
    border-radius: 0.375rem;
  }
</style>
