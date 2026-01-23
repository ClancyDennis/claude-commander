<script lang="ts">
  import type { PendingElevatedCommand } from "$lib/types";

  interface Props {
    command: PendingElevatedCommand;
  }

  let { command }: Props = $props();
</script>

<!-- Main command display -->
<div class="command-section">
  <h3>Command</h3>
  <div class="command-box">
    <code>{command.command}</code>
  </div>
</div>

<!-- Compound command breakdown (if applicable) -->
{#if command.preCommands?.length || command.postCommands?.length}
  <div class="compound-section">
    <h4>Compound Command Breakdown</h4>
    {#if command.preCommands?.length}
      <div class="compound-part">
        <span class="compound-label">Before sudo:</span>
        <code class="compound-cmd">{command.preCommands.join(" && ")}</code>
      </div>
    {/if}
    {#if command.sudoCommand}
      <div class="compound-part sudo-part">
        <span class="compound-label">Sudo runs:</span>
        <code class="compound-cmd highlight">{command.sudoCommand}</code>
      </div>
    {/if}
    {#if command.postCommands?.length}
      <div class="compound-part">
        <span class="compound-label">After sudo:</span>
        <code class="compound-cmd">{command.postCommands.join(" && ")}</code>
      </div>
    {/if}
  </div>
{/if}

<!-- Inner command for bash -c -->
{#if command.innerCommand}
  <div class="inner-command-section">
    <h4>Expanded Command (from bash -c)</h4>
    <div class="command-box inner">
      <code>{command.innerCommand}</code>
    </div>
  </div>
{/if}

<style>
  .command-section h3,
  .compound-section h4,
  .inner-command-section h4 {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-muted);
    margin: 0 0 var(--space-sm) 0;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .command-box {
    background-color: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: var(--space-md);
    overflow-x: auto;
  }

  .command-box code {
    font-family: var(--font-mono);
    font-size: 14px;
    color: var(--text-primary);
    white-space: pre-wrap;
    word-break: break-all;
  }

  .command-box.inner {
    background-color: var(--bg-tertiary);
    border-color: var(--warning);
  }

  .compound-section {
    background-color: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: var(--space-md);
  }

  .compound-part {
    display: flex;
    align-items: flex-start;
    gap: var(--space-sm);
    margin-bottom: var(--space-sm);
  }

  .compound-part:last-child {
    margin-bottom: 0;
  }

  .compound-label {
    color: var(--text-muted);
    font-size: 12px;
    min-width: 90px;
    flex-shrink: 0;
  }

  .compound-cmd {
    font-family: var(--font-mono);
    font-size: 13px;
    color: var(--text-secondary);
  }

  .compound-cmd.highlight {
    color: var(--accent);
    font-weight: 600;
  }

  .sudo-part {
    background-color: var(--accent-glow);
    margin: 0 calc(-1 * var(--space-md));
    padding: var(--space-sm) var(--space-md);
  }
</style>
