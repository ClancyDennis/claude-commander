<script lang="ts">
  import { Mic, MicOff, Loader2 } from "$lib/components/ui/icons";
  import { IconButton } from "$lib/components/ui/button";
  import {
    voiceState,
    startVoiceSession,
    stopVoiceSession,
    initVoiceListeners,
  } from "$lib/stores/voice";
  import { useAudioCapture } from "$lib/hooks/useAudioCapture.svelte";
  import { onMount } from "svelte";

  const audioCapture = useAudioCapture();

  let isStarting = $state(false);

  onMount(() => {
    initVoiceListeners();
  });

  async function handleClick() {
    if ($voiceState.isRecording) {
      // Stop recording
      audioCapture.stop();
      await stopVoiceSession();
    } else if (!$voiceState.isConnecting && !isStarting) {
      // Start recording
      isStarting = true;
      try {
        await startVoiceSession();
        await audioCapture.start();
      } finally {
        isStarting = false;
      }
    }
  }

  const isActive = $derived($voiceState.isRecording);
  const isLoading = $derived($voiceState.isConnecting || isStarting);
  const currentIcon = $derived(
    isLoading ? Loader2 : isActive ? MicOff : Mic
  );
  const buttonLabel = $derived(
    isLoading ? "Connecting..." : isActive ? "Stop" : "Voice"
  );
  const buttonVariant = $derived(isActive ? "active" : "ghost");
</script>

<IconButton
  icon={currentIcon}
  label={buttonLabel}
  variant={buttonVariant}
  onclick={handleClick}
  disabled={isLoading}
  title={isActive ? "Stop voice recording" : "Start voice recording"}
  class={isLoading ? "animate-pulse" : ""}
/>

{#if $voiceState.error}
  <div class="voice-error" role="alert">
    {$voiceState.error}
  </div>
{/if}

<style>
  .voice-error {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 0.5rem;
    padding: 0.5rem 0.75rem;
    background: var(--error);
    color: white;
    font-size: 0.75rem;
    border-radius: 0.375rem;
    white-space: nowrap;
    z-index: 50;
  }
</style>
