/**
 * Security-related utilities for the Claude Commander frontend.
 *
 * Consolidates risk level and severity classification that was duplicated across:
 * - ElevatedCommandModal.svelte
 * - SecurityAlertDetail.svelte
 */

import type { CommandRiskLevel } from '$lib/types';

// ============================================================================
// Risk Level Classification (for elevated commands)
// ============================================================================

/**
 * Get CSS class for command risk level styling
 * Used in: ElevatedCommandModal
 */
export function getRiskClass(level: CommandRiskLevel): string {
  switch (level) {
    case 'high':
      return 'risk-high';
    case 'suspicious':
      return 'risk-suspicious';
    default:
      return 'risk-normal';
  }
}

/**
 * Get display label for command risk level
 * Used in: ElevatedCommandModal
 */
export function getRiskLabel(level: CommandRiskLevel): string {
  switch (level) {
    case 'high':
      return 'HIGH RISK';
    case 'suspicious':
      return 'SUSPICIOUS';
    default:
      return 'NORMAL';
  }
}

/**
 * Get CSS variable color for risk level
 * Used in: ElevatedCommandModal styling
 */
export function getRiskColor(level: CommandRiskLevel): string {
  switch (level) {
    case 'high':
      return 'var(--error)';
    case 'suspicious':
      return 'var(--warning)';
    default:
      return 'var(--success)';
  }
}

// ============================================================================
// Severity Classification (for security alerts)
// ============================================================================

export type SeverityLevel = 'critical' | 'high' | 'medium' | 'low';

/**
 * Get CSS variable color for severity level
 * Used in: SecurityAlertDetail
 */
export function getSeverityColor(severity: SeverityLevel | string): string {
  switch (severity) {
    case 'critical':
      return 'var(--error)';
    case 'high':
      return 'var(--warning)';
    case 'medium':
      return 'var(--accent)';
    case 'low':
      return 'var(--text-muted)';
    default:
      return 'var(--text-muted)';
  }
}

/**
 * Get display label for severity level
 * Used in: SecurityAlertDetail
 */
export function getSeverityLabel(severity: SeverityLevel | string): string {
  switch (severity) {
    case 'critical':
      return 'CRITICAL';
    case 'high':
      return 'HIGH';
    case 'medium':
      return 'MEDIUM';
    case 'low':
      return 'LOW';
    default:
      return severity.toUpperCase();
  }
}

/**
 * Get CSS class for severity styling
 * Used in: SecurityAlertDetail
 */
export function getSeverityClass(severity: SeverityLevel | string): string {
  switch (severity) {
    case 'critical':
      return 'severity-critical';
    case 'high':
      return 'severity-high';
    case 'medium':
      return 'severity-medium';
    case 'low':
      return 'severity-low';
    default:
      return 'severity-unknown';
  }
}

// ============================================================================
// Risk Pattern Detection
// ============================================================================

/**
 * Patterns that indicate high-risk commands
 */
export const HIGH_RISK_PATTERNS = [
  /rm\s+-rf\s+\//,
  /dd\s+if=/,
  /mkfs/,
  /format\s+/i,
  />\s*\/dev\//,
] as const;

/**
 * Patterns that indicate suspicious commands
 */
export const SUSPICIOUS_PATTERNS = [
  /curl.*\|\s*bash/,
  /wget.*\|\s*bash/,
  /bash\s+-c/,
  /eval\s+/,
  /base64\s+-d.*\|/,
] as const;

/**
 * Check if a command matches high-risk patterns
 */
export function isHighRiskCommand(command: string): boolean {
  return HIGH_RISK_PATTERNS.some((pattern) => pattern.test(command));
}

/**
 * Check if a command matches suspicious patterns
 */
export function isSuspiciousCommand(command: string): boolean {
  return SUSPICIOUS_PATTERNS.some((pattern) => pattern.test(command));
}

/**
 * Determine risk level for a command string
 */
export function classifyCommandRisk(command: string): CommandRiskLevel {
  if (isHighRiskCommand(command)) return 'high';
  if (isSuspiciousCommand(command)) return 'suspicious';
  return 'normal';
}
