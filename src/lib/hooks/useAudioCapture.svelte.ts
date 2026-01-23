/**
 * Audio Capture Hook
 *
 * Provides reactive state for audio capture using Web Audio API.
 * Captures 24kHz PCM16 audio for OpenAI Realtime API.
 *
 * Usage:
 *   const audio = useAudioCapture();
 *   await audio.start((chunk) => sendAudioChunk(chunk));
 *   audio.stop();
 */

import { sendAudioChunk } from "$lib/stores/voice";

export interface AudioCaptureState {
  isRecording: boolean;
  isPaused: boolean;
  error: string | null;
}

const initialState: AudioCaptureState = {
  isRecording: false,
  isPaused: false,
  error: null,
};

export function useAudioCapture() {
  let state = $state<AudioCaptureState>({ ...initialState });

  let audioContext: AudioContext | null = null;
  let processor: ScriptProcessorNode | null = null;
  let source: MediaStreamAudioSourceNode | null = null;
  let stream: MediaStream | null = null;
  let isPausedRef = false;

  /**
   * Convert Float32Array to base64-encoded PCM16
   */
  function floatTo16BitPCM(input: Float32Array): Int16Array {
    const output = new Int16Array(input.length);
    for (let i = 0; i < input.length; i++) {
      const s = Math.max(-1, Math.min(1, input[i]));
      output[i] = s < 0 ? s * 0x8000 : s * 0x7fff;
    }
    return output;
  }

  function arrayBufferToBase64(buffer: ArrayBuffer): string {
    let binary = "";
    const bytes = new Uint8Array(buffer);
    const len = bytes.byteLength;
    for (let i = 0; i < len; i++) {
      binary += String.fromCharCode(bytes[i]);
    }
    return window.btoa(binary);
  }

  /**
   * Start audio capture
   * @param onAudioChunk - Callback for each audio chunk (base64 PCM16)
   */
  async function start(
    onAudioChunk?: (chunk: string) => void
  ): Promise<void> {
    try {
      state.error = null;

      // Request microphone permission with audio enhancements
      stream = await navigator.mediaDevices.getUserMedia({
        audio: {
          echoCancellation: true,
          noiseSuppression: true,
          autoGainControl: true,
          sampleRate: 24000,
        },
      });

      // Create AudioContext at 24kHz for OpenAI Realtime API
      const AudioContextClass =
        window.AudioContext ||
        (window as unknown as { webkitAudioContext: typeof AudioContext })
          .webkitAudioContext;
      audioContext = new AudioContextClass({ sampleRate: 24000 });

      source = audioContext.createMediaStreamSource(stream);

      // Use ScriptProcessor for raw PCM access (buffer size 4096 = ~170ms at 24kHz)
      processor = audioContext.createScriptProcessor(4096, 1, 1);

      processor.onaudioprocess = (e) => {
        if (isPausedRef) return;

        const inputData = e.inputBuffer.getChannelData(0);
        const pcmData = floatTo16BitPCM(inputData);
        const base64Data = arrayBufferToBase64(pcmData.buffer);

        if (onAudioChunk) {
          onAudioChunk(base64Data);
        } else {
          // Default: send to Rust backend via store
          sendAudioChunk(base64Data);
        }
      };

      source.connect(processor);

      // Connect to a silent gain node to satisfy the API requirement
      // that processor must be connected to destination
      const silentGain = audioContext.createGain();
      silentGain.gain.value = 0;
      processor.connect(silentGain);
      silentGain.connect(audioContext.destination);

      state.isRecording = true;
      console.log("[AudioCapture] Recording started (PCM16 24kHz)");
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to access microphone";

      // Provide user-friendly error messages
      if (
        errorMessage.includes("Permission denied") ||
        errorMessage.includes("NotAllowedError")
      ) {
        state.error =
          "Microphone permission denied. Please allow microphone access.";
      } else if (errorMessage.includes("NotFoundError")) {
        state.error =
          "No microphone found. Please connect a microphone and try again.";
      } else {
        state.error =
          "Failed to start recording. Please check your microphone.";
      }

      console.error("[AudioCapture] Error starting recording:", err);
    }
  }

  /**
   * Stop audio capture
   */
  function stop(): void {
    if (!state.isRecording) return;

    // Cleanup audio nodes
    if (processor && source) {
      source.disconnect();
      processor.disconnect();
      processor.onaudioprocess = null;
    }

    if (audioContext) {
      audioContext.close();
      audioContext = null;
    }

    // Stop all tracks
    if (stream) {
      stream.getTracks().forEach((track) => track.stop());
      stream = null;
    }

    processor = null;
    source = null;

    state.isRecording = false;
    state.isPaused = false;
    console.log("[AudioCapture] Recording stopped");
  }

  /**
   * Pause audio capture
   */
  function pause(): void {
    if (state.isRecording && !state.isPaused) {
      state.isPaused = true;
      isPausedRef = true;
      if (audioContext) {
        audioContext.suspend();
      }
      console.log("[AudioCapture] Recording paused");
    }
  }

  /**
   * Resume audio capture
   */
  function resume(): void {
    if (state.isRecording && state.isPaused) {
      state.isPaused = false;
      isPausedRef = false;
      if (audioContext) {
        audioContext.resume();
      }
      console.log("[AudioCapture] Recording resumed");
    }
  }

  /**
   * Reset state and cleanup
   */
  function reset(): void {
    stop();
    state = { ...initialState };
  }

  return {
    get state() {
      return state;
    },
    start,
    stop,
    pause,
    resume,
    reset,
  };
}
