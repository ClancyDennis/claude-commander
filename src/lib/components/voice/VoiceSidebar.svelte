<script lang="ts">
  import { onMount } from "svelte";
  import * as Sheet from "$lib/components/ui/sheet";
  import { Button } from "$lib/components/ui/button";
  import { IconButton } from "$lib/components/ui/button";
  import {
    Mic,
    MicOff,
    Loader2,
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

  // Sub-components
  import ModeSelector from "./ModeSelector.svelte";
  import DictateModePanel from "./DictateModePanel.svelte";
  import DiscussModePanel from "./DiscussModePanel.svelte";
  import AudioControls from "./AudioControls.svelte";

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

  // Derived state
  const isDictateActive = $derived($voiceState.isRecording);
  const isDiscussActive = $derived($discussState.isActive);
  const isDiscussConnecting = $derived($discussState.isConnecting || isDiscussStarting);
  const isMuted = $derived(audioPlayback.state.isMuted);
  const discussMessages = $derived($discussState.messages);
  const currentUserTranscript = $derived($discussState.currentUserTranscript);
  const currentAssistantResponse = $derived($discussState.currentAssistantResponse);
  const selectedCount = $derived(discussMessages.filter((m) => m.selected).length);

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

  async function handleEndSession() {
    if (isDictateActive) {
      await handleStopDictate();
    }
    if (isDiscussActive) {
      await handleStopDiscuss();
    }
  }

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
        <AudioControls {isMuted} onToggleMute={handleMuteToggle} />
      {/if}
    </Sheet.Header>

    <div class="flex-1 overflow-y-auto mt-4">
      {#if mode === "select"}
        <ModeSelector
          onSelectDictate={handleStartDictate}
          onSelectDiscuss={handleStartDiscuss}
        />
      {:else if mode === "dictate"}
        <DictateModePanel
          transcript={$voiceState.transcript}
          isActive={isDictateActive}
          error={$voiceState.error}
        />
      {:else if mode === "discuss"}
        <DiscussModePanel
          messages={discussMessages}
          {currentUserTranscript}
          {currentAssistantResponse}
          isActive={isDiscussActive}
          isConnecting={isDiscussConnecting}
          error={$discussState.error}
          onMessageClick={handleMessageClick}
        />
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
  .title-hint {
    font-weight: 400;
    font-size: 0.75rem;
    color: var(--text-tertiary);
  }
</style>
