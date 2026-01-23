<script lang="ts">
  import { voiceState, clearVoiceState } from "$lib/stores/voice";
  import { X } from "$lib/components/ui/icons";
  import { IconButton } from "$lib/components/ui/button";

  interface Props {
    onClose?: () => void;
  }

  let { onClose }: Props = $props();

  const hasTranscript = $derived($voiceState.transcript.length > 0);
  const isRecording = $derived($voiceState.isRecording);

  function handleClear() {
    clearVoiceState();
  }

  function handleClose() {
    onClose?.();
  }
</script>

{#if hasTranscript || isRecording}
  <div class="voice-panel">
    <div class="voice-panel-header">
      <div class="header-left">
        {#if isRecording}
          <span class="recording-indicator"></span>
        {/if}
        <span class="title">
          {isRecording ? "Recording..." : "Transcript"}
        </span>
      </div>
      <div class="header-actions">
        {#if hasTranscript && !isRecording}
          <IconButton
            icon={X}
            variant="ghost"
            size="sm"
            onclick={handleClear}
            title="Clear transcript"
          />
        {/if}
        {#if onClose}
          <IconButton
            icon={X}
            variant="ghost"
            size="sm"
            onclick={handleClose}
            title="Close panel"
          />
        {/if}
      </div>
    </div>
    <div class="voice-panel-content">
      {#if hasTranscript}
        <p class="transcript">{$voiceState.transcript}</p>
      {:else if isRecording}
        <p class="placeholder">Listening for speech...</p>
      {/if}
    </div>
    {#if $voiceState.segments.length > 1}
      <div class="voice-panel-segments">
        <span class="segments-label">
          {$voiceState.segments.length} segments
        </span>
      </div>
    {/if}
  </div>
{/if}

<style>
  .voice-panel {
    background: var(--bg-secondary);
    border: 1px solid var(--border-hex);
    border-radius: 0.5rem;
    overflow: hidden;
  }

  .voice-panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 0.75rem;
    background: var(--bg-tertiary);
    border-bottom: 1px solid var(--border-hex);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .recording-indicator {
    width: 8px;
    height: 8px;
    background: var(--error);
    border-radius: 50%;
    animation: pulse 1.5s ease-in-out infinite;
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.4;
    }
  }

  .title {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-primary);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .voice-panel-content {
    padding: 0.75rem;
    max-height: 200px;
    overflow-y: auto;
  }

  .transcript {
    font-size: 0.875rem;
    line-height: 1.5;
    color: var(--text-primary);
    margin: 0;
  }

  .placeholder {
    font-size: 0.875rem;
    color: var(--text-tertiary);
    font-style: italic;
    margin: 0;
  }

  .voice-panel-segments {
    padding: 0.5rem 0.75rem;
    background: var(--bg-tertiary);
    border-top: 1px solid var(--border-hex);
  }

  .segments-label {
    font-size: 0.75rem;
    color: var(--text-tertiary);
  }
</style>
