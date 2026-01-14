<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { addAgent, selectedAgentId } from "../stores/agents";
  import { addPipeline, openPipeline } from "../stores/pipelines";
  import { getAgentSettings } from "../stores/pipelineSettings";
  import type { Agent } from "../types";
  import type { Pipeline } from "../stores/pipelines";

  let { onClose }: { onClose: () => void } = $props();

  // Default to home directory
  let workingDir = $state("/home/arrakis");
  let githubUrl = $state("");
  let creationType = $state<'agent' | 'pipeline'>('agent');
  let isCreating = $state(false);
  let error = $state("");
  let isLoadingRepos = $state(false);
  let githubRepos = $state<Array<{
    nameWithOwner: string;
    name: string;
    description: string | null;
    url: string;
    updatedAt: string;
  }>>([]);
  let showRepoDropdown = $state(false);

  // Load GitHub repos on mount
  $effect(() => {
    loadGithubRepos();
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

    isCreating = true;
    error = "";

    try {
      if (creationType === 'pipeline') {
        // Create pipeline with user-configured settings
        // Settings are configured via AgentSettings.svelte UI (gear icon in agent list)
        const userRequest = `Development project in ${workingDir}${githubUrl.trim() ? ` (GitHub: ${githubUrl.trim()})` : ''}`;

        // Get pipeline settings from store
        // Currently uses 'default' settings - can be made per-pipeline later by:
        // 1. Creating settings UI in this dialog
        // 2. Using pipeline ID instead of 'default'
        // 3. Allowing settings override after creation
        const settings = getAgentSettings('default');

        // Build backend config from frontend settings
        // Note: Only Phase D (Checkpoints) settings are passed to backend
        // Phase A-C (Pool, Orchestration, Verification) are frontend-only for now
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

        const pipelineId = await invoke<string>("create_pipeline", {
          userRequest,
          config: config
        });

        // Add pipeline to store
        const pipeline: Pipeline = {
          id: pipelineId,
          workingDir: workingDir,
          userRequest: userRequest,
          status: 'planning',
          createdAt: new Date(),
        };

        addPipeline(pipeline);
        openPipeline(pipelineId);
        onClose();
      } else {
        // Create single agent (existing logic)
        const agentId = await invoke<string>("create_agent", {
          workingDir: workingDir,
          githubUrl: githubUrl.trim() || null,
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
      } else {
        onClose();
      }
    } else if (e.key === "Enter" && workingDir.trim() && !showRepoDropdown) {
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
        disabled={isCreating || !workingDir.trim()}
      >
        {#if isCreating}
          <span class="spinner"></span>
          Creating...
        {:else}
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="12" y1="5" x2="12" y2="19"/>
            <line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
          {creationType === 'pipeline' ? 'Create Pipeline' : 'Create Agent'}
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
</style>
