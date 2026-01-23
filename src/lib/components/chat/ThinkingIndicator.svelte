<script lang="ts">
  // Animated thinking indicator shown while waiting for response
</script>

<div class="thinking-message">
  <div class="thinking-dots">
    <span></span>
    <span></span>
    <span></span>
  </div>
</div>

<style>
  .thinking-message {
    padding: var(--space-3) 0;
  }

  .thinking-dots {
    background: var(--bg-tertiary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-lg);
    padding: var(--space-3) var(--space-4);
    display: flex;
    gap: var(--space-2);
    width: fit-content;
  }

  .thinking-dots span {
    width: 6px;
    height: 6px;
    border-radius: var(--radius-full);
    background: var(--accent-hex);
    animation: bounce 1.4s infinite ease-in-out;
  }

  .thinking-dots span:nth-child(1) {
    animation-delay: -0.32s;
  }

  .thinking-dots span:nth-child(2) {
    animation-delay: -0.16s;
  }

  @keyframes bounce {
    0%, 80%, 100% {
      transform: scale(0.6);
      opacity: 0.4;
    }
    40% {
      transform: scale(1);
      opacity: 1;
    }
  }
</style>
