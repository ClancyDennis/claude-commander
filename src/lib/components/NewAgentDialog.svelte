<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { addAgent, selectedAgentId } from "../stores/agents";
  import { addPipeline, openPipeline } from "../stores/pipelines";
  import { getAgentSettings } from "../stores/pipelineSettings";
  import type { Agent, GeneratedSkill, SkillContent, InstructionFileInfo } from "../types";
  import type { Pipeline } from "../stores/pipelines";

  let { onClose }: { onClose: () => void } = $props();

  // Default to home directory
  let workingDir = $state("/home/arrakis");
  let githubUrl = $state("");
  let pipelineTask = $state("");
  let creationType = $state<'agent' | 'pipeline' | 'auto-pipeline'>('agent');
  let isCreating = $state(false);
  let error = $state("");
  let isLoadingRepos = $state(false);
  let showPipelineSettings = $state(false);
  let githubRepos = $state<Array<{
    nameWithOwner: string;
    name: string;
    description: string | null;
    url: string;
    updatedAt: string;
  }>>([]);
  let showRepoDropdown = $state(false);
  let showInstructionSelector = $state(false);
  let instructionFiles = $state<Array<InstructionFileInfo>>([]);
  let selectedInstructions = $state<Set<string>>(new Set());
  let isLoadingInstructions = $state(false);

  // Instruction Creator State
  let showInstructionCreator = $state(false);
  let newInstructionName = $state("");
  let newInstructionContent = $state("");
  let isSavingInstruction = $state(false);

  // Skill Generation State
  let generatingSkillForFile = $state<string | null>(null);
  let generatedSkills = $state<Map<string, GeneratedSkill>>(new Map());
  let skillGenerationError = $state<string | null>(null);

  // Load GitHub repos on mount
  $effect(() => {
    loadGithubRepos();
  });

  // Load instruction files when working directory changes
  $effect(() => {
    if (workingDir.trim()) {
      loadInstructionFiles();
    }
  });

  async function loadGithubRepos() {
    try {
      isLoadingRepos = true;
      const repos = await invoke<Array<any>>("list_github_repos");
      githubRepos = repos;
    } catch (e) {
      console.error("Failed to load GitHub repos:", e);
      // Silent fail - user can still enter URL manually
    } finally {
      isLoadingRepos = false;
    }
  }

  async function loadInstructionFiles() {
    try {
      isLoadingInstructions = true;
      const files = await invoke<Array<any>>("list_instruction_files", {
        workingDir: workingDir,
      });
      instructionFiles = files;

      // Check for existing skills after loading files
      await checkExistingSkills();
    } catch (e) {
      console.error("Failed to load instruction files:", e);
      // Silent fail - instructions are optional
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

      // Match skills to instruction files by name heuristics
      for (const skill of skills) {
        for (const file of instructionFiles) {
          // Simple matching: skill name derived from file name
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

      instructionFiles = [...instructionFiles]; // Trigger reactivity
    } catch (e) {
      console.error("Failed to check existing skills:", e);
    }
  }

  async function generateSkillForInstruction(file: InstructionFileInfo) {
    generatingSkillForFile = file.id;
    skillGenerationError = null;

    try {
      const result = await invoke<GeneratedSkill>("generate_skill_from_instruction", {
        filePath: file.path,
        workingDir: workingDir,
      });

      generatedSkills.set(file.id, result);
      file.hasSkill = true;
      file.skillName = result.skillName;

      // Re-trigger reactivity
      instructionFiles = [...instructionFiles];
    } catch (error) {
      skillGenerationError = `Failed to generate skill: ${error}`;
      console.error("Skill generation error:", error);
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

      // For now, log to console - could create a modal later
      console.log("Skill content:", content);
      alert(`Skill: ${content.skillName}\n\n${content.skillMd.substring(0, 500)}...\n\nFull content logged to console.`);
    } catch (error) {
      console.error("Failed to load skill:", error);
      alert(`Failed to load skill: ${error}`);
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
    } catch (error) {
      console.error("Failed to delete skill:", error);
      alert(`Failed to delete skill: ${error}`);
    }
  }

  async function saveNewInstruction() {
    if (!newInstructionName.trim() || !newInstructionContent.trim()) return;
    
    let filename = newInstructionName.trim();
    if (!filename.endsWith('.md') && !filename.endsWith('.txt')) {
      filename += '.md';
    }
    
    isSavingInstruction = true;
    try {
      await invoke("save_instruction_file", {
        workingDir,
        filename,
        content: newInstructionContent
      });
      
      // Reset form
      newInstructionName = "";
      newInstructionContent = "";
      showInstructionCreator = false;
      
      // Reload instructions
      await loadInstructionFiles();
      
      // Auto-select the new file
      const newFile = instructionFiles.find(f => f.name === filename);
      if (newFile) {
        selectedInstructions.add(newFile.id);
        selectedInstructions = new Set(selectedInstructions);
      }
      
      // Ensure selector is open to show the new selection
      showInstructionSelector = true;
      
    } catch (e) {
      console.error("Failed to save instruction:", e);
      error = "Failed to save instruction: " + e;
    } finally {
      isSavingInstruction = false;
    }
  }

  function toggleInstructionSelection(fileId: string) {
    if (selectedInstructions.has(fileId)) {
      selectedInstructions.delete(fileId);
    } else {
      selectedInstructions.add(fileId);
    }
    selectedInstructions = new Set(selectedInstructions); // Trigger reactivity
  }

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

  function selectRepo(repo: typeof githubRepos[0]) {
    githubUrl = repo.url;
    showRepoDropdown = false;
  }

  async function createAgent() {
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
        // Get pipeline settings from store
        const settings = getAgentSettings('default');

        // Build backend config from frontend settings
        const config = {
          require_plan_review: settings.requirePlanReview,
          require_final_review: settings.requireFinalReview,
          auto_approve_on_verification: settings.autoApproveOnVerification,
          verification_strategy: settings.enableVerification
            ? settings.verificationStrategy
            : null,
          verification_n: settings.enableVerification
            ? settings.verificationN
            : 1,
        };

        // Create pipeline with task and config
        const pipelineId = await invoke<string>("create_pipeline", {
          userRequest: pipelineTask.trim(),
          config: config
        });

        // Start pipeline execution immediately
        await invoke("start_pipeline", {
          pipelineId,
          userRequest: pipelineTask.trim(),
        });

        // Add pipeline to store
        const pipeline: Pipeline = {
          id: pipelineId,
          workingDir: workingDir,
          userRequest: pipelineTask.trim(),
          status: 'planning',
          createdAt: new Date(),
        };

        addPipeline(pipeline);
        openPipeline(pipelineId);
        onClose();
      } else if (creationType === 'auto-pipeline') {
        // Create automated 3-step pipeline
        const pipelineId = await invoke<string>("create_auto_pipeline", {
          userRequest: pipelineTask.trim(),
          workingDir: workingDir,
        });

        // Start pipeline execution immediately
        await invoke("start_auto_pipeline", {
          pipelineId,
        });

        // Note: The AutoPipelineView component will handle displaying this
        // For now, we'll just close the dialog
        // TODO: Add navigation to auto-pipeline view or add to a store
        onClose();
      } else {
        // Create single agent (existing logic)
        const agentId = await invoke<string>("create_agent", {
          workingDir: workingDir,
          githubUrl: githubUrl.trim() || null,
          selectedInstructionFiles: selectedInstructions.size > 0
            ? Array.from(selectedInstructions)
            : null,
        });

        const agent: Agent = {
          id: agentId,
          workingDir: workingDir,
          status: "running",
          createdAt: new Date(),
        };

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
      if (showRepoDropdown) {
        showRepoDropdown = false;
      } else if (showInstructionCreator) {
        showInstructionCreator = false;
      } else {
        onClose();
      }
    } else if (e.key === "Enter" && workingDir.trim() && !showRepoDropdown && !showInstructionCreator) {
      createAgent();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="overlay animate-fade-in" onclick={onClose} role="presentation">
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
          <h2>Create New Agent</h2>
          <p class="subtitle">Start a new Claude Code instance</p>
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
      {#if showInstructionCreator}
        <div class="instruction-creator animate-slide-up">
          <h3>Create New Instruction</h3>
          <div class="creator-form">
            <input 
              type="text" 
              bind:value={newInstructionName} 
              placeholder="Filename (e.g., system-prompt.md)"
              class="filename-input"
            />
            <textarea 
              bind:value={newInstructionContent} 
              placeholder="Enter instruction content..."
              class="content-input"
            ></textarea>
            <div class="creator-actions">
              <button class="secondary small" onclick={() => showInstructionCreator = false} disabled={isSavingInstruction}>Cancel</button>
              <button class="primary small" onclick={saveNewInstruction} disabled={isSavingInstruction || !newInstructionName || !newInstructionContent}>
                {isSavingInstruction ? 'Saving...' : 'Save Instruction'}
              </button>
            </div>
          </div>
        </div>
      {:else}
        <label>
          <span class="label-text">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
            </svg>
            Working Directory
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
              Browse
            </button>
          </div>
          <span class="helper-text">The agent will run Claude Code in this directory</span>
        </label>

        <label>
          <span class="label-text">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="3"/>
              <path d="M12 1v6M12 17v6M5.64 5.64l4.24 4.24m6.36 6.36l4.24 4.24M1 12h6M17 12h6M5.64 18.36l4.24-4.24m6.36-6.36l4.24-4.24"/>
            </svg>
            Creation Type
          </span>
          <div class="radio-group">
            <label class="radio-option" class:selected={creationType === 'agent'}>
              <input type="radio" bind:group={creationType} value="agent" disabled={isCreating} />
              <div class="radio-content">
                <strong>Single Agent</strong>
                <span>Direct Claude Code instance for focused tasks</span>
              </div>
            </label>
            <label class="radio-option" class:selected={creationType === 'pipeline'}>
              <input type="radio" bind:group={creationType} value="pipeline" disabled={isCreating} />
              <div class="radio-content">
                <strong>Pipeline</strong>
                <span>Multi-phase workflow with orchestration & verification</span>
              </div>
            </label>
            <label class="radio-option" class:selected={creationType === 'auto-pipeline'}>
              <input type="radio" bind:group={creationType} value="auto-pipeline" disabled={isCreating} />
              <div class="radio-content">
                <strong>Auto Pipeline</strong>
                <span>3-step automated pipeline: Planning → Building → Verification</span>
              </div>
            </label>
          </div>
        </label>

        <label>
          <span class="label-text">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"/>
            </svg>
            GitHub Repository <span style="color: var(--text-muted); font-weight: 400;">(Optional)</span>
          </span>
          <div class="repo-selector">
            <div class="input-group">
              <input
                type="text"
                bind:value={githubUrl}
                placeholder="https://github.com/owner/repo"
                disabled={isCreating}
                onfocus={() => githubRepos.length > 0 && (showRepoDropdown = true)}
              />
              {#if githubRepos.length > 0}
                <button
                  type="button"
                  class="dropdown-btn"
                  onclick={() => showRepoDropdown = !showRepoDropdown}
                  disabled={isCreating || isLoadingRepos}
                  title="Select from your repositories"
                >
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="6 9 12 15 18 9"/>
                  </svg>
                </button>
              {/if}
            </div>
            {#if showRepoDropdown && githubRepos.length > 0}
              <div class="dropdown animate-slide-up">
                {#each githubRepos as repo}
                  <button
                    type="button"
                    class="repo-item"
                    onclick={() => selectRepo(repo)}
                  >
                    <div class="repo-header">
                      <span class="repo-name">{repo.nameWithOwner}</span>
                      <span class="repo-date">{new Date(repo.updatedAt).toLocaleDateString()}</span>
                    </div>
                    {#if repo.description}
                      <span class="repo-description">{repo.description}</span>
                    {/if}
                  </button>
                {/each}
              </div>
            {/if}
            {#if isLoadingRepos}
              <span class="helper-text">Loading your GitHub repositories...</span>
            {:else}
              <span class="helper-text">
                Link this agent to a GitHub repository for context
                {#if githubRepos.length > 0}
                  · {githubRepos.length} repos available
                {/if}
              </span>
            {/if}
          </div>
        </label>

        <label>
          <div class="label-row">
            <span class="label-text" style="margin-bottom: 0;">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                <polyline points="14 2 14 8 20 8"/>
              </svg>
              Instructions <span style="color: var(--text-muted); font-weight: 400;">(Optional)</span>
            </span>
            <button class="icon-btn small" onclick={() => showInstructionCreator = true} title="Create new instruction">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="12" y1="5" x2="12" y2="19"/>
                <line x1="5" y1="12" x2="19" y2="12"/>
              </svg>
              New
            </button>
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
                  ? `${selectedInstructions.size} instruction${selectedInstructions.size > 1 ? 's' : ''} selected`
                  : 'Select instruction files'}
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
                    Loading instructions...
                  </div>
                {:else if instructionFiles.length === 0}
                  <div class="empty-state">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                      <polyline points="14 2 14 8 20 8"/>
                    </svg>
                    <p>No instruction files found</p>
                    <span class="helper-text">
                      Create new instructions using the <strong>+ New</strong> button
                    </span>
                  </div>
                {:else}
                  <div class="instructions-list">
                    {#each instructionFiles as file}
                      <label class="instruction-item">
                        <input
                          type="checkbox"
                          checked={selectedInstructions.has(file.id)}
                          onchange={() => toggleInstructionSelection(file.id)}
                          disabled={isCreating}
                        />
                        <div class="instruction-info">
                          <span class="instruction-name">{file.name}</span>
                          <span class="instruction-meta">
                            {(file.size / 1024).toFixed(1)} KB
                            {#if file.relativePath !== file.name}
                              · {file.relativePath}
                            {/if}
                          </span>
                          {#if file.hasSkill}
                            <span class="skill-badge success">
                              ✓ Skill: {file.skillName}
                            </span>
                          {/if}
                        </div>
                        <div class="instruction-actions">
                          {#if generatingSkillForFile === file.id}
                            <button class="icon-btn tiny" disabled>
                              <div class="spinner"></div>
                              Generating...
                            </button>
                          {:else if file.hasSkill}
                            <button
                              class="icon-btn tiny"
                              onclick={(e) => { e.preventDefault(); viewSkillContent(file.skillName!) }}
                              title="View skill content"
                              type="button"
                            >
                              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
                                <circle cx="12" cy="12" r="3"/>
                              </svg>
                              View
                            </button>
                            <button
                              class="icon-btn tiny danger"
                              onclick={(e) => { e.preventDefault(); deleteSkill(file) }}
                              title="Delete skill"
                              type="button"
                            >
                              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <polyline points="3 6 5 6 21 6"/>
                                <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
                              </svg>
                              Delete
                            </button>
                          {:else}
                            <button
                              class="icon-btn tiny primary"
                              onclick={(e) => { e.preventDefault(); generateSkillForInstruction(file) }}
                              title="Generate Claude Code skill from this instruction"
                              type="button"
                            >
                              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
                              </svg>
                              Generate Skill
                            </button>
                          {/if}
                        </div>
                      </label>
                    {/each}
                  </div>
                  {#if skillGenerationError}
                    <div class="error-banner">
                      {skillGenerationError}
                    </div>
                  {/if}
                {/if}
              </div>
            {/if}
          </div>
        </label>

        {#if creationType === 'pipeline' || creationType === 'auto-pipeline'}
          <label>
            <span class="label-text">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
              </svg>
              Task Description
            </span>
            <textarea
              bind:value={pipelineTask}
              placeholder={creationType === 'auto-pipeline'
                ? "Example: Add dark mode support to the application..."
                : "Example: Build a REST API for user authentication with JWT tokens, including tests and documentation..."}
              rows="4"
              disabled={isCreating}
            ></textarea>
            <span class="helper-text">
              {creationType === 'auto-pipeline'
                ? "Describe your task. The auto-pipeline will plan, build, and verify automatically."
                : "Describe what you want the pipeline to accomplish. The meta-agent will break it down into phases."}
            </span>
          </label>

          <div class="settings-section">
            <button
              type="button"
              class="settings-toggle"
              onclick={() => showPipelineSettings = !showPipelineSettings}
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="3"/>
                <path d="M12 1v6m0 6v6M5.64 5.64l4.24 4.24m5.66 5.66l4.24 4.24M1 12h6m6 0h6M5.64 18.36l4.24-4.24m5.66-5.66l4.24-4.24"/>
              </svg>
              Pipeline Settings
              <svg class="chevron" class:rotated={showPipelineSettings} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="6 9 12 15 18 9"/>
              </svg>
            </button>
            {#if showPipelineSettings}
              <div class="settings-info animate-slide-up">
                <p>Configure pipeline behavior via <strong>Settings</strong> (gear icon in agent list) before creating the pipeline.</p>
                <ul>
                  <li>Plan review checkpoints</li>
                  <li>Final review requirements</li>
                  <li>Verification strategies</li>
                  <li>Auto-approval settings</li>
                </ul>
              </div>
            {/if}
          </div>
        {/if}
      {/if}

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
      <button class="secondary" onclick={onClose} disabled={isCreating}>
        Cancel
      </button>
      <button
        class="primary"
        onclick={createAgent}
        disabled={isCreating || !workingDir.trim() || ((creationType === 'pipeline' || creationType === 'auto-pipeline') && !pipelineTask.trim())}
      >
        {#if isCreating}
          <span class="spinner"></span>
          {creationType === 'pipeline' ? 'Starting Pipeline...' : creationType === 'auto-pipeline' ? 'Starting Auto Pipeline...' : 'Creating...'}
        {:else}
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            {#if creationType === 'pipeline'}
              <polygon points="5 3 19 12 5 21 5 3"/>
            {:else}
              <line x1="12" y1="5" x2="12" y2="19"/>
              <line x1="5" y1="12" x2="19" y2="12"/>
            {/if}
          </svg>
          {creationType === 'pipeline' ? 'Start Pipeline' : 'Create Agent'}
        {/if}
      </button>
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
    width: 540px;
    max-width: 100%;
    background-color: var(--bg-secondary);
    border-radius: 20px;
    border: 1px solid var(--border);
    box-shadow: var(--shadow-lg), 0 0 60px rgba(124, 58, 237, 0.1);
    overflow: hidden;
  }

  header {
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
  }

  label {
    display: block;
  }

  label + label {
    margin-top: var(--space-lg);
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
  }

  .browse-btn,
  .dropdown-btn {
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

  .browse-btn:hover:not(:disabled),
  .dropdown-btn:hover:not(:disabled) {
    background: var(--bg-tertiary);
    border-color: var(--accent);
    color: var(--text-primary);
  }

  .browse-btn svg,
  .dropdown-btn svg {
    width: 16px;
    height: 16px;
  }

  .dropdown-btn {
    padding: 0 var(--space-sm);
    min-width: 40px;
  }

  .repo-selector {
    position: relative;
  }

  .dropdown {
    position: absolute;
    top: calc(100% + var(--space-xs));
    left: 0;
    right: 0;
    max-height: 300px;
    overflow-y: auto;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 12px;
    box-shadow: var(--shadow-lg);
    z-index: 10;
    padding: var(--space-xs);
  }

  .repo-item {
    width: 100%;
    padding: var(--space-md);
    background: transparent;
    border: 1px solid transparent;
    border-radius: 8px;
    text-align: left;
    transition: all 0.2s ease;
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }

  .repo-item:hover {
    background: var(--bg-tertiary);
    border-color: var(--accent);
  }

  .repo-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-md);
  }

  .repo-name {
    font-weight: 600;
    color: var(--text-primary);
    font-size: 14px;
  }

  .repo-date {
    font-size: 12px;
    color: var(--text-muted);
    white-space: nowrap;
  }

  .repo-description {
    font-size: 13px;
    color: var(--text-secondary);
    line-height: 1.4;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .helper-text {
    display: block;
    margin-top: var(--space-sm);
    font-size: 13px;
    color: var(--text-muted);
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
    padding: var(--space-lg);
    display: flex;
    justify-content: flex-end;
    gap: var(--space-md);
    border-top: 1px solid var(--border);
    background-color: var(--bg-tertiary);
  }

  footer button {
    min-width: 120px;
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

  .radio-group {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .radio-option {
    display: flex;
    align-items: flex-start;
    gap: var(--space-md);
    padding: var(--space-md);
    background: var(--bg-elevated);
    border: 2px solid var(--border);
    border-radius: 12px;
    cursor: pointer;
    transition: all 0.2s ease;
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
    margin-top: 2px;
    width: 20px;
    height: 20px;
    cursor: pointer;
    accent-color: var(--accent);
  }

  .radio-option input[type="radio"]:disabled {
    cursor: not-allowed;
  }

  .radio-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
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

  .settings-section {
    margin-top: var(--space-md);
  }

  .settings-toggle {
    width: 100%;
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-md);
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 10px;
    color: var(--text-secondary);
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .settings-toggle:hover {
    background: var(--bg-tertiary);
    border-color: var(--accent);
    color: var(--text-primary);
  }

  .settings-toggle svg:first-child {
    width: 18px;
    height: 18px;
    color: var(--accent);
  }

  .settings-toggle .chevron {
    width: 16px;
    height: 16px;
    margin-left: auto;
    transition: transform 0.2s ease;
  }

  .settings-toggle .chevron.rotated {
    transform: rotate(180deg);
  }

  .settings-info {
    margin-top: var(--space-sm);
    padding: var(--space-md);
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 10px;
    font-size: 13px;
    color: var(--text-secondary);
  }

  .settings-info p {
    margin: 0 0 var(--space-sm) 0;
    line-height: 1.5;
  }

  .settings-info strong {
    color: var(--accent);
    font-weight: 600;
  }

  .settings-info ul {
    margin: 0;
    padding-left: var(--space-lg);
    line-height: 1.8;
  }

  .settings-info li {
    color: var(--text-muted);
  }

  .instructions-section {
    position: relative;
  }

  .instructions-toggle {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-md);
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 10px;
    color: var(--text-secondary);
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .instructions-toggle:hover:not(:disabled) {
    background: var(--bg-tertiary);
    border-color: var(--accent);
    color: var(--text-primary);
  }

  .instructions-toggle:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .toggle-content {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 20px;
    height: 20px;
    padding: 0 6px;
    background: var(--accent);
    color: white;
    font-size: 12px;
    font-weight: 600;
    border-radius: 10px;
  }

  .instructions-toggle .chevron {
    width: 16px;
    height: 16px;
    transition: transform 0.2s ease;
  }

  .instructions-toggle .chevron.rotated {
    transform: rotate(180deg);
  }

  .instructions-dropdown {
    margin-top: var(--space-sm);
    padding: var(--space-sm);
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 10px;
    max-height: 300px;
    overflow-y: auto;
  }

  .loading-state,
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-lg);
    gap: var(--space-md);
    text-align: center;
  }

  .loading-state {
    color: var(--text-secondary);
    font-size: 14px;
  }

  .empty-state svg {
    width: 48px;
    height: 48px;
    color: var(--text-muted);
    opacity: 0.5;
  }

  .empty-state p {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .empty-state code {
    background: var(--bg-tertiary);
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 12px;
    color: var(--accent);
    font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Fira Code', monospace;
  }

  .instructions-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }

  .instruction-item {
    display: flex;
    align-items: flex-start;
    gap: var(--space-md);
    padding: var(--space-md);
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .instruction-item:hover:not(:has(input:disabled)) {
    background: var(--bg-tertiary);
    border-color: var(--accent);
  }

  .instruction-item:has(input:checked) {
    background: rgba(124, 58, 237, 0.1);
    border-color: var(--accent);
  }

  .instruction-item input[type="checkbox"] {
    margin-top: 2px;
    width: 18px;
    height: 18px;
    cursor: pointer;
    accent-color: var(--accent);
    flex-shrink: 0;
  }

  .instruction-item input[type="checkbox"]:disabled {
    cursor: not-allowed;
  }

  .instruction-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .instruction-name {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .instruction-item:has(input:checked) .instruction-name {
    color: var(--accent);
  }

  .instruction-meta {
    font-size: 12px;
    color: var(--text-muted);
  }

  .skill-badge {
    display: inline-block;
    padding: 4px 8px;
    border-radius: 6px;
    font-size: 11px;
    font-weight: 600;
    margin-top: 4px;
  }

  .skill-badge.success {
    background: rgba(34, 197, 94, 0.15);
    color: #22c55e;
    border: 1px solid rgba(34, 197, 94, 0.3);
  }

  .instruction-actions {
    display: flex;
    gap: 6px;
    align-items: center;
    margin-left: auto;
  }

  .icon-btn.tiny {
    padding: 6px 10px;
    font-size: 12px;
    border-radius: 6px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    gap: 4px;
    white-space: nowrap;
    transition: all 0.2s ease;
  }

  .icon-btn.tiny:hover:not(:disabled) {
    background: var(--bg-tertiary);
    border-color: var(--accent);
    color: var(--text-primary);
  }

  .icon-btn.tiny.primary {
    background: rgba(124, 58, 237, 0.1);
    border-color: rgba(124, 58, 237, 0.3);
    color: var(--accent);
  }

  .icon-btn.tiny.primary:hover:not(:disabled) {
    background: rgba(124, 58, 237, 0.2);
    border-color: var(--accent);
  }

  .icon-btn.tiny.danger {
    color: #ef4444;
  }

  .icon-btn.tiny.danger:hover:not(:disabled) {
    background: rgba(239, 68, 68, 0.1);
    border-color: rgba(239, 68, 68, 0.3);
  }

  .icon-btn.tiny:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .icon-btn.tiny svg {
    width: 14px;
    height: 14px;
    flex-shrink: 0;
  }

  .spinner {
    width: 14px;
    height: 14px;
    border: 2px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .error-banner {
    padding: 12px;
    background: rgba(239, 68, 68, 0.1);
    color: #ef4444;
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: 8px;
    margin-top: 12px;
    font-size: 13px;
  }

  .label-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-sm);
  }

  .icon-btn.small {
    padding: 4px 10px;
    font-size: 12px;
    border-radius: 6px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .icon-btn.small:hover {
    background: var(--bg-tertiary);
    color: var(--accent);
    border-color: var(--accent);
  }

  .icon-btn.small svg {
    width: 14px;
    height: 14px;
  }

  .instruction-creator {
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: var(--space-md);
    margin-bottom: var(--space-lg);
  }

  .instruction-creator h3 {
    font-size: 15px;
    font-weight: 600;
    margin: 0 0 var(--space-md) 0;
    color: var(--text-primary);
  }

  .creator-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .filename-input,
  .content-input {
    width: 100%;
    padding: var(--space-sm) var(--space-md);
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 8px;
    color: var(--text-primary);
    font-size: 14px;
  }

  .content-input {
    min-height: 100px;
    resize: vertical;
  }

  .filename-input:focus,
  .content-input:focus {
    outline: none;
    border-color: var(--accent);
  }

  .creator-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-sm);
  }

  button.small {
    padding: 6px 12px;
    font-size: 13px;
  }
</style>
