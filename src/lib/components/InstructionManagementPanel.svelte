<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { homeDir } from "@tauri-apps/api/path";
  import { onMount } from "svelte";
  import type { InstructionFileInfo, GeneratedSkill } from "../types";
  import { useAsyncData } from "$lib/hooks/useAsyncData.svelte";
  import InstructionRow from "./instruction-management/InstructionRow.svelte";
  import SkillDetailModal from "./instruction-management/SkillDetailModal.svelte";
  import InstructionViewModal from "./instruction-management/InstructionViewModal.svelte";
  import InstructionEditorModal from "./InstructionEditorModal.svelte";
  import InstructionWizard from "./InstructionWizard.svelte";
  import { RefreshCw, X, FolderOpen, Sparkles, Plus, FileText, AlertCircle } from "lucide-svelte";

  let { onClose }: { onClose?: () => void } = $props();

  // Data state
  const instructionFilesData = useAsyncData(() =>
    invoke<InstructionFileInfo[]>("list_instruction_files", { workingDir: "" })
  );
  let generatedSkills = $state<Map<string, GeneratedSkill>>(new Map());

  // Working directory for skill operations
  let workingDir = $state("");

  // Skill generation state
  let generatingSkillForFile = $state<string | null>(null);

  // Modal states
  let showEditorModal = $state(false);
  let showWizardModal = $state(false);
  let showSkillModal = $state(false);
  let showViewModal = $state(false);
  let editingFile = $state<InstructionFileInfo | null>(null);
  let viewingFile = $state<InstructionFileInfo | null>(null);
  let viewingSkillName = $state<string | null>(null);

  onMount(async () => {
    try {
      workingDir = await homeDir();
    } catch (e) {
      console.error("Failed to get home directory:", e);
    }
    await loadInstructionFiles();
  });

  // Re-check skills when working dir changes
  $effect(() => {
    const files = instructionFilesData.data;
    if (workingDir && workingDir.trim() && files && files.length > 0) {
      checkExistingSkills();
    }
  });

  async function loadInstructionFiles() {
    const files = await instructionFilesData.fetch();
    if (files && workingDir) {
      await checkExistingSkills();
    }
  }

  async function checkExistingSkills() {
    if (!workingDir || !instructionFilesData.data) return;
    try {
      const skills = await invoke<GeneratedSkill[]>("list_generated_skills", {
        workingDir,
      });

      const skillMap = new Map<string, GeneratedSkill>();
      for (const skill of skills) {
        for (const file of instructionFilesData.data) {
          const expectedSkillName = file.name
            .replace(/\.(txt|md)$/, "")
            .toLowerCase()
            .replace(/[^a-z0-9]+/g, "-");

          if (skill.skillName === expectedSkillName) {
            file.hasSkill = true;
            file.skillName = skill.skillName;
            skillMap.set(file.id, skill);
          }
        }
      }

      generatedSkills = skillMap;
      // Trigger reactivity by updating the data
      instructionFilesData.setData([...instructionFilesData.data]);
    } catch (e) {
      console.error("Failed to check existing skills:", e);
    }
  }

  async function openInstructionsFolder() {
    try {
      await invoke("open_instructions_directory");
    } catch (e) {
      console.error("Failed to open instructions folder:", e);
    }
  }

  function openCreateModal() {
    editingFile = null;
    showEditorModal = true;
  }

  function openViewModal(file: InstructionFileInfo) {
    viewingFile = file;
    showViewModal = true;
  }

  function openEditModal(file: InstructionFileInfo) {
    editingFile = file;
    showEditorModal = true;
  }

  function handleModalSaved(filename: string) {
    showEditorModal = false;
    editingFile = null;
    loadInstructionFiles();
  }

  function handleWizardSaved(filename: string) {
    showWizardModal = false;
    loadInstructionFiles();
  }

  async function deleteInstruction(file: InstructionFileInfo) {
    if (!confirm(`Delete "${file.name}"? This cannot be undone.`)) {
      return;
    }

    try {
      await invoke("delete_instruction_file", {
        filePath: file.path,
      });
      await loadInstructionFiles();
    } catch (e) {
      console.error("Failed to delete instruction:", e);
      alert(`Failed to delete: ${e}`);
    }
  }

  async function generateSkillForInstruction(file: InstructionFileInfo) {
    if (!workingDir) {
      alert("Please set a working directory for skill generation.");
      return;
    }

    generatingSkillForFile = file.id;

    try {
      const result = await invoke<GeneratedSkill>("generate_skill_from_instruction", {
        filePath: file.path,
        workingDir: workingDir,
      });

      generatedSkills.set(file.id, result);
      file.hasSkill = true;
      file.skillName = result.skillName;
      // Trigger reactivity
      if (instructionFilesData.data) {
        instructionFilesData.setData([...instructionFilesData.data]);
      }
    } catch (e) {
      console.error("Skill generation error:", e);
      alert(`Failed to generate skill: ${e}`);
    } finally {
      generatingSkillForFile = null;
    }
  }

  function viewSkillContent(skillName: string) {
    viewingSkillName = skillName;
    showSkillModal = true;
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
      // Trigger reactivity
      if (instructionFilesData.data) {
        instructionFilesData.setData([...instructionFilesData.data]);
      }
    } catch (e) {
      console.error("Failed to delete skill:", e);
      alert(`Failed to delete skill: ${e}`);
    }
  }

  function handleSkillDelete() {
    if (viewingSkillName && instructionFilesData.data) {
      const file = instructionFilesData.data.find((f) => f.skillName === viewingSkillName);
      if (file) {
        showSkillModal = false;
        viewingSkillName = null;
        deleteSkill(file);
      }
    }
  }
</script>

<!-- Editor Modal -->
{#if showEditorModal}
  <InstructionEditorModal
    {workingDir}
    existingFile={editingFile}
    onClose={() => {
      showEditorModal = false;
      editingFile = null;
    }}
    onSaved={handleModalSaved}
  />
{/if}

<!-- Wizard Modal -->
{#if showWizardModal}
  <InstructionWizard
    {workingDir}
    onClose={() => (showWizardModal = false)}
    onSaved={handleWizardSaved}
  />
{/if}

<!-- Instruction View Modal -->
{#if showViewModal && viewingFile}
  <InstructionViewModal
    file={viewingFile}
    onClose={() => {
      showViewModal = false;
      viewingFile = null;
    }}
    onEdit={() => {
      showViewModal = false;
      openEditModal(viewingFile!);
      viewingFile = null;
    }}
  />
{/if}

<!-- Skill Detail Modal -->
{#if showSkillModal && viewingSkillName}
  <SkillDetailModal
    skillName={viewingSkillName}
    {workingDir}
    onClose={() => {
      showSkillModal = false;
      viewingSkillName = null;
    }}
    onDelete={handleSkillDelete}
  />
{/if}

<div class="instruction-panel">
  <header class="panel-header">
    <div class="header-title">
      <FileText size={22} />
      <h2>Instructions</h2>
    </div>
    <div class="header-actions">
      <button
        class="icon-btn"
        onclick={loadInstructionFiles}
        disabled={instructionFilesData.loading}
        title="Refresh"
      >
        <RefreshCw size={16} class={instructionFilesData.loading ? "spinning" : ""} />
      </button>
      {#if onClose}
        <button class="icon-btn" onclick={onClose} title="Close">
          <X size={16} />
        </button>
      {/if}
    </div>
  </header>

  <div class="panel-content">
    <!-- Action Bar -->
    <section class="action-section">
      <div class="action-bar">
        <button class="action-btn" onclick={openInstructionsFolder}>
          <FolderOpen size={16} />
          <span>Open Folder</span>
        </button>
        <button class="action-btn primary" onclick={() => (showWizardModal = true)}>
          <Sparkles size={16} />
          <span>Wizard</span>
        </button>
        <button class="action-btn" onclick={openCreateModal}>
          <Plus size={16} />
          <span>New</span>
        </button>
      </div>
    </section>

    <!-- Instructions List -->
    <section class="list-section">
      <h3>Your Instructions</h3>

      {#if instructionFilesData.loading}
        <div class="loading-state">
          <span class="spinner"></span>
          <p>Loading instructions...</p>
        </div>
      {:else if instructionFilesData.error}
        <div class="error-state">
          <AlertCircle size={20} />
          <p>Failed to load instructions: {instructionFilesData.error}</p>
          <button class="retry-btn" onclick={loadInstructionFiles}>Retry</button>
        </div>
      {:else if !instructionFilesData.data || instructionFilesData.data.length === 0}
        <div class="empty-state">
          <div class="empty-icon">
            <FileText size={32} />
          </div>
          <h4>No instructions yet</h4>
          <p>Create your first instruction file to get started.</p>
          <div class="empty-actions">
            <button class="action-btn primary" onclick={() => (showWizardModal = true)}>
              <Sparkles size={16} />
              <span>Create with Wizard</span>
            </button>
            <button class="action-btn" onclick={openCreateModal}>
              <Plus size={16} />
              <span>Create Manually</span>
            </button>
          </div>
        </div>
      {:else}
        <div class="instructions-list">
          {#each instructionFilesData.data as file (file.id)}
            <InstructionRow
              {file}
              isGeneratingSkill={generatingSkillForFile === file.id}
              onView={() => openViewModal(file)}
              onEdit={() => openEditModal(file)}
              onDelete={() => deleteInstruction(file)}
              onGenerateSkill={() => generateSkillForInstruction(file)}
              onViewSkill={() => viewSkillContent(file.skillName!)}
              onDeleteSkill={() => deleteSkill(file)}
            />
          {/each}
        </div>
      {/if}
    </section>

    <!-- Info Section -->
    <section class="info-section">
      <p class="info-text">
        Instructions are stored in <code>~/.instructions/</code>
      </p>
    </section>
  </div>
</div>

<style>
  .instruction-panel {
    height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--bg-primary);
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-5) var(--space-6);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .header-title {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    color: var(--text-primary);
  }

  .header-title h2 {
    margin: 0;
    font-size: var(--text-xl);
    font-weight: var(--font-semibold);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .icon-btn {
    width: 36px;
    height: 36px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .icon-btn:hover:not(:disabled) {
    background: var(--bg-elevated);
    border-color: rgba(255, 255, 255, 0.15);
    color: var(--text-primary);
  }

  .icon-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  :global(.spinning) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .panel-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-6);
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }

  .action-section {
    flex-shrink: 0;
  }

  .action-bar {
    display: flex;
    gap: var(--space-2);
  }

  .action-btn {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-4);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    border-radius: var(--radius-md);
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .action-btn:hover {
    background: var(--bg-elevated);
    border-color: rgba(255, 255, 255, 0.15);
    color: var(--text-primary);
  }

  .action-btn.primary {
    background: linear-gradient(135deg, var(--accent-hex), var(--accent-hover));
    border-color: var(--accent-hex);
    color: white;
  }

  .action-btn.primary:hover {
    background: linear-gradient(135deg, var(--accent-hover), var(--accent-hex));
  }

  .list-section {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  .list-section h3 {
    margin: 0 0 var(--space-4);
    font-size: var(--text-sm);
    font-weight: var(--font-semibold);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .instructions-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .loading-state,
  .error-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-8);
    gap: var(--space-4);
    color: var(--text-muted);
  }

  .error-state {
    color: var(--error);
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 2px solid rgba(255, 255, 255, 0.2);
    border-top-color: var(--accent-hex);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  .retry-btn {
    padding: var(--space-2) var(--space-4);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text-primary);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .retry-btn:hover {
    background: var(--bg-elevated);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-8) var(--space-6);
    text-align: center;
    background: var(--bg-secondary);
    border: 1px dashed var(--border);
    border-radius: var(--radius-lg);
  }

  .empty-icon {
    width: 64px;
    height: 64px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-tertiary);
    border-radius: var(--radius-lg);
    color: var(--text-muted);
    margin-bottom: var(--space-4);
  }

  .empty-state h4 {
    margin: 0 0 var(--space-2);
    font-size: var(--text-lg);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
  }

  .empty-state p {
    margin: 0 0 var(--space-5);
    font-size: var(--text-sm);
    color: var(--text-muted);
  }

  .empty-actions {
    display: flex;
    gap: var(--space-3);
  }

  .info-section {
    flex-shrink: 0;
    padding-top: var(--space-4);
    border-top: 1px solid var(--border);
  }

  .info-text {
    margin: 0;
    font-size: var(--text-xs);
    color: var(--text-muted);
  }

  .info-text code {
    background: var(--bg-tertiary);
    padding: 2px 6px;
    border-radius: 4px;
  }
</style>
