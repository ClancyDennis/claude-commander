<script lang="ts">
  import { get } from "svelte/store";
  import { invoke } from "@tauri-apps/api/core";
  import {
    showElevatedCommandModal,
    closeElevatedCommandModal,
    selectedElevatedCommand,
    pendingElevatedSorted,
    updateElevatedCommandStatus,
  } from "../stores/security";
  import { agents } from "../stores/agents";
  import type { PendingElevatedCommand, CommandRiskLevel } from "../types";

  let approveScope = $state(false);
  let isProcessing = $state(false);
  let error = $state<string | null>(null);

  // Get the command to display (selected or first pending)
  let currentCommand = $derived.by(() => {
    if ($selectedElevatedCommand) {
      return $selectedElevatedCommand;
    }
    const pending = $pendingElevatedSorted;
    return pending.length > 0 ? pending[0] : null;
  });

  // Safety: auto-close modal if no valid command exists
  $effect(() => {
    if ($showElevatedCommandModal && !currentCommand) {
      closeElevatedCommandModal();
    }
  });

  // Risk level styling
  function getRiskClass(level: CommandRiskLevel): string {
    switch (level) {
      case "high":
        return "risk-high";
      case "suspicious":
        return "risk-suspicious";
      default:
        return "risk-normal";
    }
  }

  function getRiskLabel(level: CommandRiskLevel): string {
    switch (level) {
      case "high":
        return "HIGH RISK";
      case "suspicious":
        return "SUSPICIOUS";
      default:
        return "NORMAL";
    }
  }

  function getAgentName(agentId: string): string {
    if (!agentId) return "Unknown";
    const agentsMap = get(agents);
    const agent = agentsMap.get(agentId);
    return agent?.title || agentId.slice(0, 8);
  }

  function formatTime(timestamp: number): string {
    return new Date(timestamp).toLocaleTimeString();
  }

  function getTimeRemaining(expiresAt: number): string {
    const remaining = Math.max(0, expiresAt - Date.now());
    const minutes = Math.floor(remaining / 60000);
    const seconds = Math.floor((remaining % 60000) / 1000);
    return `${minutes}:${seconds.toString().padStart(2, "0")}`;
  }

  async function handleApprove() {
    const cmd = currentCommand;
    if (!cmd || isProcessing) return;

    isProcessing = true;
    error = null;

    try {
      await invoke("approve_elevated_command", {
        requestId: cmd.id,
        approveScope: approveScope && !!cmd.scriptHash,
      });
      updateElevatedCommandStatus(cmd.id, "approved");
      closeElevatedCommandModal();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      isProcessing = false;
    }
  }

  async function handleDeny() {
    const cmd = currentCommand;
    if (!cmd || isProcessing) return;

    isProcessing = true;
    error = null;

    try {
      await invoke("deny_elevated_command", {
        requestId: cmd.id,
      });
      updateElevatedCommandStatus(cmd.id, "denied");
      closeElevatedCommandModal();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      isProcessing = false;
    }
  }

  function handleOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      closeElevatedCommandModal();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      closeElevatedCommandModal();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if $showElevatedCommandModal && currentCommand}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="overlay animate-fade-in" onclick={handleOverlayClick}>
    <div class="dialog animate-slide-up" role="dialog" aria-modal="true" tabindex="-1">
      <div class="header">
        <div class="title-row">
          <svg class="lock-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
            <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
          </svg>
          <h2>Elevated Command Request</h2>
        </div>
        <button class="close-btn" onclick={closeElevatedCommandModal} aria-label="Close dialog">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M6 18L18 6M6 6l12 12" stroke-linecap="round"/>
          </svg>
        </button>
      </div>

      <div class="content">
        <!-- Risk level badge -->
        <div class="risk-badge {getRiskClass(currentCommand.riskLevel)}">
          {getRiskLabel(currentCommand.riskLevel)}
        </div>

        <!-- Agent info -->
        <div class="info-row">
          <span class="label">Agent:</span>
          <span class="value">{getAgentName(currentCommand.agentId)}</span>
        </div>
        <div class="info-row">
          <span class="label">Working Dir:</span>
          <span class="value mono">{currentCommand.workingDir}</span>
        </div>
        <div class="info-row">
          <span class="label">Requested:</span>
          <span class="value">{formatTime(currentCommand.requestedAt)}</span>
          <span class="timer">Expires in {getTimeRemaining(currentCommand.expiresAt)}</span>
        </div>

        <!-- Command display -->
        <div class="command-section">
          <h3>Command</h3>
          <div class="command-box">
            <code>{currentCommand.command}</code>
          </div>
        </div>

        <!-- Compound command breakdown (if applicable) -->
        {#if currentCommand.preCommands?.length || currentCommand.postCommands?.length}
          <div class="compound-section">
            <h4>Compound Command Breakdown</h4>
            {#if currentCommand.preCommands?.length}
              <div class="compound-part">
                <span class="compound-label">Before sudo:</span>
                <code class="compound-cmd">{currentCommand.preCommands.join(" && ")}</code>
              </div>
            {/if}
            {#if currentCommand.sudoCommand}
              <div class="compound-part sudo-part">
                <span class="compound-label">Sudo runs:</span>
                <code class="compound-cmd highlight">{currentCommand.sudoCommand}</code>
              </div>
            {/if}
            {#if currentCommand.postCommands?.length}
              <div class="compound-part">
                <span class="compound-label">After sudo:</span>
                <code class="compound-cmd">{currentCommand.postCommands.join(" && ")}</code>
              </div>
            {/if}
          </div>
        {/if}

        <!-- Inner command for bash -c -->
        {#if currentCommand.innerCommand}
          <div class="inner-command-section">
            <h4>Expanded Command (from bash -c)</h4>
            <div class="command-box inner">
              <code>{currentCommand.innerCommand}</code>
            </div>
          </div>
        {/if}

        <!-- Warnings -->
        {#if currentCommand.warnings?.length}
          <div class="warnings-section">
            <h4>Warnings</h4>
            <ul class="warnings-list">
              {#each currentCommand.warnings as warning}
                <li class="warning-item">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M12 9v4m0 4h.01M12 2L2 22h20L12 2z"/>
                  </svg>
                  <span>{warning}</span>
                </li>
              {/each}
            </ul>
          </div>
        {/if}

        <!-- High risk extra warning -->
        {#if currentCommand.riskLevel === "high"}
          <div class="danger-warning">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 9v4m0 4h.01M12 2L2 22h20L12 2z"/>
            </svg>
            <div class="danger-text">
              <strong>DANGER:</strong> This command has been flagged as potentially destructive.
              Carefully review before approving.
            </div>
          </div>
        {/if}

        <!-- Script scope option -->
        {#if currentCommand.scriptHash}
          <div class="scope-option">
            <label class="checkbox-label">
              <input type="checkbox" bind:checked={approveScope} />
              <span>Approve all commands from this script</span>
            </label>
            {#if currentCommand.parentCmd}
              <span class="parent-cmd">Script: {currentCommand.parentCmd}</span>
            {/if}
          </div>
        {/if}

        <!-- Error display -->
        {#if error}
          <div class="error-message">
            {error}
          </div>
        {/if}
      </div>

      <div class="footer">
        <button
          class="btn btn-deny"
          onclick={handleDeny}
          disabled={isProcessing}
        >
          {isProcessing ? "Processing..." : "Deny"}
        </button>
        <button
          class="btn btn-approve"
          class:btn-dangerous={currentCommand?.riskLevel === "high"}
          onclick={handleApprove}
          disabled={isProcessing}
        >
          {#if currentCommand?.riskLevel === "high"}
            {isProcessing ? "Processing..." : "Approve Anyway"}
          {:else}
            {isProcessing ? "Processing..." : "Approve"}
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.8);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1100;
    padding: var(--space-lg);
  }

  .dialog {
    width: 700px;
    max-width: 100%;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    background-color: var(--bg-secondary);
    border-radius: 16px;
    border: 1px solid var(--border);
    box-shadow: var(--shadow-lg);
    overflow: hidden;
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-lg);
    border-bottom: 1px solid var(--border);
    background-color: var(--bg-primary);
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: var(--space-md);
  }

  .lock-icon {
    width: 24px;
    height: 24px;
    color: var(--warning);
  }

  .header h2 {
    font-size: 18px;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
  }

  .close-btn {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: transparent;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    color: var(--text-muted);
    transition: all 0.2s ease;
  }

  .close-btn:hover {
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .close-btn svg {
    width: 18px;
    height: 18px;
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-lg);
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .risk-badge {
    display: inline-flex;
    align-self: flex-start;
    padding: 4px 12px;
    border-radius: 4px;
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.05em;
  }

  .risk-badge.risk-normal {
    background-color: var(--success-bg, rgba(34, 197, 94, 0.1));
    color: var(--success);
    border: 1px solid var(--success);
  }

  .risk-badge.risk-suspicious {
    background-color: var(--warning-bg, rgba(234, 179, 8, 0.1));
    color: var(--warning);
    border: 1px solid var(--warning);
  }

  .risk-badge.risk-high {
    background-color: var(--error-bg, rgba(239, 68, 68, 0.1));
    color: var(--error);
    border: 1px solid var(--error);
  }

  .info-row {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    font-size: 14px;
  }

  .info-row .label {
    color: var(--text-muted);
    min-width: 100px;
  }

  .info-row .value {
    color: var(--text-primary);
  }

  .info-row .value.mono {
    font-family: var(--font-mono);
    font-size: 13px;
  }

  .info-row .timer {
    margin-left: auto;
    color: var(--warning);
    font-weight: 600;
  }

  .command-section h3,
  .compound-section h4,
  .inner-command-section h4,
  .warnings-section h4 {
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

  .warnings-section {
    background-color: var(--warning-bg, rgba(234, 179, 8, 0.1));
    border: 1px solid var(--warning);
    border-radius: 8px;
    padding: var(--space-md);
  }

  .warnings-list {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .warning-item {
    display: flex;
    align-items: flex-start;
    gap: var(--space-sm);
    color: var(--warning);
    font-size: 14px;
    margin-bottom: var(--space-xs);
  }

  .warning-item:last-child {
    margin-bottom: 0;
  }

  .warning-item svg {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
    margin-top: 2px;
  }

  .danger-warning {
    display: flex;
    align-items: flex-start;
    gap: var(--space-md);
    background-color: var(--error-bg, rgba(239, 68, 68, 0.1));
    border: 2px solid var(--error);
    border-radius: 8px;
    padding: var(--space-md);
  }

  .danger-warning svg {
    width: 24px;
    height: 24px;
    color: var(--error);
    flex-shrink: 0;
  }

  .danger-text {
    color: var(--error);
    font-size: 14px;
    line-height: 1.5;
  }

  .scope-option {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
    padding: var(--space-md);
    background-color: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 8px;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    cursor: pointer;
    font-size: 14px;
    color: var(--text-primary);
  }

  .checkbox-label input {
    width: 16px;
    height: 16px;
    cursor: pointer;
  }

  .parent-cmd {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-muted);
    margin-left: 24px;
  }

  .error-message {
    background-color: var(--error-bg, rgba(239, 68, 68, 0.1));
    border: 1px solid var(--error);
    border-radius: 8px;
    padding: var(--space-md);
    color: var(--error);
    font-size: 14px;
  }

  .footer {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-md);
    padding: var(--space-lg);
    border-top: 1px solid var(--border);
    background-color: var(--bg-primary);
  }

  .btn {
    padding: 10px 24px;
    font-size: 14px;
    font-weight: 600;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s ease;
    border: none;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-deny {
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
    border: 1px solid var(--border);
  }

  .btn-deny:hover:not(:disabled) {
    background-color: var(--bg-secondary);
    border-color: var(--text-muted);
  }

  .btn-approve {
    background-color: var(--success);
    color: white;
  }

  .btn-approve:hover:not(:disabled) {
    opacity: 0.9;
  }

  .btn-approve.btn-dangerous {
    background-color: var(--error);
  }

  /* Animations */
  :global(.animate-fade-in) {
    animation: fadeIn 0.2s ease-out;
  }

  :global(.animate-slide-up) {
    animation: slideUp 0.3s ease-out;
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  @keyframes slideUp {
    from {
      opacity: 0;
      transform: translateY(20px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
