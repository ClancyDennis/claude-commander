<script lang="ts">
  import { Volume2, VolumeX } from "$lib/components/ui/icons";

  interface Props {
    isMuted: boolean;
    onToggleMute: () => void;
  }

  let { isMuted, onToggleMute }: Props = $props();
</script>

<div class="audio-controls">
  <button
    type="button"
    class="mute-button"
    class:muted={isMuted}
    onclick={onToggleMute}
  >
    {#if isMuted}
      <VolumeX size={16} />
      <span>Unmute</span>
    {:else}
      <Volume2 size={16} />
      <span>Mute</span>
    {/if}
  </button>
</div>

<style>
  .audio-controls {
    display: flex;
    justify-content: center;
    padding: 0.5rem 0;
  }

  .mute-button {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    border: 1px solid var(--border-hex);
    border-radius: 0.5rem;
    background: var(--bg-secondary);
    color: var(--text-primary);
    font-size: 0.875rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .mute-button:hover {
    background: var(--bg-tertiary);
    border-color: var(--accent-hex);
  }

  .mute-button.muted {
    background: rgba(239, 68, 68, 0.1);
    border-color: rgb(239, 68, 68);
    color: rgb(239, 68, 68);
  }
</style>
