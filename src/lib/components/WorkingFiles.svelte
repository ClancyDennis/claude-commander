<script lang="ts">
  import { selectedAgentTools } from "../stores/agents";
  import type { ToolEvent } from "../types";
  import HelpTip from "./new-agent/HelpTip.svelte";

  interface FileOperation {
    path: string;
    fileName: string;
    operation: 'read' | 'write' | 'edit';
    status: 'pending' | 'success' | 'failed';
    timestamp: Date;
  }

  function getFileName(path: string): string {
    return path.split('/').pop() || path;
  }

  function deriveWorkingFiles(events: ToolEvent[]): FileOperation[] {
    const fileMap = new Map<string, FileOperation>();

    // Process events chronologically
    for (const event of events) {
      if (!['Read', 'Write', 'Edit'].includes(event.toolName)) continue;

      const filePath = event.toolInput?.file_path as string;
      if (!filePath) continue;

      // Determine status from event
      let status: 'pending' | 'success' | 'failed' = 'pending';
      if (event.hookEventName === 'PostToolUse') {
        status = event.status === 'failed' ? 'failed' : 'success';
      }

      fileMap.set(filePath, {
        path: filePath,
        fileName: getFileName(filePath),
        operation: event.toolName.toLowerCase() as 'read' | 'write' | 'edit',
        status,
        timestamp: event.timestamp
      });
    }

    // Sort by timestamp descending (most recent first)
    return Array.from(fileMap.values())
      .sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime());
  }

  // Derive working files reactively
  const workingFiles = $derived(deriveWorkingFiles($selectedAgentTools));

  // Group files by status for summary
  const summary = $derived({
    pending: workingFiles.filter(f => f.status === 'pending').length,
    modified: workingFiles.filter(f => f.status === 'success' && f.operation !== 'read').length,
    read: workingFiles.filter(f => f.status === 'success' && f.operation === 'read').length,
    failed: workingFiles.filter(f => f.status === 'failed').length
  });
</script>

<div class="working-files">
  <div class="panel-header">
    <h3>Working Files <HelpTip text="Files the agent has read or modified, sorted by most recent activity." placement="right" /></h3>
    {#if workingFiles.length > 0}
      <span class="file-count">{workingFiles.length}</span>
    {/if}
  </div>

  {#if workingFiles.length === 0}
    <div class="empty-state">
      <div class="empty-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/>
          <polyline points="13 2 13 9 20 9"/>
        </svg>
      </div>
      <p>No file activity yet</p>
      <span class="hint">Files will appear here as the agent reads and writes them</span>
    </div>
  {:else}
    <!-- Summary badges -->
    <div class="summary">
      {#if summary.pending > 0}
        <span class="badge pending">{summary.pending} active</span>
      {/if}
      {#if summary.modified > 0}
        <span class="badge modified">{summary.modified} modified</span>
      {/if}
      {#if summary.read > 0}
        <span class="badge read">{summary.read} read</span>
      {/if}
      {#if summary.failed > 0}
        <span class="badge failed">{summary.failed} failed</span>
      {/if}
    </div>

    <ul class="file-list">
      {#each workingFiles as file (file.path)}
        <li class="file-item {file.status}" title={file.path}>
          <span class="icon">
            {#if file.status === 'pending'}
              <span class="spinner"></span>
            {:else if file.status === 'failed'}
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"/>
                <line x1="15" y1="9" x2="9" y2="15"/>
                <line x1="9" y1="9" x2="15" y2="15"/>
              </svg>
            {:else if file.operation === 'read'}
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
                <circle cx="12" cy="12" r="3"/>
              </svg>
            {:else}
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
                <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
              </svg>
            {/if}
          </span>
          <span class="file-name">{file.fileName}</span>
          <span class="operation">({file.operation})</span>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .working-files {
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-md) var(--space-lg);
    border-bottom: 1px solid var(--border);
    background-color: var(--bg-tertiary);
  }

  .panel-header h3 {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .file-count {
    font-size: 12px;
    background-color: var(--accent);
    color: white;
    padding: 2px 8px;
    border-radius: 10px;
    font-weight: 600;
  }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-xl);
    text-align: center;
  }

  .empty-icon {
    width: 64px;
    height: 64px;
    border-radius: 16px;
    background: linear-gradient(135deg, var(--bg-secondary) 0%, var(--bg-tertiary) 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: var(--space-md);
    border: 1px solid var(--border);
  }

  .empty-icon svg {
    width: 32px;
    height: 32px;
    color: var(--text-muted);
  }

  .empty-state p {
    font-size: 14px;
    color: var(--text-secondary);
    margin: 0 0 var(--space-xs);
  }

  .empty-state .hint {
    font-size: 12px;
    color: var(--text-muted);
  }

  .summary {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-xs);
    padding: var(--space-sm) var(--space-lg);
    border-bottom: 1px solid var(--border);
  }

  .badge {
    font-size: 11px;
    padding: 2px 8px;
    border-radius: 10px;
    font-weight: 500;
  }

  .badge.pending {
    background-color: rgba(245, 158, 11, 0.15);
    color: var(--warning);
  }

  .badge.modified {
    background-color: rgba(34, 197, 94, 0.15);
    color: var(--success);
  }

  .badge.read {
    background-color: rgba(240, 112, 90, 0.15);
    color: var(--accent);
  }

  .badge.failed {
    background-color: var(--error-glow);
    color: var(--error);
  }

  .file-list {
    flex: 1;
    overflow-y: auto;
    list-style: none;
    margin: 0;
    padding: var(--space-sm);
  }

  .file-item {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-sm) var(--space-md);
    border-radius: 8px;
    margin-bottom: 2px;
    transition: background-color 0.15s ease;
  }

  .file-item:hover {
    background-color: var(--bg-tertiary);
  }

  .file-item .icon {
    width: 18px;
    height: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .file-item .icon svg {
    width: 16px;
    height: 16px;
  }

  .file-item.pending .icon {
    color: var(--warning);
  }

  .file-item.success .icon {
    color: var(--success);
  }

  .file-item.failed .icon {
    color: var(--error);
  }

  /* Read operation uses accent color */
  .file-item.success:has(.operation:contains(read)) .icon {
    color: var(--accent);
  }

  .spinner {
    width: 14px;
    height: 14px;
    border: 2px solid var(--warning);
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .file-name {
    flex: 1;
    font-size: 13px;
    font-family: 'SF Mono', 'Monaco', 'Menlo', monospace;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .operation {
    font-size: 11px;
    color: var(--text-muted);
    flex-shrink: 0;
  }
</style>
