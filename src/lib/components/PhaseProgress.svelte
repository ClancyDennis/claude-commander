<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";

  interface CheckpointResult {
    passed: boolean;
    message: string;
    details?: any;
    timestamp: number;
  }

  interface Phase {
    name: string;
    description: string;
    task_ids: string[];
    checkpoint: string;
    status: string;
    checkpoint_result?: CheckpointResult;
  }

  interface Pipeline {
    id: string;
    user_request: string;
    phases: Phase[];
    current_phase_index: number;
    status: string;
    created_at: number;
    completed_at?: number;
  }

  let { pipelineId }: { pipelineId: string } = $props();
  let pipeline = $state<Pipeline | null>(null);
  let refreshInterval: number;

  async function loadPipeline() {
    try {
      pipeline = await invoke<Pipeline>("get_pipeline", { pipelineId });
    } catch (err) {
      console.error("Failed to load pipeline:", err);
    }
  }

  async function approveCheckpoint(phaseIndex: number, approved: boolean, comment: string = "") {
    try {
      await invoke("approve_pipeline_checkpoint", {
        pipelineId,
        phaseIndex,
        approved,
        comment: comment || null,
      });
      await loadPipeline();
    } catch (err) {
      console.error("Failed to approve checkpoint:", err);
    }
  }

  onMount(() => {
    loadPipeline();
    refreshInterval = setInterval(loadPipeline, 2000);
  });

  onDestroy(() => {
    clearInterval(refreshInterval);
  });

  function getPhaseStatusColor(status: string): string {
    switch (status) {
      case "Completed": return "var(--success)";
      case "Running": return "var(--accent)";
      case "WaitingCheckpoint": return "var(--warning)";
      case "CheckpointFailed": return "var(--error)";
      case "Failed": return "var(--error)";
      default: return "var(--text-muted)";
    }
  }

  function getPhaseIcon(status: string): string {
    switch (status) {
      case "Completed": return "✓";
      case "Running": return "▶";
      case "WaitingCheckpoint": return "⏸";
      case "CheckpointFailed": return "✗";
      case "Failed": return "✗";
      default: return "○";
    }
  }

  function getCheckpointIcon(checkpoint: string): string {
    if (checkpoint === "None") return "";
    if (checkpoint === "HumanReview") return "👤";
    if (checkpoint.startsWith("AutomaticValidation")) return "⚙️";
    if (checkpoint.startsWith("BestOfN")) return "🔍";
    if (checkpoint === "Conditional") return "❓";
    return "🔒";
  }

  let pendingApproval = $state<{ phaseIndex: number; comment: string } | null>(null);
</script>

<div class="phase-progress">
  {#if pipeline}
    <header class="progress-header">
      <div class="header-content">
        <h2>Pipeline Execution</h2>
        <span class="pipeline-status" style="color: {getPhaseStatusColor(pipeline.status)};">
          {pipeline.status}
        </span>
      </div>
      <p class="user-request">{pipeline.user_request}</p>
    </header>

    <div class="phases-timeline">
      {#each pipeline.phases as phase, i (i)}
        <div class="phase-item" class:active={i === pipeline.current_phase_index} class:completed={phase.status === "Completed"}>
          <div class="phase-connector">
            {#if i > 0}
              <div class="connector-line" class:completed={pipeline.phases[i - 1].status === "Completed"}></div>
            {/if}
            <div class="phase-icon" style="background: {getPhaseStatusColor(phase.status)};">
              {getPhaseIcon(phase.status)}
            </div>
            {#if i < pipeline.phases.length - 1}
              <div class="connector-line" class:completed={phase.status === "Completed"}></div>
            {/if}
          </div>

          <div class="phase-content">
            <div class="phase-header">
              <h3>{phase.name}</h3>
              <span class="phase-status" style="color: {getPhaseStatusColor(phase.status)};">
                {phase.status}
              </span>
            </div>
            <p class="phase-description">{phase.description}</p>

            {#if phase.checkpoint !== "None"}
              <div class="checkpoint-info">
                <div class="checkpoint-header">
                  <span class="checkpoint-icon">{getCheckpointIcon(phase.checkpoint)}</span>
                  <span class="checkpoint-label">Checkpoint: {phase.checkpoint}</span>
                </div>

                {#if phase.checkpoint_result}
                  <div class="checkpoint-result" class:passed={phase.checkpoint_result.passed}>
                    <div class="result-header">
                      <span class="result-icon">{phase.checkpoint_result.passed ? "✓" : "✗"}</span>
                      <span class="result-message">{phase.checkpoint_result.message}</span>
                    </div>

                    {#if phase.checkpoint_result.details}
                      <details class="result-details">
                        <summary>View Details</summary>
                        <pre>{JSON.stringify(phase.checkpoint_result.details, null, 2)}</pre>
                      </details>
                    {/if}
                  </div>
                {/if}

                {#if phase.status === "WaitingCheckpoint" && phase.checkpoint === "HumanReview"}
                  <div class="approval-interface">
                    {#if pendingApproval?.phaseIndex === i}
                      <div class="approval-form">
                        <textarea
                          bind:value={pendingApproval.comment}
                          placeholder="Add optional comment..."
                          rows="3"
                        ></textarea>
                        <div class="approval-actions">
                          <button
                            class="btn btn-success"
                            onclick={() => {
                              if (pendingApproval) {
                                approveCheckpoint(i, true, pendingApproval.comment);
                                pendingApproval = null;
                              }
                            }}
                          >
                            Approve
                          </button>
                          <button
                            class="btn btn-error"
                            onclick={() => {
                              if (pendingApproval) {
                                approveCheckpoint(i, false, pendingApproval.comment);
                                pendingApproval = null;
                              }
                            }}
                          >
                            Reject
                          </button>
                          <button
                            class="btn btn-secondary"
                            onclick={() => pendingApproval = null}
                          >
                            Cancel
                          </button>
                        </div>
                      </div>
                    {:else}
                      <button
                        class="btn btn-primary"
                        onclick={() => pendingApproval = { phaseIndex: i, comment: "" }}
                      >
                        Review Checkpoint
                      </button>
                    {/if}
                  </div>
                {/if}
              </div>
            {/if}

            {#if phase.task_ids.length > 0}
              <div class="phase-tasks">
                <span class="tasks-label">Tasks: {phase.task_ids.join(", ")}</span>
              </div>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <div class="loading">Loading pipeline...</div>
  {/if}
</div>

<style>
  .phase-progress {
    padding: var(--space-lg);
    background: var(--bg-secondary);
    border-radius: 12px;
    border: 1px solid var(--border);
    height: 100%;
    overflow-y: auto;
  }

  .progress-header {
    margin-bottom: var(--space-xl);
    padding-bottom: var(--space-lg);
    border-bottom: 1px solid var(--border);
  }

  .header-content {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-sm);
  }

  .progress-header h2 {
    margin: 0;
    font-size: 24px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .pipeline-status {
    padding: 6px 12px;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 600;
    text-transform: uppercase;
    background: var(--bg-tertiary);
  }

  .user-request {
    margin: 0;
    font-size: 14px;
    color: var(--text-secondary);
  }

  .phases-timeline {
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
  }

  .phase-item {
    display: flex;
    gap: var(--space-md);
    opacity: 0.6;
    transition: opacity 0.3s ease;
  }

  .phase-item.active {
    opacity: 1;
  }

  .phase-item.completed {
    opacity: 0.8;
  }

  .phase-connector {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding-top: 4px;
  }

  .phase-icon {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 18px;
    font-weight: 700;
    color: white;
    flex-shrink: 0;
    box-shadow: 0 0 12px currentColor;
  }

  .connector-line {
    width: 2px;
    flex: 1;
    background: var(--border);
    transition: background 0.3s ease;
  }

  .connector-line.completed {
    background: var(--success);
  }

  .phase-content {
    flex: 1;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: var(--space-md);
  }

  .phase-item.active .phase-content {
    border-color: var(--accent);
    box-shadow: 0 0 16px var(--accent-glow);
  }

  .phase-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-sm);
  }

  .phase-header h3 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .phase-status {
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
  }

  .phase-description {
    margin: 0 0 var(--space-md) 0;
    font-size: 14px;
    color: var(--text-secondary);
  }

  .checkpoint-info {
    margin-top: var(--space-md);
    padding-top: var(--space-md);
    border-top: 1px solid var(--border);
  }

  .checkpoint-header {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    margin-bottom: var(--space-sm);
  }

  .checkpoint-icon {
    font-size: 18px;
  }

  .checkpoint-label {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-muted);
  }

  .checkpoint-result {
    padding: var(--space-md);
    border-radius: 8px;
    background: var(--bg-secondary);
    border: 2px solid var(--border);
    margin-top: var(--space-md);
  }

  .checkpoint-result.passed {
    border-color: var(--success);
    background: rgba(34, 197, 94, 0.1);
  }

  .checkpoint-result:not(.passed) {
    border-color: var(--error);
    background: rgba(239, 68, 68, 0.1);
  }

  .result-header {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .result-icon {
    font-size: 20px;
    font-weight: 700;
  }

  .checkpoint-result.passed .result-icon {
    color: var(--success);
  }

  .checkpoint-result:not(.passed) .result-icon {
    color: var(--error);
  }

  .result-message {
    font-size: 14px;
    color: var(--text-primary);
  }

  .result-details {
    margin-top: var(--space-sm);
  }

  .result-details summary {
    cursor: pointer;
    font-size: 12px;
    color: var(--text-muted);
    user-select: none;
  }

  .result-details pre {
    margin-top: var(--space-sm);
    padding: var(--space-sm);
    background: var(--bg-secondary);
    border-radius: 6px;
    font-size: 11px;
    overflow-x: auto;
  }

  .approval-interface {
    margin-top: var(--space-md);
  }

  .approval-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .approval-form textarea {
    width: 100%;
    padding: var(--space-sm);
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-primary);
    font-size: 14px;
    font-family: inherit;
    resize: vertical;
  }

  .approval-actions {
    display: flex;
    gap: var(--space-sm);
  }

  .phase-tasks {
    margin-top: var(--space-sm);
    padding-top: var(--space-sm);
    border-top: 1px solid var(--border);
  }

  .tasks-label {
    font-size: 12px;
    color: var(--text-muted);
  }

  .btn {
    padding: 8px 16px;
    border: none;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .btn-primary {
    background: var(--accent);
    color: white;
  }

  .btn-primary:hover {
    background: var(--accent-hover, var(--accent));
    transform: scale(1.02);
  }

  .btn-success {
    background: var(--success);
    color: white;
  }

  .btn-success:hover {
    background: #16a34a;
    transform: scale(1.02);
  }

  .btn-error {
    background: var(--error);
    color: white;
  }

  .btn-error:hover {
    background: #dc2626;
    transform: scale(1.02);
  }

  .btn-secondary {
    background: var(--bg-secondary);
    color: var(--text-primary);
    border: 1px solid var(--border);
  }

  .btn-secondary:hover {
    background: var(--bg-tertiary);
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-xl);
    color: var(--text-muted);
  }
</style>
