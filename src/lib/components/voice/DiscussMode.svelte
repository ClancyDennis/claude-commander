<script lang="ts">
  import { onMount, tick } from "svelte";
  import { X, MessageCircle, Mic, MicOff, Volume2, VolumeX, Loader2, Check, Send } from "$lib/components/ui/icons";
  import { IconButton, Button } from "$lib/components/ui/button";
  import {
    discussState,
    startDiscussSession,
    stopDiscussSession,
    sendDiscussAudio,
    initDiscussListeners,
    setDiscussAudioCallback,
    setMissionControlCallback,
    clearDiscussState,
    toggleMessageSelection,
    getSelectedMessages,
    clearMessageSelections,
    type DiscussMessage,
  } from "$lib/stores/voice";
  import { pendingChatInput } from "$lib/stores/voice";
  import { useAudioCapture } from "$lib/hooks/useAudioCapture.svelte";
  import { useAudioPlayback } from "$lib/hooks/useAudioPlayback.svelte";

  interface Props {
    onClose?: () => void;
    onSendToChat?: (text: string) => void;
  }

  let { onClose, onSendToChat }: Props = $props();

  const audioCapture = useAudioCapture();
  const audioPlayback = useAudioPlayback();

  let isStarting = $state(false);
  let conversationEl: HTMLDivElement | null = $state(null);

  const isActive = $derived($discussState.isActive);
  const isConnecting = $derived($discussState.isConnecting || isStarting);
  const isMuted = $derived(audioPlayback.state.isMuted);
  const messages = $derived($discussState.messages);
  const currentUserTranscript = $derived($discussState.currentUserTranscript);
  const currentAssistantResponse = $derived($discussState.currentAssistantResponse);
  const hasMessages = $derived(messages.length > 0 || currentUserTranscript.length > 0 || currentAssistantResponse.length > 0);
  const selectedCount = $derived(messages.filter(m => m.selected).length);

  // Auto-scroll to bottom when new messages arrive
  $effect(() => {
    if (messages.length > 0 || currentUserTranscript || currentAssistantResponse) {
      tick().then(() => {
        if (conversationEl) {
          conversationEl.scrollTop = conversationEl.scrollHeight;
        }
      });
    }
  });

  onMount(() => {
    initDiscussListeners();

    // Set up audio playback callback
    setDiscussAudioCallback((audio) => {
      audioPlayback.playChunk(audio);
    });

    // Set up mission control callback (will be called by backend)
    setMissionControlCallback(async (message: string) => {
      // This is where we'd call mission control
      // For now, just log it - the actual implementation will route through chat
      console.log("[Discuss] Mission control request:", message);
      return "Message received by mission control";
    });

    return () => {
      setDiscussAudioCallback(null);
      setMissionControlCallback(null);
      audioPlayback.cleanup();
    };
  });

  async function handleStartDiscuss() {
    if (isActive || isConnecting) return;

    isStarting = true;
    try {
      // Initialize audio playback (requires user interaction)
      audioPlayback.init();
      await audioPlayback.resume();

      await startDiscussSession();

      // Start audio capture and route to discuss endpoint
      await audioCapture.start((chunk) => {
        sendDiscussAudio(chunk);
      });
    } finally {
      isStarting = false;
    }
  }

  async function handleStopDiscuss() {
    audioCapture.stop();
    audioPlayback.stop();
    await stopDiscussSession();
  }

  function handleClose() {
    if (isActive) {
      handleStopDiscuss();
    }
    // Don't clear state on close - let user see the transcript
    onClose?.();
  }

  function handleClearHistory() {
    clearDiscussState();
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

    // Format selected messages for chat
    const text = selected
      .map(m => `[${m.role === 'user' ? 'You' : 'AI'}]: ${m.content}`)
      .join('\n\n');

    // Send to chat via callback or pending input
    if (onSendToChat) {
      onSendToChat(text);
    } else {
      pendingChatInput.set(text);
    }

    clearMessageSelections();
  }
</script>

<div class="discuss-mode">
  <div class="discuss-header">
    <div class="header-left">
      <MessageCircle size={18} />
      <span class="title">Discuss Mode</span>
      {#if isActive}
        <span class="status-badge active">Active</span>
      {:else if isConnecting}
        <span class="status-badge connecting">Connecting...</span>
      {/if}
    </div>
    <div class="header-actions">
      {#if selectedCount > 0}
        <Button size="sm" onclick={handleSendSelected} title="Send selected messages to chat">
          <Send size={14} />
          Send ({selectedCount})
        </Button>
      {/if}
      {#if isActive}
        <IconButton
          icon={isMuted ? VolumeX : Volume2}
          variant="ghost"
          size="sm"
          onclick={handleMuteToggle}
          title={isMuted ? "Unmute AI voice" : "Mute AI voice"}
        />
      {/if}
      {#if messages.length > 0 && !isActive}
        <IconButton
          icon={X}
          variant="ghost"
          size="sm"
          onclick={handleClearHistory}
          title="Clear conversation history"
        />
      {/if}
      <IconButton
        icon={X}
        variant="ghost"
        size="sm"
        onclick={handleClose}
        title="Close discuss mode"
      />
    </div>
  </div>

  <div class="discuss-content" bind:this={conversationEl}>
    {#if !hasMessages && !isActive && !isConnecting}
      <div class="welcome">
        <p class="welcome-text">
          Start a voice conversation to discuss your project ideas.
          The AI can talk to Mission Control to help plan and execute tasks.
        </p>
        <p class="tip">Click messages after to select them for the chat.</p>
        <Button onclick={handleStartDiscuss} disabled={isConnecting}>
          <Mic size={16} />
          Start Discussion
        </Button>
      </div>
    {:else}
      <div class="conversation">
        <!-- Past messages (committed) -->
        {#each messages as msg (msg.id)}
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
            <span class="label">{msg.role === 'user' ? 'You' : 'AI'}</span>
            <p class="text">{msg.content}</p>
          </button>
        {/each}

        <!-- Current user transcript (in progress) -->
        {#if currentUserTranscript}
          <div class="message user current">
            <span class="label">You</span>
            <p class="text">{currentUserTranscript}<span class="cursor">|</span></p>
          </div>
        {:else if isActive && messages.length === 0}
          <p class="listening">Listening...</p>
        {/if}

        <!-- Current assistant response (in progress) -->
        {#if currentAssistantResponse}
          <div class="message assistant current">
            <span class="label">AI</span>
            <p class="text">{currentAssistantResponse}<span class="cursor">|</span></p>
          </div>
        {/if}
      </div>

      <div class="discuss-footer">
        <div class="recording-indicator">
          {#if isActive}
            <span class="pulse"></span>
            <span>Recording</span>
          {:else if isConnecting}
            <Loader2 size={14} class="animate-spin" />
            <span>Connecting...</span>
          {:else if messages.length > 0}
            <span class="select-hint">Click messages to select for chat</span>
          {/if}
        </div>
        {#if isActive}
          <Button onclick={handleStopDiscuss} variant="secondary">
            <MicOff size={16} />
            Stop
          </Button>
        {:else}
          <Button onclick={handleStartDiscuss} disabled={isConnecting}>
            <Mic size={16} />
            {messages.length > 0 ? 'Continue' : 'Start'}
          </Button>
        {/if}
      </div>
    {/if}
  </div>

  {#if $discussState.error}
    <div class="error">
      {$discussState.error}
    </div>
  {/if}
</div>

<style>
  .discuss-mode {
    background: var(--bg-secondary);
    border: 1px solid var(--border-hex);
    border-radius: 0.5rem;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .discuss-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    background: var(--bg-tertiary);
    border-bottom: 1px solid var(--border-hex);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .title {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .status-badge {
    font-size: 0.625rem;
    padding: 0.125rem 0.375rem;
    border-radius: 9999px;
    text-transform: uppercase;
    font-weight: 600;
  }

  .status-badge.active {
    background: var(--success, #22c55e);
    color: white;
  }

  .status-badge.connecting {
    background: var(--warning, #f59e0b);
    color: white;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .discuss-content {
    flex: 1;
    padding: 1rem;
    min-height: 200px;
    max-height: 400px;
    overflow-y: auto;
  }

  .welcome {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    text-align: center;
    padding: 2rem 1rem;
  }

  .welcome-text {
    font-size: 0.875rem;
    color: var(--text-secondary);
    max-width: 300px;
    line-height: 1.5;
    margin: 0;
  }

  .tip {
    font-size: 0.75rem;
    color: var(--text-tertiary);
    margin: 0;
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
  }

  .message:hover {
    transform: translateX(2px);
  }

  .message.user {
    background: var(--bg-tertiary);
    align-self: flex-end;
    max-width: 85%;
  }

  .message.assistant {
    background: rgba(34, 197, 94, 0.1);
    border: 1px solid rgba(34, 197, 94, 0.2);
    align-self: flex-start;
    max-width: 85%;
  }

  .message.selected {
    outline: 2px solid var(--accent-hex);
    outline-offset: 2px;
  }

  .message.selected.user {
    background: rgba(var(--accent-rgb), 0.15);
  }

  .message.selected.assistant {
    background: rgba(var(--accent-rgb), 0.15);
    border-color: var(--accent-hex);
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
    0%, 100% { opacity: 1; }
    50% { opacity: 0; }
  }

  .listening {
    font-size: 0.875rem;
    color: var(--text-tertiary);
    font-style: italic;
    text-align: center;
    margin: 0;
  }

  .discuss-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    background: var(--bg-tertiary);
    border-top: 1px solid var(--border-hex);
  }

  .recording-indicator {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .select-hint {
    font-style: italic;
    color: var(--text-tertiary);
  }

  .pulse {
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

  .error {
    padding: 0.5rem 1rem;
    background: var(--error);
    color: white;
    font-size: 0.75rem;
  }
</style>
