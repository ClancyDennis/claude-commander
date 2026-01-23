<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import type { GeneratedSkill, SkillContent, InstructionFileInfo } from "../../types";
  import HelpTip from "./HelpTip.svelte";
  import InstructionItem from "./InstructionItem.svelte";
  import InstructionEditorModal from "../InstructionEditorModal.svelte";

  let {
    workingDir,
    selectedInstructions = $bindable(new Set()),
    isCreating
  }: {
    workingDir: string,
    selectedInstructions: Set<string>,
    isCreating: boolean
  } = $props();

  let instructionFiles = $state<Array<InstructionFileInfo>>([]);
  let isLoadingInstructions = $state(false);
  let showInstructionSelector = $state(false);

  // Editor Modal State
  let showEditorModal = $state(false);
  let editingFile = $state<InstructionFileInfo | null>(null);
  let error = $state("");

  // Skill Generation State
  let generatingSkillForFile = $state<string | null>(null);
  let generatedSkills = $state<Map<string, GeneratedSkill>>(new Map());

  // Load instruction files on mount (from ~/.instructions/, not workingDir)
  onMount(() => {
    loadInstructionFiles();
  });

  // Re-check existing skills when workingDir changes (skills are per-project)
  $effect(() => {
    if (workingDir && workingDir.trim() && instructionFiles.length > 0) {
      checkExistingSkills();
    }
  });

  async function loadInstructionFiles() {
    try {
      isLoadingInstructions = true;
      const files = await invoke<Array<any>>("list_instruction_files", {
        workingDir: "",
      });
      instructionFiles = files;
    } catch (e) {
      console.error("Failed to load instruction files:", e);
      instructionFiles = [];
    } finally {
      isLoadingInstructions = false;
    }
  }

  async function checkExistingSkills() {
    try {
      const skills = await invoke<GeneratedSkill[]>("list_generated_skills", {
        workingDir,
      });

      for (const skill of skills) {
        for (const file of instructionFiles) {
          const expectedSkillName = file.name
            .replace(/\.(txt|md)$/, '')
            .toLowerCase()
            .replace(/[^a-z0-9]+/g, '-');

          if (skill.skillName === expectedSkillName) {
            file.hasSkill = true;
            file.skillName = skill.skillName;
            generatedSkills.set(file.id, skill);
          }
        }
      }

      instructionFiles = [...instructionFiles];
    } catch (e) {
      console.error("Failed to check existing skills:", e);
    }
  }

  async function generateSkillForInstruction(file: InstructionFileInfo) {
    generatingSkillForFile = file.id;

    try {
      const result = await invoke<GeneratedSkill>("generate_skill_from_instruction", {
        filePath: file.path,
        workingDir: workingDir,
      });

      generatedSkills.set(file.id, result);
      file.hasSkill = true;
      file.skillName = result.skillName;
      instructionFiles = [...instructionFiles];
    } catch (err) {
      console.error("Skill generation error:", err);
    } finally {
      generatingSkillForFile = null;
    }
  }

  async function viewSkillContent(skillName: string) {
    try {
      const content = await invoke<SkillContent>("get_skill_content", {
        skillName,
        workingDir: workingDir,
      });

      console.log("Skill content:", content);
      alert(`Skill: ${content.skillName}\n\n${content.skillMd.substring(0, 500)}...\n\nFull content logged to console.`);
    } catch (err) {
      console.error("Failed to load skill:", err);
      alert(`Failed to load skill: ${err}`);
    }
  }

  async function deleteSkill(file: InstructionFileInfo) {
    if (!file.skillName) return;

    if (!confirm(`Delete skill "${file.skillName}"? This cannot be undone.`)) {
      return;
    }

    try {
      await invoke("delete_generated_skill", {
        skillName: file.skillName,
        workingDir: workingDir,
      });

      file.hasSkill = false;
      file.skillName = undefined;
      generatedSkills.delete(file.id);
      instructionFiles = [...instructionFiles];
    } catch (err) {
      console.error("Failed to delete skill:", err);
      alert(`Failed to delete skill: ${err}`);
    }
  }

  function openCreateModal() {
    editingFile = null;
    showEditorModal = true;
  }

  function openEditModal(file: InstructionFileInfo) {
    editingFile = file;
    showEditorModal = true;
  }

  function handleModalSaved(filename: string) {
    loadInstructionFiles().then(() => {
      const savedFile = instructionFiles.find(f => f.name === filename);
      if (savedFile) {
        selectedInstructions.add(savedFile.id);
        selectedInstructions = new Set(selectedInstructions);
      }
      showInstructionSelector = true;
    });
  }

  function toggleInstructionSelection(fileId: string) {
    if (selectedInstructions.has(fileId)) {
      selectedInstructions.delete(fileId);
    } else {
      selectedInstructions.add(fileId);
    }
    selectedInstructions = new Set(selectedInstructions);
  }

  async function openInstructionsFolder() {
    try {
      await invoke("open_instructions_directory");
    } catch (e) {
      console.error("Failed to open instructions folder:", e);
    }
  }
</script>

<!-- Instruction Editor Modal -->
{#if showEditorModal}
  <InstructionEditorModal
    {workingDir}
    existingFile={editingFile}
    onClose={() => { showEditorModal = false; editingFile = null; }}
    onSaved={handleModalSaved}
  />
{/if}

<label>
  <div class="label-row">
    <span class="label-text" style="margin-bottom: 0;">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
        <polyline points="14 2 14 8 20 8"/>
      </svg>
      Instructions <span style="color: var(--text-muted); font-weight: 400;">(Opt)</span>
      <HelpTip
        text="Optional. Select one or more instruction files (prompts/specs) from your working directory. They'll be passed into the agent as extra guidance."
      />
    </span>
    <div class="header-actions">
      <button class="icon-btn small" onclick={openInstructionsFolder} title="Open instructions folder">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
        </svg>
      </button>
      <button class="icon-btn small" onclick={openCreateModal} title="Create new with AI assistance">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="12" y1="5" x2="12" y2="19"/>
          <line x1="5" y1="12" x2="19" y2="12"/>
        </svg>
        New
      </button>
    </div>
  </div>

  <div class="instructions-section">
    <button
      type="button"
      class="instructions-toggle"
      onclick={() => showInstructionSelector = !showInstructionSelector}
      disabled={isCreating || isLoadingInstructions}
    >
      <span class="toggle-content">
        {#if selectedInstructions.size > 0}
          <span class="badge">{selectedInstructions.size}</span>
        {/if}
        {selectedInstructions.size > 0
          ? `${selectedInstructions.size} selected`
          : 'Select instructions'}
      </span>
      <svg class="chevron" class:rotated={showInstructionSelector} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="6 9 12 15 18 9"/>
      </svg>
    </button>

    {#if showInstructionSelector}
      <div class="instructions-dropdown animate-slide-up">
        {#if isLoadingInstructions}
          <div class="loading-state">
            <span class="spinner"></span>
            Loading...
          </div>
        {:else if instructionFiles.length === 0}
          <div class="empty-state">
            <p>No files found</p>
          </div>
        {:else}
          <div class="instructions-list">
            {#each instructionFiles as file (file.id)}
              <InstructionItem
                {file}
                isSelected={selectedInstructions.has(file.id)}
                {isCreating}
                isGenerating={generatingSkillForFile === file.id}
                onToggle={() => toggleInstructionSelection(file.id)}
                onEdit={() => openEditModal(file)}
                onGenerateSkill={() => generateSkillForInstruction(file)}
                onViewSkill={() => viewSkillContent(file.skillName!)}
                onDeleteSkill={() => deleteSkill(file)}
              />
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  </div>
</label>

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

<style>
  label {
    display: block;
  }

  .label-text {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--text-primary);
  }

  .label-text svg {
    width: 16px;
    height: 16px;
    color: var(--accent-hex);
    flex-shrink: 0;
  }

  .label-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-2);
    gap: var(--space-2);
  }

  .header-actions {
    display: flex;
    gap: var(--space-1);
    flex-shrink: 0;
  }

  .icon-btn.small {
    padding: var(--space-1) var(--space-2);
    font-size: var(--text-xs);
    border-radius: var(--radius-sm);
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    gap: var(--space-1);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .icon-btn.small:hover {
    background: var(--bg-elevated);
    color: var(--accent-hex);
  }

  .icon-btn.small svg {
    width: 12px;
    height: 12px;
  }

  .instructions-section {
    position: relative;
  }

  .instructions-toggle {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-2) var(--space-3);
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .instructions-toggle:hover:not(:disabled) {
    background: var(--bg-elevated);
    border-color: rgba(255, 255, 255, 0.15);
    color: var(--text-primary);
  }

  .toggle-content {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-width: 0;
  }

  .badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 18px;
    height: 18px;
    padding: 0 5px;
    background: var(--accent-hex);
    color: white;
    font-size: 11px;
    font-weight: var(--font-semibold);
    border-radius: var(--radius-full);
    flex-shrink: 0;
  }

  .instructions-toggle .chevron {
    width: 14px;
    height: 14px;
    transition: transform 0.2s ease;
    flex-shrink: 0;
  }

  .instructions-toggle .chevron.rotated {
    transform: rotate(180deg);
  }

  .instructions-dropdown {
    margin-top: var(--space-2);
    padding: var(--space-1);
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    max-height: 180px;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .loading-state,
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-3);
    gap: var(--space-2);
    text-align: center;
    color: var(--text-muted);
    font-size: var(--text-xs);
  }

  .instructions-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .spinner {
    width: 16px;
    height: 16px;
    border: 2px solid rgba(255, 255, 255, 0.2);
    border-top-color: var(--text-secondary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
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
