<script lang="ts">
  let { onClose }: { onClose: () => void } = $props();
</script>

<div class="overlay" onclick={onClose}>
  <div class="dialog" onclick={(e) => e.stopPropagation()}>
    <div class="header">
      <h2>Claude Commander Help</h2>
      <button class="close-btn" onclick={onClose}>×</button>
    </div>

    <div class="content">
      <section>
        <h3>What is Claude Commander?</h3>
        <p>
          Claude Commander is an agent management system that lets you run multiple Claude AI agents
          simultaneously, each working in different directories or on different tasks.
        </p>
      </section>

      <section>
        <h3>Keyboard Shortcuts</h3>
        <div class="shortcut-grid">
          <div class="shortcut">
            <kbd>Ctrl/Cmd + N</kbd>
            <span>Create new agent</span>
          </div>
          <div class="shortcut">
            <kbd>Ctrl/Cmd + /</kbd>
            <span>Cycle layout mode (single/split/grid)</span>
          </div>
          <div class="shortcut">
            <kbd>Ctrl/Cmd + Shift + C</kbd>
            <span>Open chat view</span>
          </div>
          <div class="shortcut">
            <kbd>Ctrl/Cmd + Shift + D</kbd>
            <span>Toggle database stats</span>
          </div>
          <div class="shortcut">
            <kbd>Ctrl/Cmd + 1-9</kbd>
            <span>Select agent by number</span>
          </div>
          <div class="shortcut">
            <kbd>Esc</kbd>
            <span>Close dialogs</span>
          </div>
        </div>
      </section>

      <section>
        <h3>Features</h3>
        <ul>
          <li><strong>Multi-Agent Management:</strong> Run multiple Claude agents simultaneously</li>
          <li><strong>Layout Modes:</strong> View agents in single, split, or grid layouts</li>
          <li><strong>Chat View:</strong> Communicate with a meta-agent that can manage other agents</li>
          <li><strong>Tool Monitoring:</strong> Track tool calls and execution times in real-time</li>
          <li><strong>Database Stats:</strong> View conversation history and session statistics</li>
          <li><strong>Export:</strong> Export conversation histories in various formats</li>
          <li><strong>Auto-Pipelines:</strong> Automated multi-step workflows with verification</li>
        </ul>
      </section>

      <section>
        <h3>Getting Started</h3>
        <ol>
          <li>Click the <strong>+ New Agent</strong> button or press <kbd>Ctrl/Cmd + N</kbd></li>
          <li>Choose a working directory for your agent</li>
          <li>Optionally link to a GitHub repository</li>
          <li>Start chatting with your agent to give it tasks</li>
          <li>Use layout controls to view multiple agents at once</li>
        </ol>
      </section>

      <section>
        <h3>Agent States</h3>
        <div class="status-list">
          <div class="status-item">
            <span class="badge idle">Idle</span>
            <span>Agent is ready for input</span>
          </div>
          <div class="status-item">
            <span class="badge running">Running</span>
            <span>Agent is processing a request</span>
          </div>
          <div class="status-item">
            <span class="badge stopped">Stopped</span>
            <span>Agent has been stopped</span>
          </div>
          <div class="status-item">
            <span class="badge error">Error</span>
            <span>Agent encountered an error</span>
          </div>
        </div>
      </section>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    backdrop-filter: blur(4px);
  }

  .dialog {
    background-color: var(--bg-primary);
    border-radius: 16px;
    border: 1px solid var(--border);
    box-shadow: 0 24px 48px rgba(0, 0, 0, 0.4);
    width: 90%;
    max-width: 700px;
    max-height: 85vh;
    display: flex;
    flex-direction: column;
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-lg) var(--space-xl);
    border-bottom: 1px solid var(--border);
  }

  .header h2 {
    margin: 0;
    font-size: 24px;
    font-weight: 700;
    background: linear-gradient(135deg, var(--accent) 0%, #10b981 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .close-btn {
    width: 36px;
    height: 36px;
    border: none;
    background-color: var(--bg-tertiary);
    border-radius: 8px;
    cursor: pointer;
    font-size: 28px;
    color: var(--text-muted);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
    line-height: 1;
  }

  .close-btn:hover {
    background-color: var(--bg-elevated);
    color: var(--text-primary);
  }

  .content {
    overflow-y: auto;
    padding: var(--space-xl);
    display: flex;
    flex-direction: column;
    gap: var(--space-xl);
  }

  section h3 {
    margin: 0 0 var(--space-md) 0;
    font-size: 18px;
    font-weight: 700;
    color: var(--text-primary);
  }

  section p {
    margin: 0;
    line-height: 1.6;
    color: var(--text-secondary);
  }

  section ul, section ol {
    margin: 0;
    padding-left: var(--space-lg);
    line-height: 1.8;
    color: var(--text-secondary);
  }

  section li {
    margin-bottom: var(--space-sm);
  }

  .shortcut-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: var(--space-md);
  }

  .shortcut {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-md);
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
  }

  kbd {
    display: inline-block;
    padding: 6px 12px;
    font-size: 12px;
    font-weight: 600;
    line-height: 1;
    color: var(--text-primary);
    background-color: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 6px;
    box-shadow: 0 2px 0 var(--border);
    white-space: nowrap;
    font-family: monospace;
  }

  .shortcut span {
    color: var(--text-secondary);
    font-size: 14px;
  }

  .status-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .status-item {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-md);
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
  }

  .status-item span:last-child {
    color: var(--text-secondary);
    font-size: 14px;
  }

  .badge {
    padding: 6px 12px;
    border-radius: 6px;
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    min-width: 80px;
    text-align: center;
  }

  .badge.idle {
    background-color: rgba(59, 130, 246, 0.15);
    color: #3b82f6;
  }

  .badge.running {
    background-color: rgba(16, 185, 129, 0.15);
    color: #10b981;
  }

  .badge.stopped {
    background-color: rgba(107, 114, 128, 0.15);
    color: #9ca3af;
  }

  .badge.error {
    background-color: rgba(239, 68, 68, 0.15);
    color: #ef4444;
  }

  strong {
    color: var(--text-primary);
    font-weight: 600;
  }
</style>
