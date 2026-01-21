<script lang="ts">
  import type { InstructionIssue } from "../../types";

  let { issue }: { issue: InstructionIssue } = $props();

  const severityColors: Record<string, { bg: string; border: string; text: string }> = {
    critical: { bg: "rgba(239, 68, 68, 0.15)", border: "#ef4444", text: "#ef4444" },
    warning: { bg: "rgba(245, 158, 11, 0.15)", border: "#f59e0b", text: "#f59e0b" },
    info: { bg: "rgba(59, 130, 246, 0.15)", border: "#3b82f6", text: "#3b82f6" },
  };

  const severityIcons: Record<string, string> = {
    critical: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z",
    warning: "M12 9v2m0 4h.01M12 3a9 9 0 100 18 9 9 0 000-18z",
    info: "M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z",
  };

  let colors = $derived(severityColors[issue.severity] || severityColors.info);
  let iconPath = $derived(severityIcons[issue.severity] || severityIcons.info);
</script>

<div
  class="issue-item"
  style="background: {colors.bg}; border-color: {colors.border};"
>
  <div class="issue-header">
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke={colors.text}
      stroke-width="2"
      class="severity-icon"
    >
      <path d={iconPath} />
    </svg>
    <span class="severity-badge" style="background: {colors.text};">
      {issue.severity}
    </span>
    <span class="issue-title">{issue.title}</span>
  </div>
  <p class="issue-description">{issue.description}</p>
  {#if issue.lineStart}
    <span class="line-ref">Line {issue.lineStart}{issue.lineEnd && issue.lineEnd !== issue.lineStart ? `-${issue.lineEnd}` : ''}</span>
  {/if}
</div>

<style>
  .issue-item {
    padding: var(--space-md);
    border-radius: 8px;
    border: 1px solid;
  }

  .issue-header {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    margin-bottom: var(--space-sm);
  }

  .severity-icon {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
  }

  .severity-badge {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    padding: 2px 6px;
    border-radius: 4px;
    color: white;
  }

  .issue-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .issue-description {
    font-size: 12px;
    color: var(--text-secondary);
    margin: 0;
    line-height: 1.5;
  }

  .line-ref {
    display: inline-block;
    margin-top: var(--space-sm);
    font-size: 11px;
    color: var(--text-muted);
    font-family: monospace;
  }
</style>
