<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type {
    InstructionFileInfo,
    InstructionAnalysisResult,
    SuggestionStatus,
  } from "../types";
  import DraftEditor from "./instruction-editor/DraftEditor.svelte";
  import AnalysisPanel from "./instruction-editor/AnalysisPanel.svelte";
  import DiffView from "./instruction-editor/DiffView.svelte";

  type EditorMode = "draft" | "analyzing" | "review" | "saving";

  let {
    workingDir,
    existingFile = null,
    onClose,
    onSaved,
  }: {
    workingDir: string;
    existingFile?: InstructionFileInfo | null;
    onClose: () => void;
    onSaved: (filename: string) => void;
  } = $props();

  // State
  let mode = $state<EditorMode>("draft");
  // Intentionally capture initial filename (IIFE breaks reactive tracking)
  let filename = $state((() => existingFile?.name || "")());
  let content = $state("");
  let context = $state("");
  let error = $state("");

  // Analysis state
  let analysis = $state<InstructionAnalysisResult | null>(null);
  let suggestionStatuses = $state<Map<string, SuggestionStatus>>(new Map());

  // Load existing file content if editing
  $effect(() => {
    if (existingFile?.path) {
      loadExistingContent();
    }
  });

  async function loadExistingContent() {
    if (!existingFile?.path) return;
    try {
      const fileContent = await invoke<string>("get_instruction_file_content", {
        filePath: existingFile.path,
      });
      content = fileContent;
    } catch (e) {
      error = `Failed to load file: ${e}`;
    }
  }

  async function analyzeContent() {
    if (!content.trim()) {
      error = "Please enter some content to analyze";
      return;
    }

    mode = "analyzing";
    error = "";

    try {
      const result = await invoke<InstructionAnalysisResult>(
        "analyze_instruction_content",
        {
          content: content,
          context: context.trim() || null,
        }
      );

      analysis = result;
      suggestionStatuses = new Map(
        result.suggestions.map((s) => [s.id, "pending" as SuggestionStatus])
      );
      mode = "review";
    } catch (e) {
      error = `Analysis failed: ${e}`;
      mode = "draft";
    }
  }

  async function saveContent(applyChanges: boolean) {
    // Validate filename
    let finalFilename = filename.trim();
    if (!finalFilename) {
      error = "Please enter a filename";
      return;
    }
    if (!finalFilename.endsWith(".md") && !finalFilename.endsWith(".txt")) {
      finalFilename += ".md";
    }

    mode = "saving";
    error = "";

    try {
      let finalContent = content;

      if (applyChanges && analysis) {
        // Get accepted suggestion IDs
        const acceptedIds = Array.from(suggestionStatuses.entries())
          .filter(([_, status]) => status === "accepted")
          .map(([id, _]) => id);

        if (acceptedIds.length > 0) {
          finalContent = await invoke<string>("apply_instruction_suggestions", {
            originalContent: content,
            acceptedSuggestionIds: acceptedIds,
            analysisResult: analysis,
          });
        }
      }

      await invoke("save_instruction_file", {
        workingDir,
        filename: finalFilename,
        content: finalContent,
      });

      onSaved(finalFilename);
      onClose();
    } catch (e) {
      error = `Failed to save: ${e}`;
      mode = "review";
    }
  }

  function backToDraft() {
    mode = "draft";
    analysis = null;
    suggestionStatuses = new Map();
  }

  function acceptAllSuggestions() {
    if (!analysis) return;
    suggestionStatuses = new Map(
      analysis.suggestions.map((s) => [s.id, "accepted" as SuggestionStatus])
    );
  }

  function rejectAllSuggestions() {
    if (!analysis) return;
    suggestionStatuses = new Map(
      analysis.suggestions.map((s) => [s.id, "rejected" as SuggestionStatus])
    );
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onClose();
    } else if (e.key === "Enter" && e.ctrlKey) {
      if (mode === "draft") {
        analyzeContent();
      } else if (mode === "review") {
        saveContent(true);
      }
    }
  }

  // Computed improved content for diff view
  let improvedContent = $derived(analysis?.improvedContent || content);

  let isEditMode = $derived(!!existingFile);
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay animate-fade-in" onclick={onClose}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="dialog animate-slide-up"
    class:wide={mode === "review"}
    onclick={(e) => e.stopPropagation()}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <header>
      <div class="header-content">
        <div class="dialog-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
            <polyline points="14 2 14 8 20 8"/>
            <line x1="16" y1="13" x2="8" y2="13"/>
            <line x1="16" y1="17" x2="8" y2="17"/>
          </svg>
        </div>
        <div>
          <h2>
            {#if mode === "review"}
              Review Suggestions
            {:else}
              {isEditMode ? "Edit" : "Create"} Instruction
            {/if}
          </h2>
          <p class="subtitle">
            {#if mode === "draft"}
              {isEditMode ? "Edit your instruction file" : "Write a new instruction file for AI agents"}
            {:else if mode === "analyzing"}
              Analyzing your instruction...
            {:else if mode === "review"}
              Review AI suggestions and apply improvements
            {:else}
              Saving your instruction...
            {/if}
          </p>
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
      {#if mode === "draft" || mode === "analyzing"}
        <!-- Draft Mode -->
        <div class="draft-mode">
          <label class="filename-label">
            <span class="label-text">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                <polyline points="14 2 14 8 20 8"/>
              </svg>
              Filename
            </span>
            <input
              type="text"
              bind:value={filename}
              placeholder="my-instruction.md"
              disabled={mode === "analyzing" || isEditMode}
            />
          </label>

          <label class="content-label">
            <span class="label-text">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
                <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
              </svg>
              Content
            </span>
            <DraftEditor
              bind:content
              placeholder="Enter your instruction content here...

Tips for good instructions:
- Be specific about the desired behavior
- Include examples when helpful
- Specify any constraints or requirements
- Consider edge cases"
              disabled={mode === "analyzing"}
            />
          </label>

          <label class="context-label">
            <span class="label-text">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"/>
                <line x1="12" y1="16" x2="12" y2="12"/>
                <line x1="12" y1="8" x2="12.01" y2="8"/>
              </svg>
              Context <span class="optional">(optional)</span>
            </span>
            <input
              type="text"
              bind:value={context}
              placeholder="Brief description of what this instruction is for..."
              disabled={mode === "analyzing"}
            />
          </label>
        </div>
      {:else if mode === "review" && analysis}
        <!-- Review Mode -->
        <div class="review-mode">
          <div class="review-left">
            <AnalysisPanel
              {analysis}
              bind:suggestionStatuses
              onAcceptAll={acceptAllSuggestions}
              onRejectAll={rejectAllSuggestions}
            />
          </div>
          <div class="review-right">
            <DiffView
              original={content}
              improved={improvedContent}
            />
          </div>
        </div>
      {/if}

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
      {#if mode === "draft"}
        <button class="secondary" onclick={onClose}>
          Cancel
        </button>
        <button
          class="secondary"
          onclick={() => saveContent(false)}
          disabled={!content.trim() || !filename.trim()}
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/>
            <polyline points="17 21 17 13 7 13 7 21"/>
            <polyline points="7 3 7 8 15 8"/>
          </svg>
          Save Draft
        </button>
        <button
          class="primary"
          onclick={analyzeContent}
          disabled={!content.trim()}
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="3"/>
            <path d="M12 1v4M12 19v4M1 12h4M19 12h4"/>
          </svg>
          Analyze with AI
        </button>
      {:else if mode === "analyzing"}
        <button class="secondary" disabled>
          Cancel
        </button>
        <button class="primary" disabled>
          <span class="spinner"></span>
          Analyzing...
        </button>
      {:else if mode === "review"}
        <button class="secondary" onclick={backToDraft}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="19" y1="12" x2="5" y2="12"/>
            <polyline points="12 19 5 12 12 5"/>
          </svg>
          Edit Draft
        </button>
        <button
          class="secondary"
          onclick={() => saveContent(false)}
        >
          Save Without Changes
        </button>
        <button
          class="primary"
          onclick={() => saveContent(true)}
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="20 6 9 17 4 12"/>
          </svg>
          Apply & Save
        </button>
      {:else if mode === "saving"}
        <button class="secondary" disabled>
          Cancel
        </button>
        <button class="primary" disabled>
          <span class="spinner"></span>
          Saving...
        </button>
      {/if}
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
    width: 700px;
    max-width: 100%;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    background-color: var(--bg-secondary);
    border-radius: 20px;
    border: 1px solid var(--border);
    box-shadow: var(--shadow-lg), 0 0 60px rgba(124, 58, 237, 0.1);
    overflow: hidden;
    transition: width 0.3s ease;
  }

  .dialog.wide {
    width: 1200px;
  }

  header {
    flex-shrink: 0;
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
    border: none;
    cursor: pointer;
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
    flex: 1;
    padding: var(--space-lg);
    overflow-y: auto;
  }

  .draft-mode {
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
  }

  .review-mode {
    display: grid;
    grid-template-columns: 350px 1fr;
    gap: var(--space-lg);
    height: 100%;
    min-height: 500px;
  }

  .review-left {
    overflow-y: auto;
  }

  .review-right {
    overflow: hidden;
  }

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

  .optional {
    font-weight: 400;
    color: var(--text-muted);
  }

  input[type="text"] {
    width: 100%;
    padding: var(--space-md);
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 10px;
    color: var(--text-primary);
    font-size: 14px;
    transition: all 0.2s ease;
  }

  input[type="text"]:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-glow);
  }

  input[type="text"]:disabled {
    opacity: 0.6;
    cursor: not-allowed;
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
    flex-shrink: 0;
    padding: var(--space-lg);
    display: flex;
    justify-content: flex-end;
    gap: var(--space-md);
    border-top: 1px solid var(--border);
    background-color: var(--bg-tertiary);
  }

  footer button {
    min-width: 120px;
    padding: 10px 20px;
    border-radius: 10px;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    border: none;
  }

  .secondary {
    background: var(--bg-elevated);
    color: var(--text-secondary);
    border: 1px solid var(--border);
  }

  .secondary:hover:not(:disabled) {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .primary {
    background: var(--accent);
    color: white;
  }

  .primary:hover:not(:disabled) {
    background: var(--accent-hover, #7c3aed);
    transform: scale(1.02);
  }

  button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  button svg {
    width: 18px;
    height: 18px;
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
