// Welcome Modal Types

export interface ClaudeCodeStatus {
  checking: boolean;
  installed: boolean;
  path: string | null;
  version: string | null;
  authenticated: boolean;
  authType: string | null;
  subscriptionType: string | null;
}

export type ApiKeyStatus = "unchecked" | "validating" | "valid" | "invalid";
export type ClaudeAuthStatus = "unchecked" | "ready";

export interface ApiKeyConfig {
  expanded: boolean;
  value: string;
  status: ApiKeyStatus;
  message: string;
}

export interface ClaudeAuthConfig {
  expanded: boolean;
  status: ClaudeAuthStatus;
}

export interface AuthConfig {
  claudeAuth: ClaudeAuthConfig;
  anthropicKey: ApiKeyConfig;
  openaiKey: ApiKeyConfig;
}

export interface KeyboardShortcut {
  keys: string[];
  label: string;
}

export const TOTAL_STEPS = 3;
export const STORAGE_KEY = "claude-commander-onboarding-complete";

export function createInitialClaudeStatus(): ClaudeCodeStatus {
  return {
    checking: true,
    installed: false,
    path: null,
    version: null,
    authenticated: false,
    authType: null,
    subscriptionType: null
  };
}

export function createInitialAuthConfig(): AuthConfig {
  return {
    claudeAuth: { expanded: false, status: "unchecked" },
    anthropicKey: { expanded: false, value: "", status: "unchecked", message: "" },
    openaiKey: { expanded: false, value: "", status: "unchecked", message: "" }
  };
}
