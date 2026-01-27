<script lang="ts">
  import type { AgentOutput } from "../types";
  import { invoke } from "@tauri-apps/api/core";
  import HelpTip from "./new-agent/HelpTip.svelte";
  import { useAsyncData } from "../hooks/useAsyncData.svelte";

  let {
    outputs,
    agentId,
    onClose,
  }: {
    outputs: AgentOutput[];
    agentId: string;
    onClose: () => void;
  } = $props();

  let format = $state<"json" | "markdown" | "html" | "text">("json");
  let includeTimestamps = $state(true);
  let includeToolCalls = $state(true);
  let includeMetadata = $state(true);
  let filterType = $state<string>("all");

  // Export action state via useAsyncData
  const exportAction = useAsyncData<void>(() => performExport());

  // Get unique output types
  const outputTypes = $derived.by(() => {
    const types = new Set<string>();
    outputs.forEach(o => types.add(o.type));
    return Array.from(types).sort();
  });

  // Filter outputs based on settings - skip spread when no filter needed
  const filteredOutputs = $derived.by(() => {
    const needsFilter = filterType !== "all" || !includeToolCalls;
    if (!needsFilter) return outputs;

    let filtered = outputs;
    if (filterType !== "all") {
      filtered = filtered.filter(o => o.type === filterType);
    }
    if (!includeToolCalls) {
      filtered = filtered.filter(o => o.type !== "tool_use" && o.type !== "tool_result");
    }
    return filtered;
  });

  function formatJSON(outputs: AgentOutput[]): string {
    return JSON.stringify(
      outputs.map(o => ({
        type: o.type,
        content: o.content,
        timestamp: includeTimestamps ? o.timestamp.toISOString() : undefined,
        parsedJson: includeMetadata ? o.parsedJson : undefined,
        metadata: includeMetadata ? o.metadata : undefined,
      })),
      null,
      2
    );
  }

  function formatMarkdown(outputs: AgentOutput[]): string {
    let md = `# Agent Output Export\n\n`;
    md += `Agent ID: ${agentId}\n`;
    md += `Exported: ${new Date().toISOString()}\n`;
    md += `Total Outputs: ${outputs.length}\n\n---\n\n`;

    filteredOutputs.forEach((output, i) => {
      md += `## Output ${i + 1}\n\n`;
      if (includeTimestamps) {
        md += `**Timestamp:** ${output.timestamp.toLocaleString()}\n\n`;
      }
      md += `**Type:** \`${output.type}\`\n\n`;
      md += `### Content\n\n\`\`\`\n${output.content}\n\`\`\`\n\n`;

      if (includeMetadata && output.metadata) {
        md += `**Metadata:**\n`;
        md += `- Language: ${output.metadata.language || "N/A"}\n`;
        md += `- Line Count: ${output.metadata.lineCount || "N/A"}\n`;
        md += `- Size: ${output.metadata.byteSize || 0} bytes\n`;
        md += `- Truncated: ${output.metadata.isTruncated ? "Yes" : "No"}\n`;
      }

      md += `\n---\n\n`;
    });

    return md;
  }

  function formatHTML(outputs: AgentOutput[]): string {
    let html = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Agent Output Export</title>
  <style>
    body {
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      max-width: 1200px;
      margin: 0 auto;
      padding: 2rem;
      background: #f5f5f5;
      color: #333;
    }
    .header {
      background: white;
      padding: 2rem;
      border-radius: 12px;
      margin-bottom: 2rem;
      box-shadow: 0 2px 8px rgba(0,0,0,0.1);
    }
    .output-item {
      background: white;
      padding: 1.5rem;
      border-radius: 12px;
      margin-bottom: 1.5rem;
      box-shadow: 0 2px 8px rgba(0,0,0,0.1);
    }
    .output-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 1rem;
      padding-bottom: 0.5rem;
      border-bottom: 2px solid #f0f0f0;
    }
    .output-type {
      background: #f0705a;
      color: white;
      padding: 0.25rem 0.75rem;
      border-radius: 6px;
      font-size: 0.875rem;
      font-weight: 600;
      text-transform: uppercase;
    }
    .timestamp {
      color: #666;
      font-size: 0.875rem;
    }
    pre {
      background: #f8f8f8;
      padding: 1rem;
      border-radius: 8px;
      overflow-x: auto;
      white-space: pre-wrap;
      word-wrap: break-word;
    }
    .metadata {
      margin-top: 1rem;
      padding: 0.75rem;
      background: #f8f8f8;
      border-radius: 8px;
      font-size: 0.875rem;
      color: #666;
    }
  </style>
</head>
<body>
  <div class="header">
    <h1>Agent Output Export</h1>
    <p><strong>Agent ID:</strong> ${agentId}</p>
    <p><strong>Exported:</strong> ${new Date().toLocaleString()}</p>
    <p><strong>Total Outputs:</strong> ${filteredOutputs.length}</p>
  </div>
`;

    filteredOutputs.forEach((output, i) => {
      html += `
  <div class="output-item">
    <div class="output-header">
      <span class="output-type">${output.type}</span>
      ${includeTimestamps ? `<span class="timestamp">${output.timestamp.toLocaleString()}</span>` : ""}
    </div>
    <pre>${escapeHTML(output.content)}</pre>
    ${includeMetadata && output.metadata ? `
    <div class="metadata">
      <strong>Metadata:</strong>
      Language: ${output.metadata.language || "N/A"} |
      Lines: ${output.metadata.lineCount || "N/A"} |
      Size: ${output.metadata.byteSize || 0} bytes
    </div>
    ` : ""}
  </div>
`;
    });

    html += `
</body>
</html>`;

    return html;
  }

  function formatText(outputs: AgentOutput[]): string {
    let text = `Agent Output Export\n`;
    text += `===================\n\n`;
    text += `Agent ID: ${agentId}\n`;
    text += `Exported: ${new Date().toLocaleString()}\n`;
    text += `Total Outputs: ${filteredOutputs.length}\n\n`;
    text += `---\n\n`;

    filteredOutputs.forEach((output, i) => {
      text += `Output ${i + 1}\n`;
      if (includeTimestamps) {
        text += `Timestamp: ${output.timestamp.toLocaleString()}\n`;
      }
      text += `Type: ${output.type}\n\n`;
      text += `${output.content}\n\n`;
      text += `---\n\n`;
    });

    return text;
  }

  function escapeHTML(str: string): string {
    return str
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#039;");
  }

  async function performExport(): Promise<void> {
    let content = "";
    let filename = "";

    switch (format) {
      case "json":
        content = formatJSON(filteredOutputs);
        filename = `agent-${agentId}-output-${Date.now()}.json`;
        break;
      case "markdown":
        content = formatMarkdown(filteredOutputs);
        filename = `agent-${agentId}-output-${Date.now()}.md`;
        break;
      case "html":
        content = formatHTML(filteredOutputs);
        filename = `agent-${agentId}-output-${Date.now()}.html`;
        break;
      case "text":
        content = formatText(filteredOutputs);
        filename = `agent-${agentId}-output-${Date.now()}.txt`;
        break;
    }

    // Use the browser's download API
    const blob = new Blob([content], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);

    onClose();
  }

  async function handleExport() {
    await exportAction.fetch();
  }

  // Preview content
  const previewContent = $derived.by(() => {
    const preview = filteredOutputs.slice(0, 3);
    switch (format) {
      case "json":
        return formatJSON(preview);
      case "markdown":
        return formatMarkdown(preview);
      case "text":
        return formatText(preview);
      default:
        return "Preview not available for HTML format";
    }
  });
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onclick={onClose}></div>
<div class="dialog">
  <header>
    <h2>Export Output</h2>
    <button class="close-btn" onclick={onClose} aria-label="Close dialog">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="18" y1="6" x2="6" y2="18"/>
        <line x1="6" y1="6" x2="18" y2="18"/>
      </svg>
    </button>
  </header>

  <div class="content">
    <div class="section">
      <h3>Format</h3>
      <div class="format-options">
        <label class="format-option" class:selected={format === "json"}>
          <input type="radio" bind:group={format} value="json" />
          <div class="format-card">
            <div class="format-icon">📄</div>
            <div class="format-name">JSON</div>
            <div class="format-desc">Structured data</div>
          </div>
        </label>
        <label class="format-option" class:selected={format === "markdown"}>
          <input type="radio" bind:group={format} value="markdown" />
          <div class="format-card">
            <div class="format-icon">📝</div>
            <div class="format-name">Markdown</div>
            <div class="format-desc">Human-readable</div>
          </div>
        </label>
        <label class="format-option" class:selected={format === "html"}>
          <input type="radio" bind:group={format} value="html" />
          <div class="format-card">
            <div class="format-icon">🌐</div>
            <div class="format-name">HTML</div>
            <div class="format-desc">Web page</div>
          </div>
        </label>
        <label class="format-option" class:selected={format === "text"}>
          <input type="radio" bind:group={format} value="text" />
          <div class="format-card">
            <div class="format-icon">📃</div>
            <div class="format-name">Plain Text</div>
            <div class="format-desc">Simple format</div>
          </div>
        </label>
      </div>
    </div>

    <div class="section">
      <h3>Options</h3>
      <div class="options-grid">
        <label class="checkbox-label">
          <input type="checkbox" bind:checked={includeTimestamps} />
          <span>Include timestamps <HelpTip text="Add ISO-8601 timestamps to each output entry for tracking chronology." placement="right" /></span>
        </label>
        <label class="checkbox-label">
          <input type="checkbox" bind:checked={includeToolCalls} />
          <span>Include tool calls <HelpTip text="Include tool_use and tool_result entries showing agent actions." placement="right" /></span>
        </label>
        <label class="checkbox-label">
          <input type="checkbox" bind:checked={includeMetadata} />
          <span>Include metadata <HelpTip text="Add language detection, line counts, and truncation info to each output." placement="right" /></span>
        </label>
      </div>
    </div>

    <div class="section">
      <h3>Filter</h3>
      <select bind:value={filterType} class="filter-select">
        <option value="all">All Types ({outputs.length})</option>
        {#each outputTypes as type}
          <option value={type}>{type}</option>
        {/each}
      </select>
    </div>

    <div class="section">
      <h3>Preview (first 3 items)</h3>
      <div class="preview">
        <pre>{previewContent}</pre>
      </div>
    </div>

    {#if exportAction.error}
      <div class="error">
        Export failed: {exportAction.error}
      </div>
    {/if}

    <div class="summary">
      Exporting {filteredOutputs.length} output{filteredOutputs.length !== 1 ? "s" : ""}
    </div>
  </div>

  <footer>
    <button class="secondary" onclick={onClose} disabled={exportAction.loading}>
      Cancel
    </button>
    <button class="primary" onclick={handleExport} disabled={exportAction.loading}>
      {#if exportAction.loading}
        <div class="spinner"></div>
        Exporting...
      {:else}
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
          <polyline points="7 10 12 15 17 10"/>
          <line x1="12" y1="15" x2="12" y2="3"/>
        </svg>
        Export
      {/if}
    </button>
  </footer>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(4px);
    z-index: 1000;
    animation: fadeIn 0.2s ease;
  }

  .dialog {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    background-color: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 16px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
    z-index: 1001;
    width: 90%;
    max-width: 800px;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    animation: slideUp 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  @keyframes slideUp {
    from {
      opacity: 0;
      transform: translate(-50%, -45%);
    }
    to {
      opacity: 1;
      transform: translate(-50%, -50%);
    }
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-lg);
    border-bottom: 1px solid var(--border);
  }

  h2 {
    font-size: 20px;
    font-weight: 700;
    margin: 0;
  }

  .close-btn {
    padding: 8px;
    background: none;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    color: var(--text-secondary);
    transition: all 0.2s ease;
  }

  .close-btn:hover {
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .close-btn svg {
    width: 20px;
    height: 20px;
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-lg);
  }

  .section {
    margin-bottom: var(--space-lg);
  }

  .section:last-child {
    margin-bottom: 0;
  }

  h3 {
    font-size: 15px;
    font-weight: 600;
    margin: 0 0 var(--space-md) 0;
    color: var(--text-primary);
  }

  .format-options {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: var(--space-md);
  }

  .format-option input {
    display: none;
  }

  .format-card {
    padding: var(--space-md);
    border: 2px solid var(--border);
    border-radius: 12px;
    background-color: var(--bg-secondary);
    cursor: pointer;
    transition: all 0.2s ease;
    text-align: center;
  }

  .format-option:hover .format-card {
    border-color: var(--accent);
    background-color: var(--bg-primary);
  }

  .format-option.selected .format-card {
    border-color: var(--accent);
    background: linear-gradient(135deg, rgba(240, 112, 90, 0.1) 0%, rgba(240, 112, 90, 0.05) 100%);
  }

  .format-icon {
    font-size: 32px;
    margin-bottom: var(--space-sm);
  }

  .format-name {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 2px;
  }

  .format-desc {
    font-size: 12px;
    color: var(--text-muted);
  }

  .options-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: var(--space-md);
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-sm);
    border-radius: 8px;
    cursor: pointer;
    transition: background-color 0.2s ease;
  }

  .checkbox-label:hover {
    background-color: var(--bg-secondary);
  }

  .checkbox-label input[type="checkbox"] {
    width: 18px;
    height: 18px;
    cursor: pointer;
  }

  .checkbox-label span {
    font-size: 14px;
    color: var(--text-primary);
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .filter-select {
    width: 100%;
    padding: 10px 14px;
    border: 1px solid var(--border);
    border-radius: 10px;
    background-color: var(--bg-secondary);
    color: var(--text-primary);
    font-size: 14px;
    cursor: pointer;
  }

  .filter-select:focus {
    outline: none;
    border-color: var(--accent);
  }

  .preview {
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: var(--space-md);
    max-height: 300px;
    overflow-y: auto;
  }

  .preview pre {
    margin: 0;
    font-family: 'SF Mono', 'Monaco', 'Menlo', monospace;
    font-size: 12px;
    line-height: 1.5;
    color: var(--text-primary);
    white-space: pre-wrap;
    word-wrap: break-word;
  }

  .summary {
    padding: var(--space-md);
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 10px;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-secondary);
    text-align: center;
  }

  .error {
    padding: var(--space-md);
    background-color: var(--error-glow);
    border: 1px solid var(--error);
    border-radius: 10px;
    color: var(--error);
    font-size: 14px;
    margin-top: var(--space-md);
  }

  footer {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-md);
    padding: var(--space-lg);
    border-top: 1px solid var(--border);
  }

  footer button {
    padding: 12px 24px;
  }

  footer button svg {
    width: 18px;
    height: 18px;
  }

  .spinner {
    width: 16px;
    height: 16px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
