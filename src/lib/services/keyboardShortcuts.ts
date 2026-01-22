/**
 * Keyboard Shortcuts Service
 *
 * Centralized keyboard shortcut handling for the application.
 * Supports Cmd/Ctrl modifier combinations.
 */

import type { Agent } from "$lib/types";

// ============================================================================
// Types
// ============================================================================

export interface KeyboardShortcutCallbacks {
  /** Called when new agent shortcut is pressed (Cmd/Ctrl + N) */
  onNewAgent: () => void;

  /** Called when layout toggle shortcut is pressed (Cmd/Ctrl + /) */
  onToggleLayout: () => void;

  /** Called when chat shortcut is pressed (Cmd/Ctrl + Shift + C) */
  onOpenChat: () => void;

  /** Called when database stats shortcut is pressed (Cmd/Ctrl + Shift + D) */
  onToggleDatabaseStats: () => void;

  /** Called when agent selection shortcut is pressed (Cmd/Ctrl + 1-9) */
  onSelectAgent: (agentIndex: number) => void;

  /** Called when escape is pressed */
  onEscape: () => void;
}

export interface KeyboardShortcutConfig {
  /** Function to get the current list of agents for number shortcuts */
  getAgents: () => Agent[];
}

// ============================================================================
// Keyboard Handler
// ============================================================================

/**
 * Creates a keyboard event handler with the provided callbacks.
 * Returns a function that can be used as an event listener.
 */
export function createKeyboardHandler(
  callbacks: KeyboardShortcutCallbacks,
  config: KeyboardShortcutConfig
): (e: KeyboardEvent) => void {
  return (e: KeyboardEvent) => {
    const isMod = e.ctrlKey || e.metaKey;

    // Cmd/Ctrl + N: New agent
    if (isMod && e.key === 'n') {
      e.preventDefault();
      callbacks.onNewAgent();
      return;
    }

    // Cmd/Ctrl + /: Toggle layout mode
    if (isMod && e.key === '/') {
      e.preventDefault();
      callbacks.onToggleLayout();
      return;
    }

    // Cmd/Ctrl + Shift + C: Open chat
    if (isMod && e.shiftKey && e.key === 'C') {
      e.preventDefault();
      callbacks.onOpenChat();
      return;
    }

    // Cmd/Ctrl + Shift + D: Toggle database stats
    if (isMod && e.shiftKey && e.key === 'D') {
      e.preventDefault();
      callbacks.onToggleDatabaseStats();
      return;
    }

    // Cmd/Ctrl + 1-9: Select agent by number
    if (isMod && e.key >= '1' && e.key <= '9') {
      e.preventDefault();
      const agentIndex = parseInt(e.key) - 1;
      const agentList = config.getAgents();
      if (agentIndex < agentList.length) {
        callbacks.onSelectAgent(agentIndex);
      }
      return;
    }

    // Escape: Close dialogs
    if (e.key === 'Escape') {
      callbacks.onEscape();
      return;
    }
  };
}

/**
 * Sets up keyboard shortcuts and returns a cleanup function.
 */
export function setupKeyboardShortcuts(
  callbacks: KeyboardShortcutCallbacks,
  config: KeyboardShortcutConfig
): () => void {
  const handler = createKeyboardHandler(callbacks, config);
  window.addEventListener('keydown', handler);

  return () => {
    window.removeEventListener('keydown', handler);
  };
}

// ============================================================================
// Shortcut Descriptions (for help display)
// ============================================================================

export const KEYBOARD_SHORTCUTS = [
  { keys: ['Cmd/Ctrl', 'N'], description: 'New agent' },
  { keys: ['Cmd/Ctrl', '/'], description: 'Toggle layout mode' },
  { keys: ['Cmd/Ctrl', 'Shift', 'C'], description: 'Open chat' },
  { keys: ['Cmd/Ctrl', 'Shift', 'D'], description: 'Toggle database stats' },
  { keys: ['Cmd/Ctrl', '1-9'], description: 'Select agent by number' },
  { keys: ['Escape'], description: 'Close dialogs' },
] as const;
