<script lang="ts">
  import { ViewHeader } from "$lib/components/ui/layout";
  import { IconButton } from "$lib/components/ui/button";
  import { Trash2, Mic, CheckSquare } from "$lib/components/ui/icons";
  import HelpTip from "../new-agent/HelpTip.svelte";
  import { metaAgentContextInfo } from "$lib/stores/agents";

  interface Props {
    isThinking: boolean;
    onClear: () => void;
    hasOpenAiKey?: boolean;
    onVoiceClick?: () => void;
    showTodoPanel?: boolean;
    onToggleTodoPanel?: () => void;
    hasTodos?: boolean;
  }

  let {
    isThinking,
    onClear,
    hasOpenAiKey = false,
    onVoiceClick,
    showTodoPanel = false,
    onToggleTodoPanel,
    hasTodos = false,
  }: Props = $props();

  // Format token count for display (e.g., 84000 -> "84k")
  function formatTokens(tokens: number): string {
    if (tokens >= 1000) {
      return `${(tokens / 1000).toFixed(0)}k`;
    }
    return tokens.toString();
  }

  // Build tooltip text
  function getTooltip(info: typeof $metaAgentContextInfo): string {
    if (!info) return "";
    return `${formatTokens(info.currentTokens)} / ${formatTokens(info.availableTokens)} tokens\n${formatTokens(info.remainingTokens)} remaining`;
  }
</script>

<ViewHeader emojiIcon="🎛️" title="System Commander">
  {#snippet children()}
    <HelpTip
      text="Natural language control center. Create agents, send prompts, and manage your fleet."
      placement="bottom"
    />
    {#if $metaAgentContextInfo}
      <span
        class="context-usage context-{$metaAgentContextInfo.state}"
        title={getTooltip($metaAgentContextInfo)}
      >
        {$metaAgentContextInfo.usagePercent.toFixed(0)}%
      </span>
    {/if}
    {#if isThinking}
      <span class="thinking-indicator">Thinking...</span>
    {/if}
  {/snippet}
  {#snippet actions()}
    {#if hasTodos || showTodoPanel}
      <IconButton
        icon={CheckSquare}
        label="Tasks"
        variant={showTodoPanel ? "primary" : "ghost"}
        onclick={onToggleTodoPanel}
        title="Toggle task list panel"
      />
    {/if}
    {#if hasOpenAiKey}
      <IconButton
        icon={Mic}
        label="Voice"
        variant="ghost"
        onclick={onVoiceClick}
        title="Open voice controls (Dictate or Discuss)"
      />
    {/if}
    <IconButton
      icon={Trash2}
      label="Clear"
      variant="ghost"
      onclick={onClear}
    />
  {/snippet}
</ViewHeader>

<style>
  .context-usage {
    font-size: 11px;
    font-weight: 500;
    padding: 2px 6px;
    border-radius: 4px;
    cursor: help;
    margin-left: 8px;
    font-variant-numeric: tabular-nums;
  }

  .context-normal {
    color: var(--text-muted);
    background: var(--surface-2);
  }

  .context-warning {
    color: var(--warning-text, #b45309);
    background: var(--warning-bg, rgba(251, 191, 36, 0.15));
  }

  .context-critical {
    color: var(--error-text, #dc2626);
    background: var(--error-bg, rgba(239, 68, 68, 0.15));
  }

  .context-overflow {
    color: var(--error-text, #dc2626);
    background: var(--error-bg, rgba(239, 68, 68, 0.25));
    animation: pulse 1s ease-in-out infinite;
  }

  .thinking-indicator {
    font-size: 12px;
    color: var(--accent-hex);
    font-weight: 500;
    animation: pulse 1.5s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }
</style>
