/**
 * Commander Personality Store
 *
 * Manages personality settings for the meta-agent (System Commander).
 * These settings control how the commander:
 * - Communicates (tone, style, verbosity)
 * - Handles agents (strictness, autonomy)
 * - Prioritizes technologies and patterns
 * - Focuses on quality aspects
 *
 * Settings are persisted to localStorage and synced to the backend
 * where an LLM threads them into the system prompt.
 */

import { writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { attentionEnabled as voiceAttentionEnabled } from "./voice";

// Types
export interface CommanderPersonality {
  // Personality
  strictness: number; // 1-10 scale (1=lenient, 10=demanding)
  communicationStyle: "concise" | "balanced" | "verbose";
  tone: "professional" | "friendly" | "direct";

  // Tech preferences
  preferredLanguages: string[];
  preferredFrameworks: string[];
  patternsToFavor: string;
  patternsToAvoid: string;

  // Quality focus
  focusAreas: string[]; // ['type-safety', 'error-handling', 'performance', 'readability', 'test-coverage', 'security']

  // Agent handling
  autonomyLevel: number; // 1-10 scale (1=hand-holding, 10=full autonomy)

  // Voice settings
  attentionEnabled: boolean;
  listenTimeout: number; // seconds (5, 10, 15, 30)
  openaiVoice: "alloy" | "ash" | "ballad" | "coral" | "echo" | "sage" | "shimmer" | "verse";
  voicePersona: "professional" | "casual" | "brief";

  // Free-form
  customInstructions: string;
}

// Default settings - balanced, neutral starting point
export const defaultPersonality: CommanderPersonality = {
  strictness: 5,
  communicationStyle: "balanced",
  tone: "professional",

  preferredLanguages: [],
  preferredFrameworks: [],
  patternsToFavor: "",
  patternsToAvoid: "",

  focusAreas: [],

  autonomyLevel: 5,

  attentionEnabled: true,
  listenTimeout: 10,
  openaiVoice: "alloy",
  voicePersona: "professional",

  customInstructions: "",
};

// Available options for UI
export const focusAreaOptions = [
  { value: "type-safety", label: "Type Safety" },
  { value: "error-handling", label: "Error Handling" },
  { value: "performance", label: "Performance" },
  { value: "readability", label: "Readability" },
  { value: "test-coverage", label: "Test Coverage" },
  { value: "security", label: "Security" },
];

export const commonLanguages = [
  "TypeScript",
  "JavaScript",
  "Rust",
  "Python",
  "Go",
  "Java",
  "C++",
  "C#",
  "Ruby",
  "PHP",
  "Swift",
  "Kotlin",
];

export const commonFrameworks = [
  "Svelte",
  "React",
  "Vue",
  "Angular",
  "Tauri",
  "Electron",
  "Next.js",
  "Nuxt",
  "SvelteKit",
  "Express",
  "FastAPI",
  "Django",
  "Rails",
  "Spring",
];

// LocalStorage key
const STORAGE_KEY = "commander_personality";

// Load initial state from localStorage
function loadFromStorage(): CommanderPersonality {
  if (typeof window === "undefined") return defaultPersonality;

  const stored = localStorage.getItem(STORAGE_KEY);
  let settings = defaultPersonality;

  if (stored) {
    try {
      const parsed = JSON.parse(stored);
      // Merge with defaults to handle new fields
      settings = { ...defaultPersonality, ...parsed };
    } catch (e) {
      console.error("[CommanderPersonality] Failed to parse stored settings:", e);
    }
  }

  // Sync attentionEnabled from voice store (source of truth for backwards compatibility)
  settings.attentionEnabled = get(voiceAttentionEnabled);

  return settings;
}

// Create the store
export const commanderPersonality = writable<CommanderPersonality>(loadFromStorage());


// Note: We no longer auto-sync on every change.
// The updatePersonality() function handles explicit saves with backend sync.
// Backend loads cached prompt on startup - no need to sync from frontend.

// Update personality and sync to backend (returns Promise for UI feedback)
export async function updatePersonality(updates: Partial<CommanderPersonality>): Promise<void> {
  const newSettings = { ...get(commanderPersonality), ...updates };

  // Save to localStorage immediately
  localStorage.setItem(STORAGE_KEY, JSON.stringify(newSettings));

  // Sync attentionEnabled to voice store if changed
  if (newSettings.attentionEnabled !== get(voiceAttentionEnabled)) {
    voiceAttentionEnabled.set(newSettings.attentionEnabled);
  }

  // Sync to backend (this triggers prompt generation)
  await invoke("set_commander_personality", { personality: newSettings });

  // Update the store after successful sync
  commanderPersonality.set(newSettings);

  console.log("[CommanderPersonality] Saved and synced to backend");
}

// Reset to defaults and sync to backend
export async function resetPersonality(): Promise<void> {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(defaultPersonality));
  await invoke("set_commander_personality", { personality: defaultPersonality });
  commanderPersonality.set(defaultPersonality);
  console.log("[CommanderPersonality] Reset to defaults");
}

// Check if settings differ from defaults
export function hasCustomSettings(): boolean {
  const current = get(commanderPersonality);
  return JSON.stringify(current) !== JSON.stringify(defaultPersonality);
}

// Get current settings (non-reactive)
export function getPersonality(): CommanderPersonality {
  return get(commanderPersonality);
}
