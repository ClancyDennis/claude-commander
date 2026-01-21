<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { homeDir } from "@tauri-apps/api/path";
  import { onMount } from "svelte";
  import { addAgent, selectedAgentId } from "../stores/agents";
  import { addPipeline, openPipeline } from "../stores/pipelines";
  import { defaultSettings } from "../stores/pipelineSettings";
  import {
    createSingleAgent,
    createCustomPipeline,
    createAutoPipeline,
  } from "../services/agentCreation";
  import { useSkillGeneration } from "../hooks/useSkillGeneration.svelte";

  import CreationTypeSelector from "./new-agent/CreationTypeSelector.svelte";
  import RepoSelector from "./new-agent/RepoSelector.svelte";
  import InstructionSelector from "./new-agent/InstructionSelector.svelte";
  import PipelineSettingsForm from "./new-agent/PipelineSettingsForm.svelte";
  import HelpTip from "./new-agent/HelpTip.svelte";

  let { onClose }: { onClose: () => void } = $props();

  // Form state
  let workingDir = $state("");
  let githubUrl = $state("");
  let pipelineTask = $state("");
  let creationType = $state<'agent' | 'pipeline' | 'auto-pipeline'>('agent');
  let isCreating = $state(false);
  let error = $state("");
  let selectedInstructions = $state<Set<string>>(new Set());
  let pipelineSettings = $state({ ...defaultSettings });

  // Skill generation tracking via hook
  const skillGen = useSkillGeneration();

  onMount(async () => {
    // Set working directory to user's home directory
    try {
      workingDir = await homeDir();
    } catch (e) {
      console.error("Failed to get home directory:", e);
    }

    skillGen.start();
    return () => skillGen.cleanup();
  });

  async function selectDirectory() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        defaultPath: workingDir,
      });

      if (selected) {
        workingDir = selected as string;
      }
    } catch (e) {
      console.error("Failed to open directory picker:", e);
    }
  }

  async function handleCreate() {
    if (!workingDir.trim()) {
      error = "Please select a working directory";
      return;
    }

    if ((creationType === 'pipeline' || creationType === 'auto-pipeline') && !pipelineTask.trim()) {
      error = "Please provide a task description for the pipeline";
      return;
    }

    isCreating = true;
    error = "";

    try {
      if (creationType === 'pipeline') {
        const { pipeline, pipelineId } = await createCustomPipeline({
          workingDir,
          task: pipelineTask,
          settings: pipelineSettings,
          instructions: Array.from(selectedInstructions),
        });
        addPipeline(pipeline);
        openPipeline(pipelineId);
        onClose();
      } else if (creationType === 'auto-pipeline') {
        await createAutoPipeline({
          workingDir,
          task: pipelineTask,
          instructions: Array.from(selectedInstructions),
        });
        onClose();
      } else {
        const { agent, agentId } = await createSingleAgent({
          workingDir,
          githubUrl: githubUrl.trim() || undefined,
          instructions: Array.from(selectedInstructions),
        });
        addAgent(agent);
        selectedAgentId.set(agentId);
        onClose();
      }
    } catch (e) {
      error = String(e);
      isCreating = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
       onClose();
    } else if (e.key === "Enter" && workingDir.trim() && !e.shiftKey) {
       if (document.activeElement?.tagName !== 'TEXTAREA') {
          handleCreate();
       }
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay animate-fade-in" onclick={onClose}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="dialog animate-slide-up"
    onclick={(e) => e.stopPropagation()}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <header>
      <div class="header-content">
        <div class="dialog-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="3"/>
            <path d="M12 1v4M12 19v4M1 12h4M19 12h4"/>
          </svg>
        </div>
        <div>
          <h2>Deploy New Agent</h2>
          <p class="subtitle">Launch a new agent into the field</p>
        </div>
      </div>
      <button class="close-btn" onclick={onClose} aria-label="Close dialog">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18"/>
          <line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    </header>

    <div class="content">
      <!-- Top Section: Creation Type -->
      <CreationTypeSelector bind:creationType {isCreating} />

      <div class="form-grid">
        <!-- Left Column: Configuration -->
        <div class="col-left">
          <label>
            <span class="label-text">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
              </svg>
              Working Directory
              <HelpTip
                text="Where the agent/pipeline will read and write files. Pick the folder you want it to work inside."
              />
            </span>
            <div class="input-group">
              <input
                type="text"
                bind:value={workingDir}
                placeholder="/path/to/your/project"
                disabled={isCreating}
              />
              <button
                type="button"
                class="browse-btn"
                onclick={selectDirectory}
                disabled={isCreating}
                title="Browse for directory"
              >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
                </svg>
              </button>
            </div>
          </label>

          <RepoSelector bind:githubUrl {isCreating} />

          {#if creationType === 'pipeline'}
            <PipelineSettingsForm bind:settings={pipelineSettings} {isCreating} />
          {/if}
        </div>

        <!-- Right Column: Context -->
        <div class="col-right">
          {#if creationType === 'pipeline' || creationType === 'auto-pipeline'}
            <label>
              <span class="label-text">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
                </svg>
                Task Description
                <HelpTip
                  text="Describe the outcome you want. More context (stack, constraints, files) helps the pipeline do the right thing."
                />
              </span>
              <textarea
                bind:value={pipelineTask}
                placeholder={creationType === 'auto-pipeline'
                  ? "What should Ralph build?"
                  : "Describe the pipeline objective..."}
                rows="4"
                disabled={isCreating}
              ></textarea>
            </label>
          {/if}

          <InstructionSelector 
            {workingDir} 
            bind:selectedInstructions 
            {isCreating} 
          />
        </div>
      </div>

      {#if error}
        <div class="error animate-slide-up">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <line x1="12" y1="8" x2="12" y2="12"/>
            <line x1="12" y1="16" x2="12.01" y2="16"/>
          </svg>
          {error}
        </div>
      {/if}
    </div>

    <footer>
      {#if skillGen.state.active}
        <div class="skill-generation-progress">
          <div class="skill-gen-header">
            <span class="skill-gen-spinner"></span>
            <span>Generating skills...</span>
          </div>
          <div class="skill-gen-details">
            <span class="skill-gen-count">{skillGen.state.completed + skillGen.state.skipped}/{skillGen.state.total}</span>
            {#if skillGen.state.currentFile}
              <span class="skill-gen-current" title={skillGen.state.currentFile}>
                {skillGen.state.currentFile.length > 30
                  ? '...' + skillGen.state.currentFile.slice(-27)
                  : skillGen.state.currentFile}
              </span>
            {/if}
          </div>
          <div class="skill-gen-bar">
            <div
              class="skill-gen-bar-fill"
              style="width: {((skillGen.state.completed + skillGen.state.skipped) / skillGen.state.total) * 100}%"
            ></div>
          </div>
        </div>
      {:else}
        <button class="secondary" onclick={onClose} disabled={isCreating}>
          Cancel
        </button>
        <button
          class="primary"
          onclick={handleCreate}
          disabled={isCreating || !workingDir.trim() || ((creationType === 'pipeline' || creationType === 'auto-pipeline') && !pipelineTask.trim())}
        >
          {#if isCreating}
            <span class="spinner"></span>
            {creationType === 'pipeline' ? 'Starting...' : creationType === 'auto-pipeline' ? 'Starting...' : 'Creating...'}
          {:else}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              {#if creationType === 'pipeline'}
                <polygon points="5 3 19 12 5 21 5 3"/>
              {:else}
                <line x1="12" y1="5" x2="12" y2="19"/>
                <line x1="5" y1="12" x2="19" y2="12"/>
              {/if}
            </svg>
            {creationType === 'pipeline' ? 'Start Pipeline' : creationType === 'auto-pipeline' ? 'Start Ralphline' : 'Create Agent'}
          {/if}
        </button>
      {/if}
    </footer>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.75);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    padding: var(--space-lg);
  }

  .dialog {
    width: 900px;
    max-width: 100%;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    background-color: var(--bg-secondary);
    border-radius: 20px;
    border: 1px solid var(--border);
    box-shadow: var(--shadow-lg), 0 0 60px rgba(124, 58, 237, 0.1);
    overflow: hidden;
  }

  header {
    flex-shrink: 0;
    padding: var(--space-lg);
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    border-bottom: 1px solid var(--border);
    background: linear-gradient(180deg, var(--bg-tertiary) 0%, var(--bg-secondary) 100%);
  }

  .header-content {
    display: flex;
    gap: var(--space-md);
    align-items: center;
  }

  .dialog-icon {
    width: 48px;
    height: 48px;
    border-radius: 14px;
    background: linear-gradient(135deg, var(--accent) 0%, #9333ea 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 4px 12px var(--accent-glow);
  }

  .dialog-icon svg {
    width: 24px;
    height: 24px;
    color: white;
  }

  h2 {
    font-size: 20px;
    font-weight: 700;
    margin-bottom: 2px;
  }

  .subtitle {
    font-size: 14px;
    color: var(--text-muted);
  }

  .close-btn {
    background: var(--bg-elevated);
    width: 40px;
    height: 40px;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 12px;
    color: var(--text-secondary);
  }

  .close-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .close-btn svg {
    width: 20px;
    height: 20px;
  }

  .content {
    padding: var(--space-lg);
    overflow-y: auto;
  }

  .form-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-lg);
  }

  .col-left, .col-right {
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
  }

  label {
    display: block;
  }

  .label-text {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    margin-bottom: var(--space-sm);
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .label-text svg {
    width: 18px;
    height: 18px;
    color: var(--accent);
  }

  .input-group {
    display: flex;
    gap: var(--space-sm);
  }

  .input-group input {
    flex: 1;
    padding: var(--space-md);
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 10px;
    color: var(--text-primary);
    font-size: 14px;
    transition: all 0.2s ease;
  }

  .input-group input:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-glow);
  }

  .browse-btn {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: 0 var(--space-md);
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 10px;
    color: var(--text-secondary);
    font-size: 14px;
    font-weight: 500;
    white-space: nowrap;
    transition: all 0.2s ease;
  }

  .browse-btn:hover:not(:disabled) {
    background: var(--bg-tertiary);
    border-color: var(--accent);
    color: var(--text-primary);
  }

  .browse-btn svg {
    width: 16px;
    height: 16px;
  }

  .error {
    margin-top: var(--space-md);
    padding: var(--space-md);
    background-color: var(--error-glow);
    border: 1px solid var(--error);
    border-radius: 12px;
    color: var(--error);
    font-size: 14px;
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .error svg {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
  }

  footer {
    flex-shrink: 0;
    padding: var(--space-lg);
    display: flex;
    justify-content: flex-end;
    gap: var(--space-md);
    border-top: 1px solid var(--border);
    background-color: var(--bg-tertiary);
  }

  footer button {
    min-width: 120px;
    padding: 10px 20px;
    border-radius: 10px;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    border: none;
  }

  .secondary {
    background: var(--bg-elevated);
    color: var(--text-secondary);
    border: 1px solid var(--border);
  }

  .secondary:hover:not(:disabled) {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .primary {
    background: var(--accent);
    color: white;
  }

  .primary:hover:not(:disabled) {
    background: var(--accent-hover, #7c3aed);
    transform: scale(1.02);
  }

  button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .spinner {
    width: 18px;
    height: 18px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  textarea {
    width: 100%;
    padding: var(--space-md);
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 10px;
    color: var(--text-primary);
    font-size: 14px;
    font-family: inherit;
    line-height: 1.5;
    resize: vertical;
    transition: all 0.2s ease;
  }

  textarea:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-glow);
  }

  textarea:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  /* Skill generation progress styles */
  .skill-generation-progress {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
    padding: var(--space-sm) var(--space-md);
    background: linear-gradient(135deg, rgba(124, 58, 237, 0.1) 0%, rgba(147, 51, 234, 0.05) 100%);
    border: 1px solid rgba(124, 58, 237, 0.3);
    border-radius: 12px;
    animation: pulse-border 2s ease-in-out infinite;
  }

  @keyframes pulse-border {
    0%, 100% {
      border-color: rgba(124, 58, 237, 0.3);
      box-shadow: 0 0 0 0 rgba(124, 58, 237, 0);
    }
    50% {
      border-color: rgba(124, 58, 237, 0.6);
      box-shadow: 0 0 12px 2px rgba(124, 58, 237, 0.15);
    }
  }

  .skill-gen-header {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    font-size: 14px;
    font-weight: 600;
    color: var(--accent);
  }

  .skill-gen-spinner {
    width: 16px;
    height: 16px;
    border: 2px solid rgba(124, 58, 237, 0.3);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  .skill-gen-details {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    font-size: 12px;
    color: var(--text-secondary);
  }

  .skill-gen-count {
    font-weight: 600;
    color: var(--text-primary);
    font-variant-numeric: tabular-nums;
  }

  .skill-gen-current {
    color: var(--text-muted);
    font-family: var(--font-mono, monospace);
    font-size: 11px;
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .skill-gen-bar {
    height: 4px;
    background: rgba(124, 58, 237, 0.2);
    border-radius: 2px;
    overflow: hidden;
  }

  .skill-gen-bar-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--accent) 0%, #9333ea 100%);
    border-radius: 2px;
    transition: width 0.3s ease;
    position: relative;
  }

  .skill-gen-bar-fill::after {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(
      90deg,
      transparent 0%,
      rgba(255, 255, 255, 0.3) 50%,
      transparent 100%
    );
    animation: shimmer 1.5s ease-in-out infinite;
  }

  @keyframes shimmer {
    0% { transform: translateX(-100%); }
    100% { transform: translateX(100%); }
  }
</style>
