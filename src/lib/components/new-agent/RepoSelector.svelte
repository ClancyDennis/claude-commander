<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import HelpTip from "./HelpTip.svelte";

  let { githubUrl = $bindable(), isCreating }: { githubUrl: string, isCreating: boolean } = $props();

  let githubRepos = $state<Array<{
    nameWithOwner: string;
    name: string;
    description: string | null;
    url: string;
    updatedAt: string;
  }>>([]);
  let showRepoDropdown = $state(false);
  let isLoadingRepos = $state(false);

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

  function selectRepo(repo: typeof githubRepos[0]) {
    githubUrl = repo.url;
    showRepoDropdown = false;
  }
</script>

<label>
  <span class="label-text">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"/>
    </svg>
    GitHub Repo <span style="color: var(--text-muted); font-weight: 400;">(Opt)</span>
    <HelpTip
      text="Optional. If set, the backend can use it as context for cloning or linking your project. Leave blank if you’re working locally."
    />
  </span>
  <div class="repo-selector">
    <div class="input-group">
      <input
        type="text"
        bind:value={githubUrl}
        placeholder="https://github.com/..."
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
      <!-- svelte-ignore a11y_no_static_element_interactions -->
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
          </button>
        {/each}
      </div>
    {/if}
  </div>
</label>

<style>
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

  .dropdown-btn {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: 0 var(--space-sm);
    min-width: 40px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 10px;
    color: var(--text-secondary);
    transition: all 0.2s ease;
  }

  .dropdown-btn:hover:not(:disabled) {
    background: var(--bg-tertiary);
    border-color: var(--accent);
    color: var(--text-primary);
  }

  .dropdown-btn svg {
    width: 16px;
    height: 16px;
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
</style>
