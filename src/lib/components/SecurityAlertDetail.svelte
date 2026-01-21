<script lang="ts">
  import { selectedAlertDetail, hideAlertDetail } from "../stores/security";
  import { selectedAgentId } from "../stores/agents";
  import type { ThreatType } from "../types";

  // Threat type configuration with icons and colors
  const threatTypeConfig: Record<ThreatType, { icon: string; color: string; label: string }> = {
    PromptInjection: {
      icon: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z",
      color: "var(--warning)",
      label: "Prompt Injection"
    },
    JailbreakAttempt: {
      icon: "M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z",
      color: "var(--error)",
      label: "Jailbreak Attempt"
    },
    DataExfiltration: {
      icon: "M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4",
      color: "var(--error)",
      label: "Data Exfiltration"
    },
    UnauthorizedAccess: {
      icon: "M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z",
      color: "var(--error)",
      label: "Unauthorized Access"
    },
    MaliciousCodeExecution: {
      icon: "M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4",
      color: "var(--error)",
      label: "Malicious Code Execution"
    },
    PrivilegeEscalation: {
      icon: "M5 11l7-7 7 7M5 19l7-7 7 7",
      color: "var(--error)",
      label: "Privilege Escalation"
    },
    SystemManipulation: {
      icon: "M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z",
      color: "var(--error)",
      label: "System Manipulation"
    },
    SocialEngineering: {
      icon: "M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z",
      color: "var(--warning)",
      label: "Social Engineering"
    },
    ChainedAttack: {
      icon: "M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1",
      color: "var(--error)",
      label: "Chained Attack"
    },
    Unknown: {
      icon: "M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z",
      color: "var(--text-muted)",
      label: "Unknown Threat"
    }
  };

  function formatConfidence(confidence: number): string {
    return `${Math.round(confidence * 100)}%`;
  }

  function getSeverityColor(severity: string): string {
    switch (severity.toLowerCase()) {
      case "critical":
        return "var(--error)";
      case "high":
        return "var(--error)";
      case "medium":
        return "var(--warning)";
      case "low":
        return "var(--accent)";
      default:
        return "var(--text-muted)";
    }
  }

  function handleViewAgent() {
    if ($selectedAlertDetail) {
      selectedAgentId.set($selectedAlertDetail.agentId);
      hideAlertDetail();
    }
  }

  function handleOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      hideAlertDetail();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      hideAlertDetail();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if $selectedAlertDetail}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="overlay animate-fade-in" onclick={handleOverlayClick}>
    <div class="dialog animate-slide-up" role="dialog" aria-modal="true" tabindex="-1">
      <div class="header">
        <h2>Security Alert Details</h2>
        <button class="close-btn" onclick={hideAlertDetail} aria-label="Close dialog">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M6 18L18 6M6 6l12 12" stroke-linecap="round"/>
          </svg>
        </button>
      </div>

      <div class="content">
        <!-- Alert header with severity badge -->
        <div class="alert-header">
          <span class="severity-badge" style="background-color: {getSeverityColor($selectedAlertDetail.severity)}20; color: {getSeverityColor($selectedAlertDetail.severity)}; border-color: {getSeverityColor($selectedAlertDetail.severity)}">
            {$selectedAlertDetail.severity.toUpperCase()}
          </span>
          <h3>{$selectedAlertDetail.title}</h3>
        </div>

        <!-- Overall analysis summary -->
        <section class="summary-section">
          <p class="summary-text">{$selectedAlertDetail.description}</p>
          <div class="confidence-bar">
            <span class="confidence-label">Overall Confidence:</span>
            <span class="confidence-value">{formatConfidence($selectedAlertDetail.overallConfidence)}</span>
          </div>
        </section>

        <!-- Individual threats -->
        {#if $selectedAlertDetail.threats && $selectedAlertDetail.threats.length > 0}
          <div class="threats-container">
            {#each $selectedAlertDetail.threats as threat}
              <div class="threat-card">
                <div class="threat-header">
                  <div class="threat-type">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="color: {threatTypeConfig[threat.threatType]?.color || 'var(--text-muted)'}">
                      <path d={threatTypeConfig[threat.threatType]?.icon || threatTypeConfig.Unknown.icon} stroke-linecap="round" stroke-linejoin="round"/>
                    </svg>
                    <span style="color: {threatTypeConfig[threat.threatType]?.color || 'var(--text-muted)'}">
                      {threatTypeConfig[threat.threatType]?.label || threat.threatType}
                    </span>
                  </div>
                  <span class="threat-confidence">{formatConfidence(threat.confidence)}</span>
                </div>

                <div class="threat-section">
                  <h4>Analysis</h4>
                  <p>{threat.explanation}</p>
                </div>

                {#if threat.evidence && threat.evidence.length > 0}
                  <div class="threat-section">
                    <h4>Evidence</h4>
                    <ul>
                      {#each threat.evidence as item}
                        <li>{item}</li>
                      {/each}
                    </ul>
                  </div>
                {/if}

                {#if threat.mitigations && threat.mitigations.length > 0}
                  <div class="threat-section mitigations">
                    <h4>Recommended Actions</h4>
                    <ul>
                      {#each threat.mitigations as item}
                        <li>{item}</li>
                      {/each}
                    </ul>
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        {/if}

        <!-- Metadata -->
        <div class="metadata">
          <span>Alert ID: <code>{$selectedAlertDetail.alertId}</code></span>
          <span>Detected: {$selectedAlertDetail.timestamp.toLocaleString()}</span>
        </div>
      </div>

      <div class="footer">
        <button class="btn-secondary" onclick={hideAlertDetail}>Close</button>
        <button class="btn-primary" onclick={handleViewAgent}>View Agent</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.75);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 300;
    padding: var(--space-lg);
  }

  .dialog {
    width: 700px;
    max-width: 100%;
    max-height: 85vh;
    display: flex;
    flex-direction: column;
    background-color: var(--bg-secondary);
    border-radius: 16px;
    border: 1px solid var(--border);
    box-shadow: var(--shadow-lg), 0 0 60px rgba(239, 68, 68, 0.2);
    overflow: hidden;
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-lg);
    border-bottom: 1px solid var(--border);
  }

  .header h2 {
    font-size: 20px;
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
    gap: var(--space-lg);
  }

  .alert-header {
    display: flex;
    align-items: center;
    gap: var(--space-md);
  }

  .severity-badge {
    padding: 4px 10px;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.05em;
    border-radius: 6px;
    border: 1px solid;
  }

  .alert-header h3 {
    font-size: 18px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .summary-section {
    padding: var(--space-md);
    background-color: var(--bg-tertiary);
    border-radius: 12px;
    border: 1px solid var(--border);
  }

  .summary-text {
    font-size: 14px;
    color: var(--text-secondary);
    line-height: 1.6;
    margin: 0 0 var(--space-md) 0;
  }

  .confidence-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-top: var(--space-sm);
    border-top: 1px solid var(--border);
  }

  .confidence-label {
    font-size: 13px;
    color: var(--text-muted);
    font-weight: 500;
  }

  .confidence-value {
    font-size: 14px;
    font-weight: 600;
    color: var(--accent);
  }

  .threats-container {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .threat-card {
    padding: var(--space-md);
    background-color: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 12px;
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .threat-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-bottom: var(--space-sm);
    border-bottom: 1px solid var(--border);
  }

  .threat-type {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    font-size: 14px;
    font-weight: 600;
  }

  .threat-type svg {
    width: 18px;
    height: 18px;
  }

  .threat-confidence {
    font-size: 13px;
    color: var(--text-muted);
    font-weight: 500;
  }

  .threat-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .threat-section h4 {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-secondary);
    margin: 0;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .threat-section p {
    font-size: 14px;
    color: var(--text-primary);
    line-height: 1.6;
    margin: 0;
  }

  .threat-section ul {
    margin: 0;
    padding-left: var(--space-lg);
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }

  .threat-section li {
    font-size: 14px;
    color: var(--text-primary);
    line-height: 1.5;
  }

  .threat-section.mitigations li {
    color: var(--accent);
  }

  .metadata {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-md);
    padding: var(--space-md);
    background-color: var(--bg-tertiary);
    border-radius: 8px;
    font-size: 12px;
    color: var(--text-muted);
  }

  .metadata code {
    font-family: monospace;
    font-size: 11px;
    padding: 2px 6px;
    background-color: var(--bg-elevated);
    border-radius: 4px;
    color: var(--text-secondary);
  }

  .footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: var(--space-sm);
    padding: var(--space-lg);
    border-top: 1px solid var(--border);
  }

  .btn-secondary {
    padding: 8px 16px;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-secondary);
    background-color: transparent;
    border: 1px solid var(--border);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .btn-secondary:hover {
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .btn-primary {
    padding: 8px 16px;
    font-size: 14px;
    font-weight: 600;
    color: white;
    background-color: var(--accent);
    border: 1px solid var(--accent);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .btn-primary:hover {
    background-color: var(--accent-hover);
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
