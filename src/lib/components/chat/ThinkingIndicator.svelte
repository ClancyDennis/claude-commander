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
    padding-top: 10px;
    padding-bottom: 10px;
  }

  .thinking-dots {
    background: #1a1a1f;
    border: 1px solid rgba(124, 58, 237, 0.2);
    border-radius: 12px;
    padding: 12px 20px;
    display: flex;
    gap: 6px;
    width: fit-content;
  }

  .thinking-dots span {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #7c3aed;
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
      transform: scale(0);
      opacity: 0.5;
    }
    40% {
      transform: scale(1);
      opacity: 1;
    }
  }
</style>
