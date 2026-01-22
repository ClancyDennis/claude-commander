<script lang="ts">
  // Simple typing indicator for when agent is processing
</script>

<div class="typing-indicator">
  <span class="dot" style="animation-delay: 0ms"></span>
  <span class="dot" style="animation-delay: 150ms"></span>
  <span class="dot" style="animation-delay: 300ms"></span>
</div>

<style>
  .typing-indicator {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 12px 20px;
    background-color: var(--bg-tertiary);
    border-radius: 12px;
    border: 1px solid var(--border);
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background-color: var(--accent);
    animation: typing-wave 1.4s ease-in-out infinite;
  }
</style>
