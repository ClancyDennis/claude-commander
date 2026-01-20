<script lang="ts">
  import HelpTip from "./HelpTip.svelte";
  let { creationType = $bindable(), isCreating }: { creationType: 'agent' | 'pipeline' | 'auto-pipeline', isCreating: boolean } = $props();

  const agentTypeImage = new URL("../../assets/creation-type/agent.svg", import.meta.url).href;
  const pipelineTypeImage = new URL("../../assets/creation-type/pipeline.svg", import.meta.url).href;
  const autoPipelineTypeImage = new URL(
    "../../assets/creation-type/auto-pipeline.svg",
    import.meta.url,
  ).href;
</script>

<div class="section-type">
  <div class="radio-group type-grid">
    <label class="radio-option" class:selected={creationType === 'agent'}>
      <input type="radio" bind:group={creationType} value="agent" disabled={isCreating} />
      <div class="corner-help">
        <HelpTip
          placement="top"
          text="Starts one agent focused on a single ongoing task in the selected working directory."
        />
      </div>
      <div class="type-image" aria-hidden="true">
        <img src={agentTypeImage} alt="" />
      </div>
      <div class="radio-content">
        <strong>Single Agent</strong>
        <span>Focused mission</span>
      </div>
    </label>
    <label class="radio-option" class:selected={creationType === 'pipeline'}>
      <input type="radio" bind:group={creationType} value="pipeline" disabled={isCreating} />
      <div class="corner-help">
        <HelpTip
          placement="top"
          text="Runs a multi-phase workflow (planning/execution/verification) with configurable checkpoints."
        />
      </div>
      <div class="type-image" aria-hidden="true">
        <img src={pipelineTypeImage} alt="" />
      </div>
      <div class="radio-content">
        <strong>Pipeline</strong>
        <span>Multi-phase</span>
      </div>
    </label>
    <label class="radio-option" class:selected={creationType === 'auto-pipeline'}>
      <input type="radio" bind:group={creationType} value="auto-pipeline" disabled={isCreating} />
      <div class="corner-help">
        <HelpTip
          placement="top"
          text="Starts the automated pipeline (Ralph) that tries to help end-to-end with minimal configuration."
        />
      </div>
      <div class="type-image" aria-hidden="true">
        <img src={autoPipelineTypeImage} alt="" />
      </div>
      <div class="radio-content">
        <strong>Ralphline</strong>
        <span>I'm helping!</span>
      </div>
    </label>
  </div>
</div>

<style>
  .type-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-md);
    margin-bottom: var(--space-lg);
  }

  .radio-option {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: var(--space-sm);
    padding: var(--space-md);
    background: var(--bg-elevated);
    border: 2px solid var(--border);
    border-radius: 12px;
    cursor: pointer;
    transition: all 0.2s ease;
    height: 100%;
  }

  .radio-option:hover:not(:has(input:disabled)) {
    background: var(--bg-tertiary);
    border-color: var(--accent);
  }

  .radio-option.selected {
    background: rgba(124, 58, 237, 0.1);
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-glow);
  }

  .radio-option input[type="radio"] {
    position: absolute;
    inset: 0;
    opacity: 0;
    margin: 0;
    pointer-events: none;
  }

  .radio-option:focus-within {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-glow);
  }

  .type-image {
    width: 56px;
    height: 56px;
    border-radius: 16px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    display: grid;
    place-items: center;
  }

  .type-image img {
    width: 32px;
    height: 32px;
    display: block;
  }

  .radio-option.selected .type-image {
    background: rgba(124, 58, 237, 0.12);
    border-color: var(--accent);
    box-shadow: 0 0 0 3px rgba(124, 58, 237, 0.1);
  }

  .radio-content {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .corner-help {
    position: absolute;
    bottom: 8px;
    right: 8px;
    z-index: 10;
  }

  .radio-content strong {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .radio-content span {
    font-size: 13px;
    color: var(--text-muted);
    line-height: 1.4;
  }

  .radio-option.selected .radio-content strong {
    color: var(--accent);
  }
</style>
