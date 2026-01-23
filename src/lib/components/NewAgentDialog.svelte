<script lang="ts">
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
  import WorkingDirectoryInput from "./new-agent/WorkingDirectoryInput.svelte";
  import RepoSelector from "./new-agent/RepoSelector.svelte";
  import TaskDescriptionInput from "./new-agent/TaskDescriptionInput.svelte";
  import InstructionSelector from "./new-agent/InstructionSelector.svelte";
  import PipelineSettingsForm from "./new-agent/PipelineSettingsForm.svelte";
  import DialogFooter from "./new-agent/DialogFooter.svelte";
  import ExpandableSection from "./ui/expandable-section.svelte";

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

  // Derived validation state
  let canCreate = $derived(
    workingDir.trim() !== '' &&
    ((creationType !== 'pipeline' && creationType !== 'auto-pipeline') || pipelineTask.trim() !== '')
  );

  onMount(async () => {
    try {
      workingDir = await homeDir();
    } catch (e) {
      console.error("Failed to get home directory:", e);
    }

    skillGen.start();
    return () => skillGen.cleanup();
  });

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
    } else if (e.key === "Enter" && canCreate && !e.shiftKey) {
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

      <!-- Essential Fields Section -->
      <div class="essential-fields">
        <WorkingDirectoryInput bind:workingDir {isCreating} />
        <TaskDescriptionInput bind:pipelineTask {creationType} {isCreating} />
      </div>

      <!-- Advanced Options - Progressive Disclosure -->
      <ExpandableSection title="Advanced Options" badge="Optional" class="advanced-section">
        <div class="form-grid">
          <!-- Left Column: Configuration -->
          <div class="col-left">
            <RepoSelector bind:githubUrl {isCreating} />

            {#if creationType === 'pipeline'}
              <PipelineSettingsForm bind:settings={pipelineSettings} {isCreating} />
            {/if}
          </div>

          <!-- Right Column: Context -->
          <div class="col-right">
            <InstructionSelector
              {workingDir}
              bind:selectedInstructions
              {isCreating}
            />
          </div>
        </div>
      </ExpandableSection>

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

    <DialogFooter
      {creationType}
      {isCreating}
      {canCreate}
      skillGenState={skillGen.state}
      onCreate={handleCreate}
      onCancel={onClose}
    />
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    padding: var(--space-6);
  }

  .dialog {
    width: 680px;
    max-width: calc(100vw - var(--space-8));
    max-height: calc(100vh - var(--space-12));
    display: flex;
    flex-direction: column;
    background-color: var(--bg-secondary);
    border-radius: var(--radius-xl);
    border: 1px solid rgba(255, 255, 255, 0.08);
    box-shadow: var(--shadow-lg);
    overflow: hidden;
  }

  header {
    flex-shrink: 0;
    padding: var(--space-5);
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--border);
  }

  .header-content {
    display: flex;
    gap: var(--space-3);
    align-items: center;
    min-width: 0;
  }

  .dialog-icon {
    width: 40px;
    height: 40px;
    border-radius: var(--radius-md);
    background: linear-gradient(135deg, var(--accent-hex) 0%, #d55a42 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .dialog-icon svg {
    width: 20px;
    height: 20px;
    color: white;
  }

  h2 {
    font-size: var(--text-lg);
    font-weight: var(--font-semibold);
    margin-bottom: 1px;
    color: var(--text-primary);
  }

  .subtitle {
    font-size: var(--text-sm);
    color: var(--text-muted);
  }

  .close-btn {
    width: 28px;
    height: 28px;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-muted);
    flex-shrink: 0;
    transition: all var(--transition-fast);
  }

  .close-btn:hover {
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-primary);
  }

  .close-btn svg {
    width: 16px;
    height: 16px;
  }

  .content {
    flex: 1;
    padding: var(--space-5);
    overflow-y: auto;
    overflow-x: hidden;
  }

  .essential-fields {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    margin-bottom: var(--space-4);
  }

  :global(.advanced-section) {
    margin-top: var(--space-2);
  }

  .form-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-4);
    margin-top: var(--space-3);
  }

  @media (max-width: 640px) {
    .form-grid {
      grid-template-columns: 1fr;
    }
  }

  .col-left, .col-right {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    min-width: 0;
  }

  .error {
    margin-top: var(--space-3);
    padding: var(--space-3);
    background-color: rgba(255, 59, 48, 0.1);
    border: 1px solid rgba(255, 59, 48, 0.3);
    border-radius: var(--radius-md);
    color: var(--error);
    font-size: var(--text-sm);
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
  }

  .error svg {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
    margin-top: 1px;
  }
</style>
