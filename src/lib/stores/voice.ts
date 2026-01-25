/**
 * Voice Store Module
 *
 * Manages voice recording state and transcription for the OpenAI Realtime API integration.
 */

import { writable, derived, get } from "svelte/store";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

// Types
export interface VoiceTranscriptEvent {
  transcript: string;
}

export interface VoiceResponseEvent {
  delta: string;
}

export interface VoiceAudioEvent {
  audio: string;
}

export interface VoiceStatus {
  is_active: boolean;
  transcript: string;
}

export type VoiceMode = "dictate" | "discuss" | "attention" | "off";

export interface VoiceState {
  isRecording: boolean;
  isConnecting: boolean;
  transcript: string;
  segments: string[];
  response: string;
  error: string | null;
  mode: VoiceMode;
}

// Stores
export const voiceState = writable<VoiceState>({
  isRecording: false,
  isConnecting: false,
  transcript: "",
  segments: [],
  response: "",
  error: null,
  mode: "off",
});

// Store for controlling voice sidebar visibility from anywhere
export const voiceSidebarOpen = writable(false);

// Audio playback callback - set by component using useAudioPlayback hook
let audioPlaybackCallback: ((audio: string) => void) | null = null;

export function setAudioPlaybackCallback(callback: ((audio: string) => void) | null): void {
  audioPlaybackCallback = callback;
}

// Store for pending chat input (text to insert into chat input)
export const pendingChatInput = writable<string | null>(null);

// Insert transcript into chat input
export function insertTranscriptToChat(): void {
  const state = get(voiceState);
  if (state.transcript) {
    pendingChatInput.set(state.transcript);
    // Clear the voice transcript after inserting
    voiceState.update((s) => ({ ...s, transcript: "", segments: [] }));
  }
}

// Derived stores
export const isVoiceActive = derived(voiceState, ($state) => $state.isRecording);
export const voiceTranscript = derived(voiceState, ($state) => $state.transcript);
export const voiceResponse = derived(voiceState, ($state) => $state.response);
export const voiceError = derived(voiceState, ($state) => $state.error);
export const voiceMode = derived(voiceState, ($state) => $state.mode);

// Set voice mode
export function setVoiceMode(mode: VoiceMode): void {
  voiceState.update((s) => ({ ...s, mode }));
}

// Event listeners (initialized once)
let listenersInitialized = false;

export async function initVoiceListeners() {
  if (listenersInitialized) return;
  listenersInitialized = true;

  // Listen for transcript events from Rust backend (user's speech)
  await listen<VoiceTranscriptEvent>("voice:transcript", (event) => {
    const { transcript } = event.payload;
    voiceState.update((state) => ({
      ...state,
      transcript: state.transcript
        ? `${state.transcript} ${transcript}`
        : transcript,
      segments: [...state.segments, transcript],
    }));
  });

  // Listen for response events from the model
  await listen<VoiceResponseEvent>("voice:response", (event) => {
    const { delta } = event.payload;
    voiceState.update((state) => ({
      ...state,
      response: state.response + delta,
    }));
  });

  // Listen for status events
  await listen<VoiceStatus>("voice:status", (event) => {
    const { is_active, transcript } = event.payload;
    voiceState.update((state) => ({
      ...state,
      isRecording: is_active,
      isConnecting: false,
    }));
  });

  // Listen for audio events from the model (for playback)
  await listen<VoiceAudioEvent>("voice:audio", (event) => {
    const { audio } = event.payload;
    if (audioPlaybackCallback && audio) {
      audioPlaybackCallback(audio);
    }
  });
}

// Actions
export async function startVoiceSession(): Promise<void> {
  const state = get(voiceState);
  if (state.isRecording || state.isConnecting) {
    return;
  }

  voiceState.update((s) => ({
    ...s,
    isConnecting: true,
    error: null,
    transcript: "",
    segments: [],
    response: "",
  }));

  try {
    await invoke("start_voice_session");
    voiceState.update((s) => ({
      ...s,
      isRecording: true,
      isConnecting: false,
    }));
  } catch (error) {
    voiceState.update((s) => ({
      ...s,
      isConnecting: false,
      error: error instanceof Error ? error.message : String(error),
    }));
  }
}

export async function stopVoiceSession(): Promise<string> {
  const state = get(voiceState);
  if (!state.isRecording) {
    return state.transcript;
  }

  try {
    const finalTranscript = await invoke<string>("stop_voice_session");
    voiceState.update((s) => ({
      ...s,
      isRecording: false,
      transcript: finalTranscript || s.transcript,
    }));
    return finalTranscript || state.transcript;
  } catch (error) {
    voiceState.update((s) => ({
      ...s,
      isRecording: false,
      error: error instanceof Error ? error.message : String(error),
    }));
    return state.transcript;
  }
}

export async function sendAudioChunk(data: string): Promise<void> {
  const state = get(voiceState);
  if (!state.isRecording) {
    return;
  }

  try {
    await invoke("send_voice_audio", { data });
  } catch (error) {
    console.error("[Voice] Failed to send audio chunk:", error);
  }
}

export function clearVoiceState(): void {
  voiceState.set({
    isRecording: false,
    isConnecting: false,
    transcript: "",
    segments: [],
    response: "",
    error: null,
    mode: "off",
  });
}

export async function getVoiceStatus(): Promise<VoiceStatus> {
  return await invoke<VoiceStatus>("get_voice_status");
}

// ============================================================================
// Discuss Mode
// ============================================================================

export interface DiscussMessage {
  id: string;
  role: "user" | "assistant";
  content: string;
  timestamp: number;
  selected?: boolean;
}

export interface DiscussState {
  isActive: boolean;
  isConnecting: boolean;
  messages: DiscussMessage[];
  currentUserTranscript: string;
  currentAssistantResponse: string;
  error: string | null;
}

export interface DiscussToolCallEvent {
  name: string;
  call_id: string;
  args: string;
}

export interface DiscussMissionControlEvent {
  message: string;
  response: string;
}

export const discussState = writable<DiscussState>({
  isActive: false,
  isConnecting: false,
  messages: [],
  currentUserTranscript: "",
  currentAssistantResponse: "",
  error: null,
});

// Attention state (defined here so isAnyVoiceActive can reference it)
export interface AttentionState {
  isActive: boolean;
  isConnecting: boolean;
  agentId: string | null;
  agentTitle: string;
  transcript: string;
  response: string;
  error: string | null;
}

export const attentionState = writable<AttentionState>({
  isActive: false,
  isConnecting: false,
  agentId: null,
  agentTitle: "",
  transcript: "",
  response: "",
  error: null,
});

// Derived store: is any voice session active?
export const isAnyVoiceActive = derived(
  [voiceState, discussState, attentionState],
  ([$voice, $discuss, $attention]) => $voice.isRecording || $discuss.isActive || $attention.isActive
);

// Callback for handling mission control messages
let missionControlCallback: ((message: string) => Promise<string>) | null = null;

export function setMissionControlCallback(callback: ((message: string) => Promise<string>) | null): void {
  missionControlCallback = callback;
}

export function getMissionControlCallback(): ((message: string) => Promise<string>) | null {
  return missionControlCallback;
}

// Discuss mode audio callback (separate from dictate mode)
let discussAudioCallback: ((audio: string) => void) | null = null;

export function setDiscussAudioCallback(callback: ((audio: string) => void) | null): void {
  discussAudioCallback = callback;
}

let discussListenersInitialized = false;

// Helper to generate unique message IDs
function generateMessageId(): string {
  return `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
}

// Commit current user transcript to messages
export function commitUserMessage(): void {
  discussState.update((state) => {
    if (!state.currentUserTranscript.trim()) return state;

    const newMessage: DiscussMessage = {
      id: generateMessageId(),
      role: "user",
      content: state.currentUserTranscript.trim(),
      timestamp: Date.now(),
    };

    return {
      ...state,
      messages: [...state.messages, newMessage],
      currentUserTranscript: "",
    };
  });
}

// Commit current assistant response to messages
// Always commits any pending user message first to ensure proper ordering
export function commitAssistantMessage(): void {
  discussState.update((state) => {
    if (!state.currentAssistantResponse.trim()) return state;

    let messages = [...state.messages];
    let currentUserTranscript = state.currentUserTranscript;

    // First, commit any pending user message to ensure proper ordering
    if (currentUserTranscript.trim()) {
      const userMessage: DiscussMessage = {
        id: generateMessageId(),
        role: "user",
        content: currentUserTranscript.trim(),
        timestamp: Date.now(),
      };
      messages = [...messages, userMessage];
      currentUserTranscript = "";
    }

    // Then commit the assistant message
    const assistantMessage: DiscussMessage = {
      id: generateMessageId(),
      role: "assistant",
      content: state.currentAssistantResponse.trim(),
      timestamp: Date.now(),
    };

    return {
      ...state,
      messages: [...messages, assistantMessage],
      currentUserTranscript,
      currentAssistantResponse: "",
    };
  });
}

// Toggle message selection
export function toggleMessageSelection(messageId: string): void {
  discussState.update((state) => ({
    ...state,
    messages: state.messages.map((msg) =>
      msg.id === messageId ? { ...msg, selected: !msg.selected } : msg
    ),
  }));
}

// Get selected messages
export function getSelectedMessages(): DiscussMessage[] {
  return get(discussState).messages.filter((msg) => msg.selected);
}

// Clear all selections
export function clearMessageSelections(): void {
  discussState.update((state) => ({
    ...state,
    messages: state.messages.map((msg) => ({ ...msg, selected: false })),
  }));
}

export async function initDiscussListeners() {
  if (discussListenersInitialized) return;
  discussListenersInitialized = true;

  // Listen for discuss transcript events (user speech, accumulates)
  await listen<VoiceTranscriptEvent>("discuss:transcript", (event) => {
    const { transcript } = event.payload;
    discussState.update((state) => ({
      ...state,
      currentUserTranscript: state.currentUserTranscript
        ? `${state.currentUserTranscript} ${transcript}`
        : transcript,
    }));
  });

  // Listen for user turn complete - commit user message
  await listen("discuss:user_turn_complete", () => {
    commitUserMessage();
  });

  // Listen for discuss response events (AI response, accumulates)
  await listen<VoiceResponseEvent>("discuss:response", (event) => {
    const { delta } = event.payload;
    discussState.update((state) => ({
      ...state,
      currentAssistantResponse: state.currentAssistantResponse + delta,
    }));
  });

  // Listen for assistant turn complete - commit assistant message
  await listen("discuss:assistant_turn_complete", () => {
    commitAssistantMessage();
  });

  // Listen for discuss status events
  await listen<VoiceStatus>("discuss:status", (event) => {
    const { is_active } = event.payload;
    discussState.update((state) => ({
      ...state,
      isActive: is_active,
      isConnecting: false,
    }));
  });

  // Listen for discuss audio events
  await listen<VoiceAudioEvent>("discuss:audio", (event) => {
    const { audio } = event.payload;
    if (discussAudioCallback && audio) {
      discussAudioCallback(audio);
    }
  });

  // Listen for mission control response events
  await listen<DiscussMissionControlEvent>("discuss:mission_control", (event) => {
    console.log("[Discuss] Mission control response:", event.payload);
  });
}

export async function startDiscussSession(): Promise<void> {
  const state = get(discussState);
  if (state.isActive || state.isConnecting) {
    return;
  }

  discussState.update((s) => ({
    ...s,
    isConnecting: true,
    error: null,
    currentUserTranscript: "",
    currentAssistantResponse: "",
    // Keep existing messages for history, clear on explicit user action
  }));

  try {
    await invoke("start_discuss_session");
    discussState.update((s) => ({
      ...s,
      isActive: true,
      isConnecting: false,
    }));
    setVoiceMode("discuss");
  } catch (error) {
    discussState.update((s) => ({
      ...s,
      isConnecting: false,
      error: error instanceof Error ? error.message : String(error),
    }));
  }
}

export async function stopDiscussSession(): Promise<void> {
  const state = get(discussState);
  if (!state.isActive) {
    return;
  }

  try {
    await invoke("stop_discuss_session");
    discussState.update((s) => ({
      ...s,
      isActive: false,
    }));
    setVoiceMode("off");
  } catch (error) {
    discussState.update((s) => ({
      ...s,
      isActive: false,
      error: error instanceof Error ? error.message : String(error),
    }));
  }
}

export async function sendDiscussAudio(data: string): Promise<void> {
  const state = get(discussState);
  if (!state.isActive) {
    return;
  }

  try {
    await invoke("send_discuss_audio", { data });
  } catch (error) {
    console.error("[Discuss] Failed to send audio chunk:", error);
  }
}

export function clearDiscussState(): void {
  discussState.set({
    isActive: false,
    isConnecting: false,
    messages: [],
    currentUserTranscript: "",
    currentAssistantResponse: "",
    error: null,
  });
}

// ============================================================================
// Attention Mode Functions
// ============================================================================

// Global attention mode setting (persisted to localStorage)
const ATTENTION_ENABLED_KEY = "attention_mode_enabled";

function getInitialAttentionEnabled(): boolean {
  if (typeof window === "undefined") return true;
  const stored = localStorage.getItem(ATTENTION_ENABLED_KEY);
  return stored !== "false"; // Default: enabled
}

export const attentionEnabled = writable<boolean>(getInitialAttentionEnabled());

// Persist to localStorage on change
if (typeof window !== "undefined") {
  attentionEnabled.subscribe((enabled) => {
    localStorage.setItem(ATTENTION_ENABLED_KEY, String(enabled));
  });
}

// Attention mode audio callback
let attentionAudioCallback: ((audio: string) => void) | null = null;

export function setAttentionAudioCallback(callback: ((audio: string) => void) | null): void {
  attentionAudioCallback = callback;
}

let attentionListenersInitialized = false;

export async function initAttentionListeners() {
  if (attentionListenersInitialized) return;
  attentionListenersInitialized = true;

  // Listen for attention transcript events (user speech)
  await listen<VoiceTranscriptEvent>("attention:transcript", (event) => {
    const { transcript } = event.payload;
    attentionState.update((state) => ({
      ...state,
      transcript: state.transcript ? `${state.transcript} ${transcript}` : transcript,
    }));
  });

  // Listen for attention response events (AI response)
  await listen<VoiceResponseEvent>("attention:response", (event) => {
    const { delta } = event.payload;
    attentionState.update((state) => ({
      ...state,
      response: state.response + delta,
    }));
  });

  // Listen for attention status events
  await listen<VoiceStatus>("attention:status", (event) => {
    const { is_active } = event.payload;
    attentionState.update((state) => ({
      ...state,
      isActive: is_active,
      isConnecting: false,
    }));
    if (is_active) {
      setVoiceMode("attention");
    } else {
      setVoiceMode("off");
    }
  });

  // Listen for attention audio events
  await listen<VoiceAudioEvent>("attention:audio", (event) => {
    const { audio } = event.payload;
    if (attentionAudioCallback && audio) {
      attentionAudioCallback(audio);
    }
  });

  // Listen for attention timeout events
  await listen<{ agent_id: string }>("attention:timeout", (event) => {
    console.log("[Attention] Session timed out for agent:", event.payload.agent_id);
    // The backend already sets isActive to false, but we update state to be safe
    attentionState.update((state) => ({
      ...state,
      isActive: false,
    }));
    setVoiceMode("off");
  });
}

export async function startAttentionSession(
  agentId: string,
  agentTitle: string,
  summary: string
): Promise<void> {
  // Check if attention mode is enabled
  const enabled = get(attentionEnabled);
  if (!enabled) {
    console.log("[Attention] Attention mode disabled, skipping");
    return;
  }

  const state = get(attentionState);
  if (state.isActive || state.isConnecting) {
    console.log("[Attention] Session already active, skipping");
    return;
  }

  // Also check if discuss or dictate is active
  const discuss = get(discussState);
  const voice = get(voiceState);
  if (discuss.isActive || voice.isRecording) {
    console.log("[Attention] Another voice session active, skipping");
    return;
  }

  attentionState.update((s) => ({
    ...s,
    isConnecting: true,
    error: null,
    agentId,
    agentTitle,
    transcript: "",
    response: "",
  }));

  try {
    await invoke("start_attention_session", { agentId, agentTitle, summary });
    attentionState.update((s) => ({
      ...s,
      isActive: true,
      isConnecting: false,
    }));
    setVoiceMode("attention");
  } catch (error) {
    console.error("[Attention] Failed to start session:", error);
    attentionState.update((s) => ({
      ...s,
      isConnecting: false,
      error: error instanceof Error ? error.message : String(error),
    }));
  }
}

export async function stopAttentionSession(): Promise<void> {
  const state = get(attentionState);
  if (!state.isActive) {
    return;
  }

  try {
    await invoke("stop_attention_session");
    attentionState.update((s) => ({
      ...s,
      isActive: false,
    }));
    setVoiceMode("off");
  } catch (error) {
    attentionState.update((s) => ({
      ...s,
      isActive: false,
      error: error instanceof Error ? error.message : String(error),
    }));
  }
}

export async function sendAttentionAudio(data: string): Promise<void> {
  const state = get(attentionState);
  if (!state.isActive) {
    return;
  }

  try {
    await invoke("send_attention_audio", { data });
  } catch (error) {
    console.error("[Attention] Failed to send audio chunk:", error);
  }
}

export function clearAttentionState(): void {
  attentionState.set({
    isActive: false,
    isConnecting: false,
    agentId: null,
    agentTitle: "",
    transcript: "",
    response: "",
    error: null,
  });
}
