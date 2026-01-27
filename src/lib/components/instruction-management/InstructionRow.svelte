<script lang="ts">
  import type { InstructionFileInfo } from "../../types";
  import { FileText, Edit2, Trash2, Eye, Zap, X } from "lucide-svelte";

  let {
    file,
    isGeneratingSkill = false,
    onView,
    onEdit,
    onDelete,
    onGenerateSkill,
    onViewSkill,
    onDeleteSkill,
  }: {
    file: InstructionFileInfo;
    isGeneratingSkill?: boolean;
    onView: () => void;
    onEdit: () => void;
    onDelete: () => void;
    onGenerateSkill: () => void;
    onViewSkill: () => void;
    onDeleteSkill: () => void;
  } = $props();

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
    });
  }
</script>

<div class="instruction-row">
  <div class="row-icon">
    <FileText size={18} />
  </div>

  <div class="row-content">
    <div class="row-header">
      <span class="filename">{file.name}</span>
      <div class="badges">
        <span class="type-badge">.{file.fileType}</span>
        {#if file.hasSkill}
          <span class="skill-badge">
            <Zap size={10} />
            skill
          </span>
        {/if}
      </div>
    </div>
    <div class="row-meta">
      <span class="meta-item">{formatFileSize(file.size)}</span>
      <span class="meta-separator">·</span>
      <span class="meta-item">Modified {formatDate(file.modified)}</span>
    </div>
  </div>

  <div class="row-actions">
    {#if isGeneratingSkill}
      <div class="spinner-container">
        <span class="spinner"></span>
      </div>
    {:else if file.hasSkill}
      <button class="action-btn" onclick={onViewSkill} title="View Skill">
        <Eye size={14} />
        <span>View</span>
      </button>
      <button class="action-btn danger" onclick={onDeleteSkill} title="Delete Skill">
        <X size={14} />
        <span>Skill</span>
      </button>
    {:else}
      <button class="action-btn primary" onclick={onGenerateSkill} title="Generate Skill">
        <Zap size={14} />
        <span>Skill</span>
      </button>
    {/if}
    <button class="action-btn" onclick={onView} title="View">
      <Eye size={14} />
    </button>
    <button class="action-btn" onclick={onEdit} title="Edit">
      <Edit2 size={14} />
    </button>
    <button class="action-btn danger" onclick={onDelete} title="Delete">
      <Trash2 size={14} />
    </button>
  </div>
</div>

<style>
  .instruction-row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    transition: all 0.15s ease;
  }

  .instruction-row:hover {
    background: var(--bg-tertiary);
    border-color: rgba(255, 255, 255, 0.1);
  }

  .row-icon {
    flex-shrink: 0;
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
    color: var(--text-muted);
  }

  .row-content {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .row-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .filename {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .badges {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .type-badge {
    font-size: 10px;
    font-weight: var(--font-medium);
    padding: 2px 6px;
    border-radius: 4px;
    background: var(--bg-elevated);
    color: var(--text-muted);
    text-transform: lowercase;
  }

  .skill-badge {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    font-size: 10px;
    font-weight: var(--font-medium);
    padding: 2px 6px;
    border-radius: 4px;
    background: rgba(34, 197, 94, 0.15);
    color: #22c55e;
  }

  .row-meta {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: var(--text-xs);
    color: var(--text-muted);
  }

  .meta-separator {
    opacity: 0.5;
  }

  .row-actions {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
    opacity: 0.6;
    transition: opacity 0.15s ease;
  }

  .instruction-row:hover .row-actions {
    opacity: 1;
  }

  .action-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 6px 10px;
    font-size: 12px;
    font-weight: var(--font-medium);
    border-radius: var(--radius-sm);
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .action-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
    border-color: rgba(255, 255, 255, 0.15);
  }

  .action-btn.primary {
    background: rgba(var(--accent-rgb), 0.1);
    border-color: rgba(var(--accent-rgb), 0.3);
    color: var(--accent-hex);
  }

  .action-btn.primary:hover {
    background: rgba(var(--accent-rgb), 0.2);
    border-color: var(--accent-hex);
  }

  .action-btn.danger:hover {
    background: rgba(239, 68, 68, 0.1);
    border-color: rgba(239, 68, 68, 0.3);
    color: #ef4444;
  }

  .spinner-container {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 60px;
  }

  .spinner {
    width: 16px;
    height: 16px;
    border: 2px solid rgba(255, 255, 255, 0.2);
    border-top-color: var(--accent-hex);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
