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
    padding: var(--space-10);
    color: var(--text-muted);
  }

  .empty-icon {
    font-size: 56px;
    margin-bottom: var(--space-4);
    opacity: 0.7;
  }

  .empty-title {
    font-size: var(--text-2xl);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
    margin-bottom: var(--space-2);
  }

  .empty-description {
    font-size: var(--text-sm);
    margin-bottom: var(--space-6);
    max-width: 360px;
    line-height: var(--leading-relaxed);
  }

  .example-prompts {
    background: var(--bg-tertiary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
    max-width: 360px;
    width: 100%;
  }

  .example-title {
    font-size: var(--text-xs);
    font-weight: var(--font-semibold);
    color: var(--accent-hex);
    margin-bottom: var(--space-3);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .example {
    font-size: var(--text-sm);
    padding: var(--space-2) var(--space-3);
    background: var(--bg-elevated);
    border-radius: var(--radius-md);
    margin-bottom: var(--space-2);
    color: var(--text-secondary);
    font-style: italic;
    text-align: left;
    transition: background var(--transition-fast);
  }

  .example:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .example:last-child {
    margin-bottom: 0;
  }
</style>
