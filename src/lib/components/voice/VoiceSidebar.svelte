<script lang="ts">
  import { onMount, tick } from "svelte";
  import * as Sheet from "$lib/components/ui/sheet";
  import { Button } from "$lib/components/ui/button";
  import { IconButton } from "$lib/components/ui/button";
  import {
    Mic,
    MicOff,
    Volume2,
    VolumeX,
    Loader2,
    Check,
    Send,
    X,
    MessageCircle,
  } from "$lib/components/ui/icons";
  import {
    voiceState,
    discussState,
    startVoiceSession,
    stopVoiceSession,
    startDiscussSession,
    stopDiscussSession,
    initVoiceListeners,
    initDiscussListeners,
    setAudioPlaybackCallback,
    setDiscussAudioCallback,
    setMissionControlCallback,
    clearDiscussState,
    toggleMessageSelection,
    getSelectedMessages,
    clearMessageSelections,
    pendingChatInput,
    voiceSidebarOpen,
    sendAudioChunk,
    sendDiscussAudio,
    type DiscussMessage,
  } from "$lib/stores/voice";
  import { useAudioCapture } from "$lib/hooks/useAudioCapture.svelte";
  import { useAudioPlayback } from "$lib/hooks/useAudioPlayback.svelte";

  interface Props {
    onSendToChat?: (text: string) => void;
  }

  let { onSendToChat }: Props = $props();

  type Mode = "select" | "dictate" | "discuss";
  let mode = $state<Mode>("select");

  const audioCapture = useAudioCapture();
  const audioPlayback = useAudioPlayback();

  let isDictateStarting = $state(false);
  let isDiscussStarting = $state(false);
  let conversationEl: HTMLDivElement | null = $state(null);

  // Derived state
  const isDictateActive = $derived($voiceState.isRecording);
  const isDiscussActive = $derived($discussState.isActive);
  const isDiscussConnecting = $derived($discussState.isConnecting || isDiscussStarting);
  const isMuted = $derived(audioPlayback.state.isMuted);
  const discussMessages = $derived($discussState.messages);
  const currentUserTranscript = $derived($discussState.currentUserTranscript);
  const currentAssistantResponse = $derived($discussState.currentAssistantResponse);
  const hasDiscussMessages = $derived(
    discussMessages.length > 0 ||
      currentUserTranscript.length > 0 ||
      currentAssistantResponse.length > 0
  );
  const selectedCount = $derived(discussMessages.filter((m) => m.selected).length);

  // Auto-scroll discuss messages
  $effect(() => {
    if (discussMessages.length > 0 || currentUserTranscript || currentAssistantResponse) {
      tick().then(() => {
        if (conversationEl) {
          conversationEl.scrollTop = conversationEl.scrollHeight;
        }
      });
    }
  });

  // Auto-select mode based on active state
  $effect(() => {
    if (isDictateActive && mode !== "dictate") {
      mode = "dictate";
    } else if ((isDiscussActive || isDiscussConnecting) && mode !== "discuss") {
      mode = "discuss";
    }
  });

  onMount(() => {
    initVoiceListeners();
    initDiscussListeners();

    // Set up audio playback callbacks
    setAudioPlaybackCallback((audio) => {
      audioPlayback.playChunk(audio);
    });

    setDiscussAudioCallback((audio) => {
      audioPlayback.playChunk(audio);
    });

    setMissionControlCallback(async (message: string) => {
      console.log("[Voice] Mission control request:", message);
      return "Message received by mission control";
    });

    return () => {
      setAudioPlaybackCallback(null);
      setDiscussAudioCallback(null);
      setMissionControlCallback(null);
      audioPlayback.cleanup();
    };
  });

  // Dictate handlers
  async function handleStartDictate() {
    if (isDictateActive || isDictateStarting) return;
    mode = "dictate";
    isDictateStarting = true;
    try {
      audioPlayback.init();
      await audioPlayback.resume();
      await startVoiceSession();
      await audioCapture.start(sendAudioChunk);
    } finally {
      isDictateStarting = false;
    }
  }

  async function handleStopDictate() {
    audioCapture.stop();
    audioPlayback.stop();
    await stopVoiceSession();
  }

  // Discuss handlers
  async function handleStartDiscuss() {
    if (isDiscussActive || isDiscussConnecting) return;
    mode = "discuss";
    isDiscussStarting = true;
    try {
      audioPlayback.init();
      await audioPlayback.resume();
      await startDiscussSession();
      await audioCapture.start(sendDiscussAudio);
    } finally {
      isDiscussStarting = false;
    }
  }

  async function handleStopDiscuss() {
    audioCapture.stop();
    audioPlayback.stop();
    await stopDiscussSession();
  }

  function handleMuteToggle() {
    audioPlayback.toggleMute();
  }

  function handleMessageClick(msg: DiscussMessage) {
    toggleMessageSelection(msg.id);
  }

  function handleSendSelected() {
    const selected = getSelectedMessages();
    if (selected.length === 0) return;

    const text = selected
      .map((m) => `[${m.role === "user" ? "You" : "AI"}]: ${m.content}`)
      .join("\n\n");

    if (onSendToChat) {
      onSendToChat(text);
    } else {
      pendingChatInput.set(text);
    }

    clearMessageSelections();
    voiceSidebarOpen.set(false);
  }

  function handleClearHistory() {
    clearDiscussState();
  }

  function handleSendDictateToChat() {
    const text = $voiceState.transcript;
    if (!text) return;

    if (onSendToChat) {
      onSendToChat(text);
    } else {
      pendingChatInput.set(text);
    }

    voiceSidebarOpen.set(false);
  }

  function handleClose() {
    // Just close the sidebar - don't stop voice sessions
    // User can continue using other parts of the app while voice runs
    voiceSidebarOpen.set(false);
  }

  async function handleEndSession() {
    // Explicitly end the current voice session
    // Keep the current mode so user can still see transcript/conversation and send to chat
    if (isDictateActive) {
      await handleStopDictate();
    }
    if (isDiscussActive) {
      await handleStopDiscuss();
    }
    // Don't reset mode - let user access Send to Chat functionality
  }

  // Handle Sheet's onOpenChange callback
  function handleOpenChange(isOpen: boolean) {
    voiceSidebarOpen.set(isOpen);
  }
</script>

<Sheet.Root open={$voiceSidebarOpen} onOpenChange={handleOpenChange}>
  <Sheet.Content side="right" class="w-[400px] flex flex-col">
    <Sheet.Header class="space-y-4">
      <div class="flex items-center justify-between">
        <Sheet.Title class="flex items-center gap-2">
          {#if mode === "dictate"}
            <Mic size={18} />
            Dictate Mode
          {:else if mode === "discuss"}
            <MessageCircle size={18} />
            Discuss Mode
          {:else}
            <Mic size={18} />
            Voice
            <span class="title-hint">(choose a mode)</span>
          {/if}
        </Sheet.Title>
        <div class="flex items-center gap-2">
          {#if mode === "discuss" && discussMessages.length > 0 && !isDiscussActive}
            <IconButton
              icon={X}
              variant="ghost"
              size="sm"
              onclick={handleClearHistory}
              title="Clear history"
            />
          {/if}
        </div>
      </div>
      {#if isDictateActive || isDiscussActive}
        <div class="audio-controls">
          <button
            type="button"
            class="mute-button"
            class:muted={isMuted}
            onclick={handleMuteToggle}
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
      {/if}
    </Sheet.Header>

    <div class="flex-1 overflow-y-auto mt-4">
      {#if mode === "select"}
        <!-- Mode selection - side by side buttons -->
        <div class="mode-select-grid">
          <button
            type="button"
            class="mode-card-half"
            onclick={() => handleStartDictate()}
          >
            <div class="mode-icon dictate">
              <Mic size={32} />
            </div>
            <h3>Dictate</h3>
            <p>Speak to type</p>
          </button>

          <button
            type="button"
            class="mode-card-half"
            onclick={() => handleStartDiscuss()}
          >
            <div class="mode-icon discuss">
              <MessageCircle size={32} />
            </div>
            <h3>Discuss</h3>
            <p>Voice conversation</p>
          </button>
        </div>

      {:else if mode === "dictate"}
        <!-- Dictate mode -->
        <div class="dictate-content">
          {#if $voiceState.transcript}
            <div class="transcript-box">
              <p class="transcript-text">{$voiceState.transcript}</p>
            </div>
          {:else if isDictateActive}
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

          {#if $voiceState.error}
            <div class="error-message">{$voiceState.error}</div>
          {/if}
        </div>

      {:else if mode === "discuss"}
        <!-- Discuss mode -->
        <div class="discuss-content" bind:this={conversationEl}>
          {#if !hasDiscussMessages && !isDiscussActive && !isDiscussConnecting}
            <div class="empty-state">
              <p>Click Start to begin a voice discussion.</p>
              <p class="hint">You can ask the AI to talk to Mission Control for help.</p>
            </div>
          {:else if discussMessages.length > 0 && !isDiscussActive && !isDiscussConnecting}
            <p class="selection-hint">Tap messages to select, then send to chat</p>
          {/if}
          {#if hasDiscussMessages}
            <div class="conversation">
              {#each discussMessages as msg (msg.id)}
                <button
                  class="message {msg.role}"
                  class:selected={msg.selected}
                  onclick={() => handleMessageClick(msg)}
                  type="button"
                >
                  {#if msg.selected}
                    <span class="select-indicator">
                      <Check size={12} />
                    </span>
                  {/if}
                  <span class="label">{msg.role === "user" ? "You" : "AI"}</span>
                  <p class="text">{msg.content}</p>
                </button>
              {/each}

              {#if currentUserTranscript}
                <div class="message user current">
                  <span class="label">You</span>
                  <p class="text">{currentUserTranscript}<span class="cursor">|</span></p>
                </div>
              {:else if isDiscussActive && discussMessages.length === 0}
                <p class="listening-text">Listening...</p>
              {/if}

              {#if currentAssistantResponse}
                <div class="message assistant current">
                  <span class="label">AI</span>
                  <p class="text">{currentAssistantResponse}<span class="cursor">|</span></p>
                </div>
              {/if}
            </div>
          {/if}

          {#if $discussState.error}
            <div class="error-message">{$discussState.error}</div>
          {/if}
        </div>
      {/if}
    </div>

    <Sheet.Footer class="border-t pt-4 mt-4 space-y-2">
      {#if mode === "discuss" && selectedCount > 0}
        <Button class="w-full" onclick={handleSendSelected}>
          <Send size={16} />
          Send {selectedCount} to Chat
        </Button>
      {/if}

      {#if mode === "dictate" && !isDictateActive && $voiceState.transcript}
        <Button class="w-full" onclick={handleSendDictateToChat}>
          <Send size={16} />
          Send to Chat
        </Button>
      {/if}

      {#if mode === "dictate"}
        <div class="flex gap-2">
          <Button
            variant="outline"
            class="flex-1"
            onclick={() => { mode = "select"; }}
            disabled={isDictateActive}
          >
            Back
          </Button>
          {#if isDictateActive}
            <Button class="flex-1" variant="destructive" onclick={handleEndSession}>
              <MicOff size={16} />
              End Session
            </Button>
          {:else}
            <Button class="flex-1" onclick={handleStartDictate} disabled={isDictateStarting}>
              {#if isDictateStarting}
                <Loader2 size={16} class="animate-spin" />
              {:else}
                <Mic size={16} />
              {/if}
              Start
            </Button>
          {/if}
        </div>

      {:else if mode === "discuss"}
        <div class="flex gap-2">
          <Button
            variant="outline"
            class="flex-1"
            onclick={() => { mode = "select"; }}
            disabled={isDiscussActive || isDiscussConnecting}
          >
            Back
          </Button>
          {#if isDiscussActive}
            <Button class="flex-1" variant="destructive" onclick={handleEndSession}>
              <MicOff size={16} />
              End Session
            </Button>
          {:else}
            <Button class="flex-1" onclick={handleStartDiscuss} disabled={isDiscussConnecting}>
              {#if isDiscussConnecting}
                <Loader2 size={16} class="animate-spin" />
              {:else}
                <Mic size={16} />
              {/if}
              {discussMessages.length > 0 ? "Continue" : "Start"}
            </Button>
          {/if}
        </div>

      {/if}
    </Sheet.Footer>
  </Sheet.Content>
</Sheet.Root>

<style>
  .mode-select-grid {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    height: 100%;
    padding: 0.5rem 0.5rem;
  }

  .mode-card-half {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    padding: 2rem 1rem;
    border: 2px solid var(--border-hex);
    border-radius: 0.75rem;
    background: var(--bg-secondary);
    text-align: center;
    cursor: pointer;
    transition: all 0.15s ease;
    flex: 1;
  }

  .mode-card-half:hover {
    border-color: var(--accent-hex);
    background: var(--bg-tertiary);
    transform: translateY(-2px);
  }

  .mode-card-half h3 {
    font-weight: 600;
    font-size: 1rem;
    margin: 0;
  }

  .mode-card-half p {
    font-size: 0.75rem;
    color: var(--text-secondary);
    margin: 0;
  }

  .mode-card {
    display: flex;
    gap: 1rem;
    padding: 1rem;
    border: 1px solid var(--border-hex);
    border-radius: 0.5rem;
    background: var(--bg-secondary);
    text-align: left;
    cursor: pointer;
    transition: all 0.15s ease;
    width: 100%;
  }

  .mode-card:hover {
    border-color: var(--accent-hex);
    background: var(--bg-tertiary);
  }

  .mode-icon {
    width: 56px;
    height: 56px;
    border-radius: 0.75rem;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .mode-icon.dictate {
    background: rgba(59, 130, 246, 0.1);
    color: rgb(59, 130, 246);
  }

  .mode-icon.discuss {
    background: rgba(34, 197, 94, 0.1);
    color: rgb(34, 197, 94);
  }

  .mode-info h3 {
    font-weight: 600;
    font-size: 0.875rem;
    margin: 0 0 0.25rem;
  }

  .mode-info p {
    font-size: 0.75rem;
    color: var(--text-secondary);
    margin: 0;
    line-height: 1.4;
  }

  .dictate-content,
  .discuss-content {
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

  .empty-state .hint {
    font-size: 0.75rem;
    color: var(--text-tertiary);
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

  .conversation {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .message {
    padding: 0.75rem;
    border-radius: 0.5rem;
    border: none;
    cursor: pointer;
    text-align: left;
    transition: all 0.15s ease;
    position: relative;
    width: fit-content;
    max-width: 90%;
  }

  .message:hover {
    transform: translateX(2px);
  }

  .message.user {
    background: var(--bg-tertiary);
    align-self: flex-end;
  }

  .message.assistant {
    background: rgba(34, 197, 94, 0.1);
    border: 1px solid rgba(34, 197, 94, 0.2);
    align-self: flex-start;
  }

  .message.selected {
    outline: 2px solid var(--accent-hex);
    outline-offset: 2px;
  }

  .message.current {
    cursor: default;
    opacity: 0.8;
  }

  .message.current:hover {
    transform: none;
  }

  .select-indicator {
    position: absolute;
    top: 0.5rem;
    right: 0.5rem;
    width: 18px;
    height: 18px;
    background: var(--accent-hex);
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
  }

  .message .label {
    font-size: 0.625rem;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--text-tertiary);
    display: block;
    margin-bottom: 0.25rem;
  }

  .message .text {
    font-size: 0.875rem;
    line-height: 1.5;
    margin: 0;
    color: var(--text-primary);
  }

  .cursor {
    animation: blink 1s step-end infinite;
  }

  @keyframes blink {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0;
    }
  }

  .listening-text {
    font-size: 0.875rem;
    color: var(--text-tertiary);
    font-style: italic;
    text-align: center;
    margin: 0;
  }

  .selection-hint {
    font-size: 0.75rem;
    color: var(--text-tertiary);
    text-align: center;
    margin: 0 0 0.5rem 0;
    padding: 0.25rem 0.5rem;
    background: var(--bg-secondary);
    border-radius: 0.25rem;
  }

  .error-message {
    margin-top: 1rem;
    padding: 0.5rem 0.75rem;
    background: var(--error);
    color: white;
    font-size: 0.75rem;
    border-radius: 0.375rem;
  }

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

  .title-hint {
    font-weight: 400;
    font-size: 0.75rem;
    color: var(--text-tertiary);
  }
</style>
