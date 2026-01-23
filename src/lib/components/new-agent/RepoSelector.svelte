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
            <span class="repo-name">{repo.nameWithOwner}</span>
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
    gap: var(--space-2);
    margin-bottom: var(--space-2);
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

  .input-group {
    display: flex;
    gap: var(--space-2);
  }

  .input-group input {
    flex: 1;
    min-width: 0;
    padding: var(--space-2) var(--space-3);
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text-primary);
    font-size: var(--text-sm);
    transition: all var(--transition-fast);
  }

  .input-group input:focus {
    outline: none;
    border-color: var(--accent-hex);
    box-shadow: 0 0 0 3px rgba(232, 102, 77, 0.15);
  }

  .dropdown-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    padding: 0;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    flex-shrink: 0;
    transition: all var(--transition-fast);
  }

  .dropdown-btn:hover:not(:disabled) {
    background: var(--bg-elevated);
    border-color: var(--accent-hex);
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
    top: calc(100% + var(--space-1));
    left: 0;
    right: 0;
    max-height: 240px;
    overflow-y: auto;
    overflow-x: hidden;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    z-index: 10;
    padding: var(--space-1);
  }

  .repo-item {
    width: 100%;
    padding: var(--space-2) var(--space-3);
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    text-align: left;
    transition: all var(--transition-fast);
    min-width: 0;
  }

  .repo-item:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .repo-name {
    font-weight: var(--font-medium);
    color: var(--text-primary);
    font-size: var(--text-sm);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    display: block;
  }
</style>
