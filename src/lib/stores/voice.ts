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

export interface VoiceStatus {
  is_active: boolean;
  transcript: string;
}

export interface VoiceState {
  isRecording: boolean;
  isConnecting: boolean;
  transcript: string;
  segments: string[];
  error: string | null;
}

// Stores
export const voiceState = writable<VoiceState>({
  isRecording: false,
  isConnecting: false,
  transcript: "",
  segments: [],
  error: null,
});

// Derived stores
export const isVoiceActive = derived(voiceState, ($state) => $state.isRecording);
export const voiceTranscript = derived(voiceState, ($state) => $state.transcript);
export const voiceError = derived(voiceState, ($state) => $state.error);

// Event listeners (initialized once)
let listenersInitialized = false;

export async function initVoiceListeners() {
  if (listenersInitialized) return;
  listenersInitialized = true;

  // Listen for transcript events from Rust backend
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

  // Listen for status events
  await listen<VoiceStatus>("voice:status", (event) => {
    const { is_active, transcript } = event.payload;
    voiceState.update((state) => ({
      ...state,
      isRecording: is_active,
      isConnecting: false,
    }));
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
    error: null,
  });
}

export async function getVoiceStatus(): Promise<VoiceStatus> {
  return await invoke<VoiceStatus>("get_voice_status");
}
