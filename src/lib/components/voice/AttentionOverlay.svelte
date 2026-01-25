<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { X, Mic, Volume2, VolumeX } from "$lib/components/ui/icons";
  import { IconButton } from "$lib/components/ui/button";
  import {
    attentionState,
    initAttentionListeners,
    stopAttentionSession,
    sendAttentionAudio,
    setAttentionAudioCallback,
  } from "$lib/stores/voice";
  import { useAudioCapture } from "$lib/hooks/useAudioCapture.svelte";
  import { useAudioPlayback } from "$lib/hooks/useAudioPlayback.svelte";

  const audioCapture = useAudioCapture();
  const audioPlayback = useAudioPlayback();

  // Derived state
  const isActive = $derived($attentionState.isActive);
  const isConnecting = $derived($attentionState.isConnecting);
  const agentTitle = $derived($attentionState.agentTitle);
  const response = $derived($attentionState.response);
  const transcript = $derived($attentionState.transcript);
  const isMuted = $derived(audioPlayback.state.isMuted);

  onMount(() => {
    initAttentionListeners();

    // Initialize audio playback immediately so it's ready when sessions start
    audioPlayback.init();

    // Set up audio playback callback - must be set before any session starts
    setAttentionAudioCallback(async (audio) => {
      // Ensure audio context is resumed (may be suspended until user interaction)
      await audioPlayback.resume();
      audioPlayback.playChunk(audio);
    });

    // Start capturing audio when attention mode becomes active
    const unsubscribe = attentionState.subscribe(async (state) => {
      if (state.isActive && !audioCapture.state.isRecording) {
        // Ensure audio is ready when session becomes active
        audioPlayback.init();
        await audioPlayback.resume();
        audioCapture.start((chunk) => {
          sendAttentionAudio(chunk);
        });
      } else if (!state.isActive && audioCapture.state.isRecording) {
        audioCapture.stop();
        audioPlayback.stop();
      }
    });

    return () => {
      unsubscribe();
    };
  });

  onDestroy(() => {
    setAttentionAudioCallback(null);
    if (audioCapture.state.isRecording) {
      audioCapture.stop();
    }
    audioPlayback.stop();
  });

  function handleDismiss() {
    stopAttentionSession();
  }

  function toggleMute() {
    audioPlayback.toggleMute();
  }

  // Computed icon for mute button
  const muteIcon = $derived(isMuted ? VolumeX : Volume2);
</script>

{#if isActive || isConnecting}
  <div class="attention-overlay">
    <div class="header">
      <div class="icon-pulse">
        <Mic class="h-4 w-4" />
      </div>
      <span class="title">{agentTitle} completed</span>
      <div class="actions">
        <IconButton
          icon={muteIcon}
          variant="ghost"
          size="sm"
          onclick={toggleMute}
          label={isMuted ? "Unmute" : "Mute"}
        />
        <IconButton
          icon={X}
          variant="ghost"
          size="sm"
          onclick={handleDismiss}
          label="Dismiss"
        />
      </div>
    </div>

    {#if response || transcript}
      <div class="content">
        {#if response}
          <p class="response">{response}</p>
        {/if}
        {#if transcript}
          <p class="transcript">You: {transcript}</p>
        {/if}
      </div>
    {:else if isConnecting}
      <div class="content">
        <p class="connecting">Connecting...</p>
      </div>
    {/if}
  </div>
{/if}

<style>
  .attention-overlay {
    position: fixed;
    top: 1rem;
    right: 1rem;
    width: 320px;
    max-width: calc(100vw - 2rem);
    background: hsl(var(--card));
    border: 1px solid hsl(var(--border));
    border-radius: var(--radius);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    z-index: 1000;
    overflow: hidden;
  }

  .header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    background: hsl(var(--muted));
    border-bottom: 1px solid hsl(var(--border));
  }

  .icon-pulse {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: hsl(var(--destructive));
    color: hsl(var(--destructive-foreground));
    animation: pulse 1.5s ease-in-out infinite;
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
      transform: scale(1);
    }
    50% {
      opacity: 0.7;
      transform: scale(1.1);
    }
  }

  .title {
    flex: 1;
    font-weight: 500;
    font-size: 0.875rem;
    color: hsl(var(--foreground));
  }

  .actions {
    display: flex;
    gap: 0.25rem;
  }

  .content {
    padding: 0.75rem 1rem;
    max-height: 200px;
    overflow-y: auto;
  }

  .response {
    font-size: 0.875rem;
    color: hsl(var(--foreground));
    line-height: 1.5;
    margin: 0;
  }

  .transcript {
    font-size: 0.75rem;
    color: hsl(var(--muted-foreground));
    margin: 0.5rem 0 0;
    font-style: italic;
  }

  .connecting {
    font-size: 0.875rem;
    color: hsl(var(--muted-foreground));
    margin: 0;
  }
</style>
