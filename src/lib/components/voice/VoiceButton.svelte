<script lang="ts">
  import { Mic, MicOff, Loader2, Volume2, VolumeX } from "$lib/components/ui/icons";
  import { IconButton } from "$lib/components/ui/button";
  import {
    voiceState,
    startVoiceSession,
    stopVoiceSession,
    initVoiceListeners,
    setAudioPlaybackCallback,
  } from "$lib/stores/voice";
  import { useAudioCapture } from "$lib/hooks/useAudioCapture.svelte";
  import { useAudioPlayback } from "$lib/hooks/useAudioPlayback.svelte";
  import { onMount } from "svelte";

  const audioCapture = useAudioCapture();
  const audioPlayback = useAudioPlayback();

  let isStarting = $state(false);

  onMount(() => {
    initVoiceListeners();
    // Set up audio playback callback
    setAudioPlaybackCallback((audio) => {
      audioPlayback.playChunk(audio);
    });

    return () => {
      setAudioPlaybackCallback(null);
      audioPlayback.cleanup();
    };
  });

  async function handleClick() {
    if ($voiceState.isRecording) {
      // Stop recording
      audioCapture.stop();
      audioPlayback.stop();
      await stopVoiceSession();
    } else if (!$voiceState.isConnecting && !isStarting) {
      // Start recording
      isStarting = true;
      try {
        // Initialize audio playback (requires user interaction)
        audioPlayback.init();
        await audioPlayback.resume();

        await startVoiceSession();
        await audioCapture.start();
      } finally {
        isStarting = false;
      }
    }
  }

  function handleMuteToggle() {
    audioPlayback.toggleMute();
  }

  const isActive = $derived($voiceState.isRecording);
  const isLoading = $derived($voiceState.isConnecting || isStarting);
  const isMuted = $derived(audioPlayback.state.isMuted);
  const currentIcon = $derived(
    isLoading ? Loader2 : isActive ? MicOff : Mic
  );
  const buttonLabel = $derived(
    isLoading ? "Connecting..." : isActive ? "Stop" : "Dictate"
  );
  const buttonVariant = $derived(isActive ? "active" : "ghost");
</script>

<div class="voice-controls">
  <IconButton
    icon={currentIcon}
    label={buttonLabel}
    variant={buttonVariant}
    onclick={handleClick}
    disabled={isLoading}
    title={isActive ? "Stop voice recording" : "Start voice dictation"}
    class={isLoading ? "animate-pulse" : ""}
  />
  {#if isActive}
    <IconButton
      icon={isMuted ? VolumeX : Volume2}
      variant="ghost"
      size="sm"
      onclick={handleMuteToggle}
      title={isMuted ? "Unmute AI voice" : "Mute AI voice"}
    />
  {/if}
</div>

{#if $voiceState.error}
  <div class="voice-error" role="alert">
    {$voiceState.error}
  </div>
{/if}

<style>
  .voice-controls {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

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
