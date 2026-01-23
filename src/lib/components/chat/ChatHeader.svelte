<script lang="ts">
  import { ViewHeader } from "$lib/components/ui/layout";
  import { IconButton } from "$lib/components/ui/button";
  import { Trash2 } from "$lib/components/ui/icons";
  import HelpTip from "../new-agent/HelpTip.svelte";
  import VoiceButton from "../voice/VoiceButton.svelte";

  interface Props {
    isThinking: boolean;
    onClear: () => void;
    hasOpenAiKey?: boolean;
  }

  let { isThinking, onClear, hasOpenAiKey = false }: Props = $props();
</script>

<ViewHeader emojiIcon="🎛️" title="System Commander">
  {#snippet children()}
    <HelpTip
      text="Natural language control center. Create agents, send prompts, and manage your fleet."
      placement="bottom"
    />
    {#if isThinking}
      <span class="thinking-indicator">Thinking...</span>
    {/if}
  {/snippet}
  {#snippet actions()}
    {#if hasOpenAiKey}
      <VoiceButton />
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
