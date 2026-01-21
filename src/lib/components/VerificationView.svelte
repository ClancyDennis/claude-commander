<script lang="ts">
  import HelpTip from "./new-agent/HelpTip.svelte";

  interface AgentResult {
    agent_id: string;
    output: string;
    confidence?: number;
    execution_time: number;
    validation_passed: boolean;
  }

  interface VerificationResult {
    selected_result: AgentResult;
    confidence: number;
    all_results: AgentResult[];
    fusion_reasoning: string;
    verification_time: number;
  }

  let { result }: { result: VerificationResult } = $props();

  // Track selected index with state, track result changes to reset it
  let selectedIndex = $state(0);
  let showAllResults = $state(false);

  // Use plain object to track previous result to avoid reactive loops in Svelte 5
  const viewState = { lastResultId: null as string | null };

  // Update selected index when result changes (use plain object tracking)
  $effect(() => {
    const currentResultId = result?.selected_result?.agent_id ?? null;
    if (result && currentResultId !== viewState.lastResultId) {
      viewState.lastResultId = currentResultId;
      // Find index of selected result
      const newIndex = result.all_results.findIndex(
        r => r.agent_id === result.selected_result.agent_id
      );
      selectedIndex = newIndex >= 0 ? newIndex : 0;
    }
  });

  function selectResult(index: number) {
    selectedIndex = index;
  }
</script>

<div class="verification-view">
  {#if result}
    <header class="verification-header">
      <h2>Verification Results</h2>
      <div class="confidence-badge" class:high={result.confidence > 0.8}>
        Confidence: {(result.confidence * 100).toFixed(0)}% <HelpTip text="Overall confidence based on agent agreement and validation outcomes." placement="left" />
      </div>
    </header>

    <div class="verification-meta">
      <div class="meta-item">
        <span class="meta-label">Strategy:</span>
        <span class="meta-value">{result.fusion_reasoning}</span>
      </div>
      <div class="meta-item">
        <span class="meta-label">Total Time:</span>
        <span class="meta-value">{result.verification_time.toFixed(2)}s</span>
      </div>
      <div class="meta-item">
        <span class="meta-label">Agents Used:</span>
        <span class="meta-value">{result.all_results.length}</span>
      </div>
    </div>

    <div class="selected-result">
      <h3>Selected Result</h3>
      <div class="result-card selected">
        <div class="result-header">
          <span class="agent-badge">{result.selected_result.agent_id.substring(0, 8)}</span>
          {#if result.selected_result.confidence}
            <span class="confidence">{(result.selected_result.confidence * 100).toFixed(0)}%</span>
          {/if}
        </div>
        <pre class="result-output">{result.selected_result.output}</pre>
        <div class="result-footer">
          <span>⏱️ {result.selected_result.execution_time.toFixed(2)}s</span>
          {#if result.selected_result.validation_passed}
            <span class="validation-passed">✅ Validated</span>
          {:else}
            <span class="validation-failed">❌ Failed validation</span>
          {/if}
        </div>
      </div>
    </div>

    <button class="toggle-all-btn" onclick={() => showAllResults = !showAllResults}>
      {showAllResults ? "Hide" : "Show"} All Results ({result.all_results.length})
    </button>

    {#if showAllResults}
      <div class="all-results">
        <h3>All Results</h3>
        <div class="results-grid">
          {#each result.all_results as agentResult, i (agentResult.agent_id)}
            <div class="result-card" class:selected={i === selectedIndex} onclick={() => selectResult(i)}>
              <div class="result-header">
                <span class="agent-badge">{agentResult.agent_id.substring(0, 8)}</span>
                {#if agentResult.confidence}
                  <span class="confidence">{(agentResult.confidence * 100).toFixed(0)}%</span>
                {/if}
              </div>
              <pre class="result-output">{agentResult.output.substring(0, 200)}{agentResult.output.length > 200 ? "..." : ""}</pre>
              <div class="result-footer">
                <span>⏱️ {agentResult.execution_time.toFixed(2)}s</span>
                {#if agentResult.validation_passed}
                  <span class="validation-passed">✅</span>
                {:else}
                  <span class="validation-failed">❌</span>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  {:else}
    <div class="empty-state">
      <p>No verification results available</p>
    </div>
  {/if}
</div>

<style>
  .verification-view {
    padding: var(--space-lg);
    background: var(--bg-secondary);
    border-radius: 12px;
    border: 1px solid var(--border);
  }

  .verification-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-lg);
  }

  .verification-header h2 {
    margin: 0;
    font-size: 20px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .confidence-badge {
    padding: 6px 12px;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 600;
    background: rgba(251, 191, 36, 0.2);
    color: #f59e0b;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .confidence-badge.high {
    background: rgba(34, 197, 94, 0.2);
    color: var(--success);
  }

  .verification-meta {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: var(--space-md);
    margin-bottom: var(--space-xl);
  }

  .meta-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .meta-label {
    font-size: 12px;
    color: var(--text-muted);
  }

  .meta-value {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .selected-result h3,
  .all-results h3 {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: var(--space-md);
  }

  .result-card {
    background: var(--bg-tertiary);
    border: 2px solid var(--border);
    border-radius: 10px;
    padding: var(--space-md);
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .result-card:hover {
    border-color: var(--accent);
  }

  .result-card.selected {
    border-color: var(--success);
    box-shadow: 0 0 16px rgba(34, 197, 94, 0.2);
  }

  .result-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-sm);
  }

  .agent-badge {
    padding: 4px 8px;
    border-radius: 6px;
    font-size: 11px;
    font-weight: 700;
    font-family: monospace;
    background: var(--accent-glow);
    color: var(--accent);
  }

  .confidence {
    font-size: 13px;
    font-weight: 600;
    color: var(--success);
  }

  .result-output {
    margin: var(--space-sm) 0;
    padding: var(--space-sm);
    background: var(--bg-secondary);
    border-radius: 6px;
    font-size: 12px;
    font-family: monospace;
    color: var(--text-secondary);
    white-space: pre-wrap;
    overflow-x: auto;
    max-height: 200px;
    overflow-y: auto;
  }

  .result-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 12px;
    color: var(--text-muted);
  }

  .validation-passed {
    color: var(--success);
  }

  .validation-failed {
    color: var(--error);
  }

  .toggle-all-btn {
    width: 100%;
    padding: var(--space-md);
    margin: var(--space-lg) 0;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 8px;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .toggle-all-btn:hover {
    background: var(--accent-glow);
    border-color: var(--accent);
  }

  .results-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: var(--space-md);
  }

  .empty-state {
    text-align: center;
    padding: var(--space-xl);
    color: var(--text-muted);
  }
</style>
