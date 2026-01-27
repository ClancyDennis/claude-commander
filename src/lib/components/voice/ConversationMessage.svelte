<script lang="ts">
  import { Check } from "$lib/components/ui/icons";
  import type { DiscussMessage } from "$lib/stores/voice";

  interface Props {
    message: DiscussMessage;
    isCurrent?: boolean;
    onclick?: () => void;
  }

  let { message, isCurrent = false, onclick }: Props = $props();

  const roleLabel = $derived(message.role === "user" ? "You" : "AI");
</script>

{#if isCurrent}
  <div class="message {message.role} current">
    <span class="label">{roleLabel}</span>
    <p class="text">{message.content}<span class="cursor">|</span></p>
  </div>
{:else}
  <button
    class="message {message.role}"
    class:selected={message.selected}
    {onclick}
    type="button"
  >
    {#if message.selected}
      <span class="select-indicator">
        <Check size={12} />
      </span>
    {/if}
    <span class="label">{roleLabel}</span>
    <p class="text">{message.content}</p>
  </button>
{/if}

<style>
  .message {
    padding: 0.75rem;
    border-radius: 0.5rem;
    border: none;
    cursor: pointer;
    text-align: left;
    transition: all 0.15s ease;
    position: relative;
    width: fit-content;
    max-width: 90%;
  }

  .message:hover {
    transform: translateX(2px);
  }

  .message.user {
    background: var(--bg-tertiary);
    align-self: flex-end;
  }

  .message.assistant {
    background: rgba(34, 197, 94, 0.1);
    border: 1px solid rgba(34, 197, 94, 0.2);
    align-self: flex-start;
  }

  .message.selected {
    outline: 2px solid var(--accent-hex);
    outline-offset: 2px;
  }

  .message.current {
    cursor: default;
    opacity: 0.8;
  }

  .message.current:hover {
    transform: none;
  }

  .select-indicator {
    position: absolute;
    top: 0.5rem;
    right: 0.5rem;
    width: 18px;
    height: 18px;
    background: var(--accent-hex);
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
  }

  .message .label {
    font-size: 0.625rem;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--text-tertiary);
    display: block;
    margin-bottom: 0.25rem;
  }

  .message .text {
    font-size: 0.875rem;
    line-height: 1.5;
    margin: 0;
    color: var(--text-primary);
  }

  .cursor {
    animation: blink 1s step-end infinite;
  }

  @keyframes blink {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0;
    }
  }
</style>
