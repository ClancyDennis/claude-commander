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
  import {
    RiskBadge,
    CommandDisplay,
    WarningsSection,
    ScopeApproval,
    ApprovalButtons,
  } from "./elevated-command";

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

  function handleApproveScopeChange(value: boolean) {
    approveScope = value;
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
        <RiskBadge riskLevel={currentCommand.riskLevel} />

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

        <!-- Command display with compound breakdown and inner command -->
        <CommandDisplay command={currentCommand} />

        <!-- Warnings and danger alerts -->
        <WarningsSection warnings={currentCommand.warnings} riskLevel={currentCommand.riskLevel} />

        <!-- Script scope option -->
        <ScopeApproval
          scriptHash={currentCommand.scriptHash}
          parentCmd={currentCommand.parentCmd}
          {approveScope}
          onApproveScopeChange={handleApproveScopeChange}
        />

        <!-- Error display (inside content for better positioning) -->
        {#if error}
          <div class="error-message">
            {error}
          </div>
        {/if}
      </div>

      <!-- Approval buttons in footer -->
      <ApprovalButtons
        riskLevel={currentCommand.riskLevel}
        {isProcessing}
        error={null}
        onApprove={handleApprove}
        onDeny={handleDeny}
      />
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

  .error-message {
    background-color: var(--error-bg, rgba(239, 68, 68, 0.1));
    border: 1px solid var(--error);
    border-radius: 8px;
    padding: var(--space-md);
    color: var(--error);
    font-size: 14px;
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
