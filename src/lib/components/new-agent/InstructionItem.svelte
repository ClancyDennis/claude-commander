<script lang="ts">
  import type { InstructionFileInfo } from "../../types";

  let {
    file,
    isSelected,
    isCreating,
    isGenerating,
    onToggle,
    onEdit,
    onGenerateSkill,
    onViewSkill,
    onDeleteSkill,
  }: {
    file: InstructionFileInfo;
    isSelected: boolean;
    isCreating: boolean;
    isGenerating: boolean;
    onToggle: () => void;
    onEdit: () => void;
    onGenerateSkill: () => void;
    onViewSkill: () => void;
    onDeleteSkill: () => void;
  } = $props();
</script>

<label class="instruction-item" class:selected={isSelected}>
  <input
    type="checkbox"
    checked={isSelected}
    onchange={onToggle}
    disabled={isCreating}
  />
  <div class="instruction-info">
    <span class="instruction-name">{file.name}</span>
    {#if file.hasSkill}
      <span class="skill-badge success">Skill</span>
    {/if}
  </div>
  <div class="instruction-actions">
    <!-- Edit button -->
    <button class="icon-btn tiny" onclick={(e) => { e.preventDefault(); onEdit() }} title="Edit with AI">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
        <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
      </svg>
    </button>
    {#if isGenerating}
      <span class="spinner"></span>
    {:else if file.hasSkill}
      <button class="icon-btn tiny" onclick={(e) => { e.preventDefault(); onViewSkill() }} title="View Skill">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="3"/>
          <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
        </svg>
      </button>
      <button class="icon-btn tiny danger" onclick={(e) => { e.preventDefault(); onDeleteSkill() }} title="Delete Skill">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18"/>
          <line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    {:else}
      <button class="icon-btn tiny primary" onclick={(e) => { e.preventDefault(); onGenerateSkill() }} title="Generate Skill">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 2L2 7l10 5 10-5-10-5z"/>
        </svg>
      </button>
    {/if}
  </div>
</label>

<style>
  .instruction-item {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-sm);
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .instruction-item:hover {
    background: var(--bg-tertiary);
  }

  .instruction-item.selected {
    background: rgba(124, 58, 237, 0.1);
    border-color: var(--accent);
  }

  .instruction-item input[type="checkbox"] {
    margin: 0;
    width: 16px;
    height: 16px;
  }

  .instruction-info {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .instruction-name {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .skill-badge {
    font-size: 10px;
    padding: 2px 6px;
    border-radius: 4px;
    background: rgba(34, 197, 94, 0.15);
    color: #22c55e;
  }

  .instruction-actions {
    display: flex;
    gap: 4px;
  }

  .icon-btn.tiny {
    padding: 4px;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 6px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    color: var(--text-secondary);
  }

  .icon-btn.tiny:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .icon-btn.tiny.danger:hover {
    color: #ef4444;
    background: rgba(239, 68, 68, 0.1);
  }

  .icon-btn.tiny svg {
    width: 14px;
    height: 14px;
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
</style>
