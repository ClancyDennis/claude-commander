<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import type { InstructionFileInfo } from "../../types";
  import { X, Copy, Check, FileText, Edit2 } from "lucide-svelte";
  import { marked } from "marked";
  import DOMPurify from "dompurify";
  import { useAsyncData } from "$lib/hooks/useAsyncData.svelte";

  let {
    file,
    onClose,
    onEdit,
  }: {
    file: InstructionFileInfo;
    onClose: () => void;
    onEdit?: () => void;
  } = $props();

  const fileContent = useAsyncData(() =>
    invoke<string>("get_instruction_file_content", {
      filePath: file.path,
    }),
    { initialData: "" }
  );

  let copied = $state(false);

  onMount(() => {
    fileContent.fetch();
  });

  async function copyToClipboard() {
    try {
      await navigator.clipboard.writeText(fileContent.data ?? "");
      copied = true;
      setTimeout(() => (copied = false), 2000);
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

  function formatFileSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function formatDate(dateStr: string): string {
    const date = new Date(dateStr);
    return date.toLocaleDateString("en-US", {
      month: "short",
      day: "numeric",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }
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
        <FileText size={20} />
        <div class="title-info">
          <h2>{file.name}</h2>
          <div class="meta">
            <span class="type-badge">.{file.fileType}</span>
            <span class="meta-text">{formatFileSize(file.size)}</span>
            <span class="meta-separator">·</span>
            <span class="meta-text">{formatDate(file.modified)}</span>
          </div>
        </div>
      </div>
      <div class="header-actions">
        <button class="icon-btn" onclick={copyToClipboard} title="Copy content">
          {#if copied}
            <Check size={16} />
          {:else}
            <Copy size={16} />
          {/if}
        </button>
        {#if onEdit}
          <button class="icon-btn" onclick={() => { onClose(); onEdit(); }} title="Edit">
            <Edit2 size={16} />
          </button>
        {/if}
        <button class="icon-btn" onclick={onClose} title="Close">
          <X size={16} />
        </button>
      </div>
    </header>

    <div class="modal-content">
      {#if fileContent.loading}
        <div class="loading-state">
          <span class="spinner"></span>
          <p>Loading content...</p>
        </div>
      {:else if fileContent.error}
        <div class="error-state">
          <p>Failed to load: {fileContent.error}</p>
          <button class="retry-btn" onclick={onClose}>Close</button>
        </div>
      {:else}
        {#if file.fileType === "md"}
          <div class="markdown-content">
            {@html renderMarkdown(fileContent.data ?? "")}
          </div>
        {:else}
          <pre class="text-content"><code>{fileContent.data}</code></pre>
        {/if}
      {/if}
    </div>
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
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .modal {
    width: 100%;
    max-width: 750px;
    max-height: 85vh;
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
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-4) var(--space-5);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .header-title {
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
    color: var(--text-primary);
    min-width: 0;
  }

  .title-info {
    min-width: 0;
  }

  .title-info h2 {
    font-size: var(--text-lg);
    font-weight: var(--font-semibold);
    margin: 0 0 4px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .meta {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .type-badge {
    font-size: 10px;
    font-weight: var(--font-medium);
    padding: 2px 6px;
    border-radius: 4px;
    background: var(--bg-tertiary);
    color: var(--text-muted);
  }

  .meta-text {
    font-size: var(--text-xs);
    color: var(--text-muted);
  }

  .meta-separator {
    color: var(--text-muted);
    opacity: 0.5;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-shrink: 0;
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

  .modal-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-5);
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

  .spinner {
    width: 24px;
    height: 24px;
    border: 2px solid rgba(255, 255, 255, 0.2);
    border-top-color: var(--accent-hex);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
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
    line-height: 1.7;
    color: var(--text-primary);
  }

  .markdown-content :global(h1) {
    font-size: var(--text-2xl);
    font-weight: var(--font-bold);
    margin: 0 0 var(--space-4);
    padding-bottom: var(--space-2);
    border-bottom: 1px solid var(--border);
  }

  .markdown-content :global(h2) {
    font-size: var(--text-xl);
    font-weight: var(--font-semibold);
    margin: var(--space-6) 0 var(--space-3);
  }

  .markdown-content :global(h3) {
    font-size: var(--text-lg);
    font-weight: var(--font-semibold);
    margin: var(--space-5) 0 var(--space-2);
  }

  .markdown-content :global(p) {
    margin-bottom: var(--space-4);
  }

  .markdown-content :global(ul),
  .markdown-content :global(ol) {
    margin-bottom: var(--space-4);
    padding-left: var(--space-6);
  }

  .markdown-content :global(li) {
    margin-bottom: var(--space-1);
  }

  .markdown-content :global(code) {
    background: var(--bg-tertiary);
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 0.9em;
    font-family: "SF Mono", Monaco, "Cascadia Code", monospace;
  }

  .markdown-content :global(pre) {
    background: var(--bg-tertiary);
    padding: var(--space-4);
    border-radius: var(--radius-md);
    overflow-x: auto;
    margin: var(--space-4) 0;
    border: 1px solid var(--border);
  }

  .markdown-content :global(pre code) {
    background: transparent;
    padding: 0;
    font-size: var(--text-sm);
  }

  .markdown-content :global(blockquote) {
    margin: var(--space-4) 0;
    padding: var(--space-3) var(--space-4);
    border-left: 3px solid var(--accent-hex);
    background: var(--bg-secondary);
    border-radius: 0 var(--radius-md) var(--radius-md) 0;
  }

  .markdown-content :global(blockquote p) {
    margin: 0;
  }

  .markdown-content :global(a) {
    color: var(--accent-hex);
    text-decoration: none;
  }

  .markdown-content :global(a:hover) {
    text-decoration: underline;
  }

  .markdown-content :global(hr) {
    border: none;
    border-top: 1px solid var(--border);
    margin: var(--space-6) 0;
  }

  .markdown-content :global(table) {
    width: 100%;
    border-collapse: collapse;
    margin: var(--space-4) 0;
  }

  .markdown-content :global(th),
  .markdown-content :global(td) {
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--border);
    text-align: left;
  }

  .markdown-content :global(th) {
    background: var(--bg-secondary);
    font-weight: var(--font-semibold);
  }

  .text-content {
    margin: 0;
    padding: var(--space-4);
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    font-size: var(--text-sm);
    line-height: 1.6;
    overflow-x: auto;
    white-space: pre-wrap;
    word-wrap: break-word;
  }

  .text-content code {
    font-family: "SF Mono", Monaco, "Cascadia Code", monospace;
    color: var(--text-primary);
  }
</style>
