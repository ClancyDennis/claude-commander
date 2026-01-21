<script lang="ts">
  let {
    content = $bindable(""),
    placeholder = "Enter your instruction content here...",
    disabled = false,
  }: {
    content: string;
    placeholder?: string;
    disabled?: boolean;
  } = $props();

  let wordCount = $derived(
    content.trim() ? content.trim().split(/\s+/).length : 0
  );
  let charCount = $derived(content.length);
</script>

<div class="draft-editor">
  <textarea
    bind:value={content}
    {placeholder}
    {disabled}
    rows="12"
  ></textarea>
  <div class="stats">
    <span>{wordCount} words</span>
    <span class="separator">|</span>
    <span>{charCount} characters</span>
  </div>
</div>

<style>
  .draft-editor {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }

  textarea {
    width: 100%;
    padding: var(--space-md);
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 10px;
    color: var(--text-primary);
    font-size: 14px;
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    line-height: 1.6;
    resize: vertical;
    transition: all 0.2s ease;
    min-height: 300px;
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

  textarea::placeholder {
    color: var(--text-muted);
  }

  .stats {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-sm);
    font-size: 12px;
    color: var(--text-muted);
    padding-right: var(--space-xs);
  }

  .separator {
    color: var(--border);
  }
</style>
