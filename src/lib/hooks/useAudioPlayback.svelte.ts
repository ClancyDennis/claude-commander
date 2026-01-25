/**
 * Audio Playback Hook
 *
 * Provides reactive state for audio playback using Web Audio API.
 * Plays 24kHz PCM16 audio from OpenAI Realtime API.
 *
 * Usage:
 *   const audio = useAudioPlayback();
 *   audio.init();
 *   audio.playChunk(base64Audio);
 *   audio.stop();
 */

export interface AudioPlaybackState {
  isPlaying: boolean;
  isMuted: boolean;
  volume: number;
  error: string | null;
}

const initialState: AudioPlaybackState = {
  isPlaying: false,
  isMuted: false,
  volume: 1.0,
  error: null,
};

export function useAudioPlayback() {
  let state = $state<AudioPlaybackState>({ ...initialState });

  let audioContext: AudioContext | null = null;
  let gainNode: GainNode | null = null;

  // Queue for buffering audio chunks for smooth playback
  let audioQueue: AudioBuffer[] = [];
  let isProcessingQueue = false;
  let nextPlayTime = 0;

  /**
   * Initialize the audio context (must be called after user interaction)
   */
  function init(): void {
    if (audioContext) return;

    try {
      const AudioContextClass =
        window.AudioContext ||
        (window as unknown as { webkitAudioContext: typeof AudioContext })
          .webkitAudioContext;

      audioContext = new AudioContextClass({ sampleRate: 24000 });
      gainNode = audioContext.createGain();
      gainNode.gain.value = state.volume;
      gainNode.connect(audioContext.destination);

      console.log("[AudioPlayback] Initialized (24kHz)");
    } catch (err) {
      state.error =
        err instanceof Error ? err.message : "Failed to initialize audio";
      console.error("[AudioPlayback] Init error:", err);
    }
  }

  /**
   * Convert base64-encoded PCM16 to Float32Array
   */
  function base64ToPcm16Float(base64: string): Float32Array {
    const binary = window.atob(base64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i);
    }

    // Convert to Int16Array then normalize to Float32
    const int16 = new Int16Array(bytes.buffer);
    const float32 = new Float32Array(int16.length);
    for (let i = 0; i < int16.length; i++) {
      float32[i] = int16[i] / 32768;
    }

    return float32;
  }

  /**
   * Process the audio queue and schedule playback
   */
  function processQueue(): void {
    if (!audioContext || !gainNode || isProcessingQueue) return;
    if (audioQueue.length === 0) {
      state.isPlaying = false;
      return;
    }

    isProcessingQueue = true;
    state.isPlaying = true;

    while (audioQueue.length > 0) {
      const buffer = audioQueue.shift()!;
      const source = audioContext.createBufferSource();
      source.buffer = buffer;
      source.connect(gainNode);

      // Schedule playback
      const currentTime = audioContext.currentTime;
      const startTime = Math.max(currentTime, nextPlayTime);
      source.start(startTime);

      // Update next play time
      nextPlayTime = startTime + buffer.duration;
    }

    isProcessingQueue = false;
  }

  /**
   * Play a base64-encoded PCM16 audio chunk
   */
  function playChunk(base64Audio: string): void {
    if (!audioContext || !gainNode) {
      console.warn("[AudioPlayback] Not initialized, call init() first");
      return;
    }

    if (state.isMuted) return;

    try {
      const pcmData = base64ToPcm16Float(base64Audio);

      // Create audio buffer
      const buffer = audioContext.createBuffer(1, pcmData.length, 24000);
      buffer.copyToChannel(pcmData, 0);

      // Add to queue
      audioQueue.push(buffer);

      // Process queue
      processQueue();
    } catch (err) {
      console.error("[AudioPlayback] Error playing chunk:", err);
    }
  }

  /**
   * Stop all audio playback
   */
  function stop(): void {
    audioQueue = [];
    nextPlayTime = 0;
    state.isPlaying = false;
    console.log("[AudioPlayback] Stopped");
  }

  /**
   * Set volume (0.0 to 1.0)
   */
  function setVolume(volume: number): void {
    state.volume = Math.max(0, Math.min(1, volume));
    if (gainNode) {
      gainNode.gain.value = state.isMuted ? 0 : state.volume;
    }
  }

  /**
   * Toggle mute
   */
  function toggleMute(): void {
    state.isMuted = !state.isMuted;
    if (gainNode) {
      gainNode.gain.value = state.isMuted ? 0 : state.volume;
    }
    console.log(`[AudioPlayback] ${state.isMuted ? "Muted" : "Unmuted"}`);
  }

  /**
   * Set mute state
   */
  function setMuted(muted: boolean): void {
    state.isMuted = muted;
    if (gainNode) {
      gainNode.gain.value = state.isMuted ? 0 : state.volume;
    }
  }

  /**
   * Resume audio context if suspended (needed after user interaction)
   */
  async function resume(): Promise<void> {
    if (audioContext && audioContext.state === "suspended") {
      await audioContext.resume();
      console.log("[AudioPlayback] Context resumed");
    }
  }

  /**
   * Cleanup resources
   */
  function cleanup(): void {
    stop();
    if (audioContext) {
      audioContext.close();
      audioContext = null;
      gainNode = null;
    }
    console.log("[AudioPlayback] Cleaned up");
  }

  /**
   * Reset state
   */
  function reset(): void {
    cleanup();
    state = { ...initialState };
  }

  return {
    get state() {
      return state;
    },
    init,
    playChunk,
    stop,
    setVolume,
    toggleMute,
    setMuted,
    resume,
    cleanup,
    reset,
  };
}
