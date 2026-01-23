<script lang="ts">
  // Empty state shown when chat has no messages
</script>

<div class="empty-state">
  <div class="empty-icon">🚀</div>
  <div class="empty-title">Mission Control</div>
  <div class="empty-description">
    Command your agents, deploy prompts, take control.
  </div>
  <div class="example-prompts">
    <div class="example-title">Try asking:</div>
    <div class="example">"List all running agents"</div>
    <div class="example">"Create a new agent in /tmp/test"</div>
    <div class="example">"Send a prompt to the most recent agent"</div>
  </div>
</div>

<style>
  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: 40px;
    color: #999;
  }

  .empty-icon {
    font-size: 64px;
    margin-bottom: 16px;
    opacity: 0.6;
  }

  .empty-title {
    font-size: 24px;
    font-weight: 600;
    color: #e0e0e0;
    margin-bottom: 8px;
  }

  .empty-description {
    font-size: 14px;
    margin-bottom: 24px;
    max-width: 400px;
  }

  .example-prompts {
    background: rgba(124, 58, 237, 0.05);
    border: 1px solid rgba(124, 58, 237, 0.2);
    border-radius: 12px;
    padding: 16px;
    max-width: 400px;
  }

  .example-title {
    font-size: 12px;
    font-weight: 600;
    color: #7c3aed;
    margin-bottom: 12px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .example {
    font-size: 13px;
    padding: 8px 12px;
    background: rgba(124, 58, 237, 0.1);
    border-radius: 6px;
    margin-bottom: 6px;
    color: #c0c0c0;
    font-style: italic;
  }

  .example:last-child {
    margin-bottom: 0;
  }
</style>
