<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import type { SkillContent } from "../../types";
  import { X, Copy, Check, Trash2, FileCode, BookOpen, Code2 } from "lucide-svelte";
  import { marked } from "marked";
  import DOMPurify from "dompurify";
  import { useAsyncData } from "$lib/hooks/useAsyncData.svelte";

  let {
    skillName,
    workingDir,
    onClose,
    onDelete,
  }: {
    skillName: string;
    workingDir: string;
    onClose: () => void;
    onDelete?: () => void;
  } = $props();

  const skillContent = useAsyncData(() =>
    invoke<SkillContent>("get_skill_content", {
      skillName,
      workingDir,
    })
  );

  let copied = $state(false);
  let activeTab = $state<'skill' | 'reference' | 'examples' | 'scripts'>('skill');

  onMount(() => {
    skillContent.fetch();
  });

  async function copyToClipboard() {
    if (!skillContent.data) return;
    try {
      await navigator.clipboard.writeText(skillContent.data.skillMd);
      copied = true;
      setTimeout(() => copied = false, 2000);
    } catch (e) {
      console.error("Failed to copy:", e);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onClose();
    }
  }

  function renderMarkdown(md: string): string {
    return DOMPurify.sanitize(marked.parse(md) as string);
  }

  $effect(() => {
    if (skillContent.data) {
      // Auto-select first available tab
      if (skillContent.data.skillMd) activeTab = 'skill';
      else if (skillContent.data.referenceMd) activeTab = 'reference';
      else if (skillContent.data.examplesMd) activeTab = 'examples';
      else if (skillContent.data.scripts.length > 0) activeTab = 'scripts';
    }
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="modal-overlay" onclick={onClose}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="modal" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true" tabindex="-1">
    <header class="modal-header">
      <div class="header-title">
        <FileCode size={20} />
        <h2>{skillName}</h2>
      </div>
      <div class="header-actions">
        <button class="icon-btn" onclick={copyToClipboard} title="Copy skill content">
          {#if copied}
            <Check size={16} />
          {:else}
            <Copy size={16} />
          {/if}
        </button>
        {#if onDelete}
          <button class="icon-btn danger" onclick={onDelete} title="Delete skill">
            <Trash2 size={16} />
          </button>
        {/if}
        <button class="icon-btn" onclick={onClose} title="Close">
          <X size={16} />
        </button>
      </div>
    </header>

    {#if skillContent.loading}
      <div class="modal-loading">
        <span class="spinner"></span>
        <p>Loading skill content...</p>
      </div>
    {:else if skillContent.error}
      <div class="modal-error">
        <p>Failed to load skill: {skillContent.error}</p>
        <button class="retry-btn" onclick={onClose}>Close</button>
      </div>
    {:else if skillContent.data}
      <div class="modal-tabs">
        <button
          class="tab"
          class:active={activeTab === 'skill'}
          onclick={() => activeTab = 'skill'}
        >
          <FileCode size={14} />
          Skill
        </button>
        {#if skillContent.data.referenceMd}
          <button
            class="tab"
            class:active={activeTab === 'reference'}
            onclick={() => activeTab = 'reference'}
          >
            <BookOpen size={14} />
            Reference
          </button>
        {/if}
        {#if skillContent.data.examplesMd}
          <button
            class="tab"
            class:active={activeTab === 'examples'}
            onclick={() => activeTab = 'examples'}
          >
            <Code2 size={14} />
            Examples
          </button>
        {/if}
        {#if skillContent.data.scripts.length > 0}
          <button
            class="tab"
            class:active={activeTab === 'scripts'}
            onclick={() => activeTab = 'scripts'}
          >
            <FileCode size={14} />
            Scripts ({skillContent.data.scripts.length})
          </button>
        {/if}
      </div>

      <div class="modal-content">
        {#if activeTab === 'skill'}
          <div class="markdown-content">
            {@html renderMarkdown(skillContent.data.skillMd)}
          </div>
        {:else if activeTab === 'reference' && skillContent.data.referenceMd}
          <div class="markdown-content">
            {@html renderMarkdown(skillContent.data.referenceMd)}
          </div>
        {:else if activeTab === 'examples' && skillContent.data.examplesMd}
          <div class="markdown-content">
            {@html renderMarkdown(skillContent.data.examplesMd)}
          </div>
        {:else if activeTab === 'scripts'}
          <div class="scripts-list">
            {#each skillContent.data.scripts as script}
              <div class="script-item">
                <div class="script-header">
                  <span class="script-name">{script.name}</span>
                  <span class="script-lang">{script.language}</span>
                </div>
                <pre class="script-content"><code>{script.content}</code></pre>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: var(--space-6);
    animation: fadeIn 0.15s ease;
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .modal {
    width: 100%;
    max-width: 700px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.4);
    animation: slideUp 0.2s ease;
  }

  @keyframes slideUp {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-4) var(--space-5);
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
    font-size: var(--text-lg);
    font-weight: var(--font-semibold);
    margin: 0;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: var(--radius-md);
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .icon-btn:hover {
    background: var(--bg-elevated);
    color: var(--text-primary);
  }

  .icon-btn.danger:hover {
    background: rgba(239, 68, 68, 0.1);
    border-color: rgba(239, 68, 68, 0.3);
    color: #ef4444;
  }

  .modal-tabs {
    display: flex;
    gap: 2px;
    padding: var(--space-2) var(--space-5);
    border-bottom: 1px solid var(--border);
    background: var(--bg-secondary);
    flex-shrink: 0;
  }

  .tab {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: var(--space-2) var(--space-3);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--text-muted);
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .tab:hover {
    color: var(--text-secondary);
    background: var(--bg-tertiary);
  }

  .tab.active {
    color: var(--text-primary);
    background: var(--bg-primary);
  }

  .modal-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-5);
  }

  .modal-loading,
  .modal-error {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-8);
    gap: var(--space-4);
    color: var(--text-muted);
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 2px solid rgba(255, 255, 255, 0.2);
    border-top-color: var(--accent-hex);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
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
  }

  .markdown-content {
    font-size: var(--text-sm);
    line-height: 1.6;
    color: var(--text-primary);
  }

  .markdown-content :global(h1),
  .markdown-content :global(h2),
  .markdown-content :global(h3) {
    margin-top: var(--space-4);
    margin-bottom: var(--space-2);
    font-weight: var(--font-semibold);
  }

  .markdown-content :global(p) {
    margin-bottom: var(--space-3);
  }

  .markdown-content :global(code) {
    background: var(--bg-tertiary);
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 0.9em;
  }

  .markdown-content :global(pre) {
    background: var(--bg-tertiary);
    padding: var(--space-4);
    border-radius: var(--radius-md);
    overflow-x: auto;
    margin: var(--space-3) 0;
  }

  .markdown-content :global(pre code) {
    background: transparent;
    padding: 0;
  }

  .markdown-content :global(ul),
  .markdown-content :global(ol) {
    margin-bottom: var(--space-3);
    padding-left: var(--space-5);
  }

  .scripts-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .script-item {
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .script-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-2) var(--space-3);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
  }

  .script-name {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--text-primary);
  }

  .script-lang {
    font-size: var(--text-xs);
    padding: 2px 6px;
    background: var(--bg-tertiary);
    border-radius: 4px;
    color: var(--text-muted);
  }

  .script-content {
    margin: 0;
    padding: var(--space-4);
    background: var(--bg-tertiary);
    font-size: var(--text-sm);
    line-height: 1.5;
    overflow-x: auto;
  }

  .script-content code {
    color: var(--text-primary);
  }
</style>
