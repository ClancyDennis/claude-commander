import { writable } from "svelte/store";

export interface RateLimitState {
  isLimited: boolean;
  resetTime: Date | null;
  timezone: string | null;
  rawMessage: string | null;
}

const initialState: RateLimitState = {
  isLimited: false,
  resetTime: null,
  timezone: null,
  rawMessage: null,
};

export const rateLimitState = writable<RateLimitState>(initialState);

/**
 * Parse a rate limit error message and extract the reset time.
 * Expected format: "You've hit your limit · resets 1pm (America/Los_Angeles)"
 */
export function parseRateLimitError(message: string): { resetTime: Date; timezone: string } | null {
  // Match patterns like "resets 1pm" or "resets 12:30pm" with optional timezone
  const resetMatch = message.match(/resets?\s+(\d{1,2}(?::\d{2})?\s*(?:am|pm))\s*(?:\(([^)]+)\))?/i);

  if (!resetMatch) return null;

  const timeStr = resetMatch[1].trim();
  const timezone = resetMatch[2]?.trim() || Intl.DateTimeFormat().resolvedOptions().timeZone;

  // Parse the time string
  const timeMatch = timeStr.match(/^(\d{1,2})(?::(\d{2}))?\s*(am|pm)$/i);
  if (!timeMatch) return null;

  let hours = parseInt(timeMatch[1], 10);
  const minutes = timeMatch[2] ? parseInt(timeMatch[2], 10) : 0;
  const isPM = timeMatch[3].toLowerCase() === "pm";

  // Convert to 24-hour format
  if (isPM && hours !== 12) {
    hours += 12;
  } else if (!isPM && hours === 12) {
    hours = 0;
  }

  // Create a date for today with the specified time
  const now = new Date();
  const resetTime = new Date(now);
  resetTime.setHours(hours, minutes, 0, 0);

  // If the reset time is in the past, assume it's tomorrow
  if (resetTime <= now) {
    resetTime.setDate(resetTime.getDate() + 1);
  }

  return { resetTime, timezone };
}

/**
 * Check if an error message indicates a rate limit and set the state if so.
 * Returns true if a rate limit was detected.
 */
export function checkAndSetRateLimit(errorMessage: string): boolean {
  // Check for rate limit indicators
  const isRateLimitError =
    errorMessage.toLowerCase().includes("hit your limit") ||
    errorMessage.toLowerCase().includes("rate limit") ||
    errorMessage.includes("resets");

  if (!isRateLimitError) return false;

  const parsed = parseRateLimitError(errorMessage);

  if (parsed) {
    rateLimitState.set({
      isLimited: true,
      resetTime: parsed.resetTime,
      timezone: parsed.timezone,
      rawMessage: errorMessage,
    });
    return true;
  }

  // Even if we can't parse the time, show the modal with a default 1-hour countdown
  rateLimitState.set({
    isLimited: true,
    resetTime: new Date(Date.now() + 60 * 60 * 1000), // Default to 1 hour
    timezone: null,
    rawMessage: errorMessage,
  });
  return true;
}

/**
 * Clear the rate limit state (called when the countdown expires or manually dismissed).
 */
export function clearRateLimit() {
  rateLimitState.set(initialState);
}

/**
 * Check if currently rate limited.
 */
export function isRateLimited(): boolean {
  let limited = false;
  rateLimitState.subscribe((state) => {
    limited = state.isLimited;
  })();
  return limited;
}
