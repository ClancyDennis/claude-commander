<script lang="ts">
  import HelpTip from "../new-agent/HelpTip.svelte";

  interface Props {
    isThinking: boolean;
    onClear: () => void;
  }

  let { isThinking, onClear }: Props = $props();
</script>

<div class="chat-header">
  <div class="header-left">
    <span class="header-icon">🎛️</span>
    <span class="header-title">
      System Commander
      <HelpTip
        text="Natural language control center. Create agents, send prompts, and manage your fleet."
        placement="bottom"
      />
    </span>
    {#if isThinking}
      <span class="thinking-indicator">Thinking...</span>
    {/if}
  </div>
  <div class="header-actions">
    <button onclick={onClear} class="action-btn" title="Clear chat">
      Clear
    </button>
  </div>
</div>

<style>
  .chat-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    background: #1a1a1f;
    border-bottom: 1px solid rgba(124, 58, 237, 0.2);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .header-icon {
    font-size: 24px;
  }

  .header-title {
    font-size: 18px;
    font-weight: 600;
    color: #e0e0e0;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .thinking-indicator {
    font-size: 12px;
    color: #7c3aed;
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

  .header-actions {
    display: flex;
    gap: 8px;
  }

  .action-btn {
    padding: 8px 16px;
    background: transparent;
    border: 1px solid rgba(124, 58, 237, 0.3);
    border-radius: 6px;
    color: #7c3aed;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .action-btn:hover {
    background: rgba(124, 58, 237, 0.1);
    border-color: rgba(124, 58, 237, 0.5);
  }
</style>
