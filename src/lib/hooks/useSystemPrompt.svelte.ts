import { invoke } from "@tauri-apps/api/core";
import type { SystemPromptInfo } from "$lib/types";

/**
 * Strips leading/trailing triple backticks (with optional language identifier) from text.
 * Handles cases like ```markdown\n...\n``` or ```\n...\n```
 */
function stripCodeFence(text: string): string {
  const trimmed = text.trim();
  // Match opening fence: ``` optionally followed by a language identifier and newline
  const openingMatch = trimmed.match(/^```[a-zA-Z]*\r?\n?/);
  if (!openingMatch) return text;

  // Check for closing fence at the end
  if (!trimmed.endsWith("```")) return text;

  // Strip both fences
  const withoutOpening = trimmed.slice(openingMatch[0].length);
  const withoutClosing = withoutOpening.slice(0, -3).trimEnd();
  return withoutClosing;
}

export function useSystemPrompt() {
  let state = $state({
    showSystemPrompt: false,
    systemPrompt: null as SystemPromptInfo | null,
    systemPromptLoading: false,
    systemPromptError: null as string | null,
    systemPromptCopied: false,
    resetting: false,
  });
  let copyTimeout: ReturnType<typeof setTimeout> | null = null;

  async function loadSystemPrompt() {
    state.systemPromptLoading = true;
    state.systemPromptError = null;
    try {
      const result = await invoke<SystemPromptInfo>("get_commander_system_prompt");
      // Strip any wrapping code fences the LLM may have added
      if (result.prompt) {
        result.prompt = stripCodeFence(result.prompt);
      }
      state.systemPrompt = result;
    } catch (err) {
      state.systemPromptError = String(err);
    } finally {
      state.systemPromptLoading = false;
    }
  }

  function openSystemPrompt() {
    state.showSystemPrompt = true;
    state.systemPromptCopied = false;
    if (copyTimeout) {
      clearTimeout(copyTimeout);
      copyTimeout = null;
    }
    void loadSystemPrompt();
  }

  function closeSystemPrompt() {
    state.showSystemPrompt = false;
  }

  async function copySystemPrompt() {
    if (!state.systemPrompt?.prompt) return;
    try {
      await navigator.clipboard.writeText(state.systemPrompt.prompt);
      state.systemPromptCopied = true;
      if (copyTimeout) {
        clearTimeout(copyTimeout);
      }
      copyTimeout = setTimeout(() => {
        state.systemPromptCopied = false;
        copyTimeout = null;
      }, 2000);
    } catch (err) {
      state.systemPromptError = String(err);
    }
  }

  function handleSystemPromptKeydown(e: KeyboardEvent) {
    if (!state.showSystemPrompt) return;
    if (e.key === "Escape") {
      closeSystemPrompt();
    }
  }

  async function resetSystemPrompt() {
    state.resetting = true;
    state.systemPromptError = null;
    try {
      await invoke("reset_commander_personality");
      // Reload the prompt to show the base version
      await loadSystemPrompt();
    } catch (err) {
      state.systemPromptError = String(err);
    } finally {
      state.resetting = false;
    }
  }

  return {
    state,
    loadSystemPrompt,
    openSystemPrompt,
    closeSystemPrompt,
    copySystemPrompt,
    handleSystemPromptKeydown,
    resetSystemPrompt,
  };
}
