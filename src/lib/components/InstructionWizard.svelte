<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type {
    InstructionDraft,
    TestAgentSession,
  } from "../types";
  import DraftEditor from "./instruction-editor/DraftEditor.svelte";
  import GoalInput from "./instruction-wizard/GoalInput.svelte";
  import DraftPreview from "./instruction-wizard/DraftPreview.svelte";
  import TestRunner from "./instruction-wizard/TestRunner.svelte";
  import { FileText, Sparkles, Play, Layers, ChevronLeft } from "lucide-svelte";

  interface Props {
    workingDir: string;
    onClose: () => void;
    onSaved: (filename: string) => void;
  }

  let { workingDir, onClose, onSaved }: Props = $props();

  // Wizard state
  let currentStep = $state(1);
  const TOTAL_STEPS = 4;

  // Step 1: Goal input
  let goalDescription = $state("");
  let context = $state("");
  let isGenerating = $state(false);

  // Step 2-3: Draft
  let draft = $state<InstructionDraft | null>(null);
  let editedContent = $state("");
  let filename = $state("");

  // Step 4: Test
  let testSession = $state<TestAgentSession | null>(null);
  let testPrompt = $state("");

  // Enhancement state
  let isEnhancing = $state(false);
  let enhancementApplied = $state(false);

  // Error handling
  let error = $state("");

  function goToStep(step: number) {
    currentStep = step;
    error = "";
  }

  async function handleGenerateDraft() {
    if (!goalDescription.trim()) {
      error = "Please describe your goal";
      return;
    }

    isGenerating = true;
    error = "";

    try {
      const result = await invoke<InstructionDraft>("generate_instruction_draft", {
        goalDescription: goalDescription.trim(),
        context: context.trim() || null,
      });

      draft = result;
      editedContent = result.content;
      filename = result.suggestedFilename;
      testPrompt = result.suggestedTestPrompt;
      goToStep(2);
    } catch (e) {
      error = `Failed to generate draft: ${e}`;
    } finally {
      isGenerating = false;
    }
  }

  async function handleStartTest() {
    if (!editedContent.trim()) {
      error = "No instruction content to test";
      return;
    }

    error = "";
    enhancementApplied = false; // Reset for new test

    try {
      const session = await invoke<TestAgentSession>("create_test_agent", {
        instructionContent: editedContent,
        testPrompt: testPrompt,
        potentialRequirements: draft?.potentialRequirements ?? [],
        workingDir: workingDir,
      });

      testSession = session;
      goToStep(4);
    } catch (e) {
      error = `Failed to start test: ${e}`;
    }
  }

  async function handleAnalyzeResults() {
    if (!testSession) return;

    isEnhancing = true;
    error = "";

    try {
      // Call AI to enhance the instruction based on test transcript
      const enhancedContent = await invoke<string>("enhance_instruction_from_test", {
        originalInstruction: editedContent,
        agentId: testSession.agentId,
      });

      // Update the instruction content with enhanced version
      editedContent = enhancedContent;
      enhancementApplied = true;

      // Go back to edit step to show the enhanced instruction
      goToStep(3);
    } catch (e) {
      error = `Failed to enhance instruction: ${e}`;
      // Fall back to showing edit step anyway
      goToStep(3);
    } finally {
      isEnhancing = false;
    }
  }

  async function handleStopTest() {
    if (!testSession) return;

    try {
      await invoke("stop_test_agent", {
        agentId: testSession.agentId,
        tempInstructionFile: testSession.tempInstructionFile,
      });

      // Analyze what we have
      await handleAnalyzeResults();
    } catch (e) {
      error = `Failed to stop test: ${e}`;
    }
  }

  async function handleSaveInstruction() {
    if (!editedContent.trim() || !filename.trim()) {
      error = "Please provide content and filename";
      return;
    }

    let finalFilename = filename.trim();
    if (!finalFilename.endsWith(".md") && !finalFilename.endsWith(".txt")) {
      finalFilename += ".md";
    }

    try {
      await invoke("save_instruction_file", {
        workingDir: "",
        filename: finalFilename,
        content: editedContent,
      });

      // Clean up test session if exists
      if (testSession) {
        await invoke("stop_test_agent", {
          agentId: testSession.agentId,
          tempInstructionFile: testSession.tempInstructionFile,
        }).catch(() => {}); // Ignore errors
      }

      onSaved(finalFilename);
    } catch (e) {
      error = `Failed to save: ${e}`;
    }
  }

  function handleEditDraft() {
    goToStep(3);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onClose();
    }
  }

  // Step titles
  const stepTitles = [
    { title: "Describe Goal", icon: Sparkles },
    { title: "Review Draft", icon: FileText },
    { title: "Edit Draft", icon: FileText },
    { title: "Test", icon: Play },
  ];

  let isWideStep = $derived(currentStep >= 3);
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay animate-fade-in" onclick={onClose}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="dialog animate-slide-up"
    class:wide={isWideStep}
    onclick={(e) => e.stopPropagation()}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <!-- Progress dots -->
    <div class="progress-dots">
      {#each Array(TOTAL_STEPS) as _, i}
        <div
          class="dot"
          class:active={i + 1 === currentStep}
          class:completed={i + 1 < currentStep}
        ></div>
      {/each}
    </div>

    <header>
      <div class="header-content">
        <div class="dialog-icon">
          <Layers class="w-6 h-6 text-white" />
        </div>
        <div>
          <h2>Instruction Wizard</h2>
          <p class="subtitle">Step {currentStep}: {stepTitles[currentStep - 1]?.title}</p>
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
      {#if currentStep === 1}
        <!-- Step 1: Describe Goal -->
        <div class="step-content animate-slide-in">
          <div class="step-header-centered">
            <div class="step-icon">
              <Sparkles class="w-8 h-8" />
            </div>
            <h3>Describe Your Goal</h3>
            <p class="step-description">What do you want to set up or integrate?</p>
          </div>

          <GoalInput
            bind:goalDescription
            bind:context
            disabled={isGenerating}
          />
        </div>
      {:else if currentStep === 2}
        <!-- Step 2: Preview Draft -->
        <div class="step-content animate-slide-in">
          <div class="step-header-centered">
            <div class="step-icon">
              <FileText class="w-8 h-8" />
            </div>
            <h3>Review Draft</h3>
            <p class="step-description">AI-generated instruction based on your goal</p>
          </div>

          {#if draft}
            <DraftPreview
              {draft}
              bind:testPrompt
            />
          {/if}
        </div>
      {:else if currentStep === 3}
        <!-- Step 3: Edit Draft -->
        <div class="step-content animate-slide-in">
          <div class="step-header-centered">
            <div class="step-icon">
              <FileText class="w-8 h-8" />
            </div>
            <h3>{enhancementApplied ? "Enhanced Instruction" : "Edit Instruction"}</h3>
            <p class="step-description">
              {enhancementApplied
                ? "AI has updated the instruction based on test results"
                : "Refine the generated instruction"}
            </p>
          </div>

          {#if enhancementApplied}
            <div class="enhancement-badge">
              <Sparkles class="w-4 h-4" />
              <span>Enhanced with test findings</span>
            </div>
          {/if}

          <div class="edit-form">
            <label class="field-label">
              <span class="label-text">Filename</span>
              <input
                type="text"
                bind:value={filename}
                placeholder="my-instruction.md"
                class="input"
              />
            </label>

            <label class="field-label">
              <span class="label-text">Content</span>
              <DraftEditor
                bind:content={editedContent}
                placeholder="Enter instruction content..."
              />
            </label>

            <label class="field-label">
              <span class="label-text">Test Prompt</span>
              <input
                type="text"
                bind:value={testPrompt}
                placeholder="A prompt to test this instruction..."
                class="input"
              />
            </label>
          </div>
        </div>
      {:else if currentStep === 4}
        <!-- Step 4: Test Runner -->
        <div class="step-content animate-slide-in">
          <div class="step-header-centered">
            <div class="step-icon">
              <Play class="w-8 h-8" />
            </div>
            <h3>Testing Instruction</h3>
            <p class="step-description">Running a test agent to discover requirements</p>
          </div>

          {#if testSession}
            {#if isEnhancing}
              <div class="enhancing-state">
                <div class="enhancing-spinner"></div>
                <div class="enhancing-text">
                  <Sparkles class="w-5 h-5" />
                  <span>Enhancing instruction with test findings...</span>
                </div>
                <p class="step-description">
                  AI is analyzing the test execution and updating the instruction with concrete details
                </p>
              </div>
            {:else}
              <TestRunner
                session={testSession}
                onStop={handleStopTest}
                onAnalyze={handleAnalyzeResults}
              />
            {/if}
          {/if}
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
      {#if currentStep === 1}
        <button class="secondary" onclick={onClose}>
          Cancel
        </button>
        <button
          class="primary"
          onclick={handleGenerateDraft}
          disabled={!goalDescription.trim() || isGenerating}
        >
          {#if isGenerating}
            <span class="spinner"></span>
            Generating...
          {:else}
            <Sparkles class="w-4 h-4" />
            Generate Draft
          {/if}
        </button>
      {:else if currentStep === 2}
        <button class="secondary" onclick={() => goToStep(1)}>
          <ChevronLeft class="w-4 h-4" />
          Back
        </button>
        <button class="primary" onclick={handleEditDraft}>
          Continue
        </button>
      {:else if currentStep === 3}
        <button class="secondary" onclick={() => goToStep(2)}>
          <ChevronLeft class="w-4 h-4" />
          Back
        </button>
        <button
          class="secondary"
          onclick={handleSaveInstruction}
          disabled={!editedContent.trim() || !filename.trim()}
        >
          Save
        </button>
        <button
          class="primary"
          onclick={handleStartTest}
          disabled={!editedContent.trim()}
        >
          <Play class="w-4 h-4" />
          Test
        </button>
      {:else if currentStep === 4}
        <button class="secondary" onclick={() => goToStep(3)} disabled={isEnhancing}>
          <ChevronLeft class="w-4 h-4" />
          Back
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
    width: 600px;
    max-width: 100%;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    background-color: var(--bg-secondary);
    border-radius: 20px;
    border: 1px solid var(--border);
    box-shadow: var(--shadow-lg), 0 0 60px rgba(240, 112, 90, 0.1);
    overflow: hidden;
    transition: width 0.3s ease;
  }

  .dialog.wide {
    width: 900px;
  }

  .progress-dots {
    display: flex;
    justify-content: center;
    gap: 8px;
    padding: 16px;
    background: linear-gradient(180deg, var(--bg-tertiary) 0%, transparent 100%);
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--bg-elevated);
    transition: all 0.3s ease;
  }

  .dot.active {
    background: var(--accent-hex);
  }

  .dot.completed {
    background: var(--accent-hex);
    opacity: 0.6;
  }

  header {
    flex-shrink: 0;
    padding: var(--space-md) var(--space-lg);
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    border-bottom: 1px solid var(--border);
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
    background: linear-gradient(135deg, var(--accent-hex) 0%, #e85a45 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 4px 12px var(--accent-glow);
  }

  h2 {
    font-size: var(--text-xl);
    font-weight: 700;
    letter-spacing: -0.02em;
    margin-bottom: 2px;
  }

  h3 {
    font-size: var(--text-lg);
    font-weight: 600;
    letter-spacing: -0.01em;
    margin-bottom: 0;
  }

  .subtitle {
    font-size: 14px;
    color: var(--text-muted);
  }

  .close-btn {
    background: var(--bg-elevated);
    width: 44px;
    height: 44px;
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

  .step-content {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .step-header-centered {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: var(--space-sm);
    margin-bottom: var(--space-lg);
  }

  .step-icon {
    width: 56px;
    height: 56px;
    border-radius: 14px;
    background: rgba(232, 102, 77, 0.12);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--accent-hex);
    margin-bottom: var(--space-xs);
  }

  .step-description {
    font-size: var(--text-base);
    color: var(--text-secondary);
    max-width: 320px;
  }

  .edit-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .field-label {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }

  .label-text {
    font-size: 14px;
    font-weight: 500;
  }

  .input {
    width: 100%;
    height: 44px;
    padding: 0 14px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text-primary);
    font-size: var(--text-base);
    font-family: inherit;
  }

  .input:focus {
    outline: none;
    border-color: var(--accent-hex);
    box-shadow: 0 0 0 2px var(--accent-glow);
  }

  footer {
    flex-shrink: 0;
    padding: var(--space-md) var(--space-lg);
    display: flex;
    justify-content: flex-end;
    gap: var(--space-sm);
    border-top: 1px solid var(--border);
  }

  button {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 20px;
    min-height: 44px;
    border-radius: var(--radius-md);
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all var(--transition-fast);
    border: none;
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  button svg {
    width: 16px;
    height: 16px;
  }

  button.primary {
    background: linear-gradient(135deg, var(--accent-hex) 0%, #e85a45 100%);
    color: white;
    box-shadow: 0 2px 8px var(--accent-glow);
  }

  button.primary:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px var(--accent-glow);
  }

  button.secondary {
    background: var(--bg-elevated);
    color: var(--text-secondary);
  }

  button.secondary:hover:not(:disabled) {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .error {
    margin-top: var(--space-md);
    padding: var(--space-md);
    background-color: rgba(255, 59, 48, 0.1);
    border: 1px solid rgba(255, 59, 48, 0.3);
    border-radius: var(--radius-md);
    color: var(--error);
    font-size: var(--text-sm);
    display: flex;
    align-items: flex-start;
    gap: var(--space-sm);
  }

  .error svg {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
    margin-top: 1px;
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

  .animate-fade-in {
    animation: fadeIn 0.2s ease-out;
  }

  .animate-slide-up {
    animation: slideUp 0.3s ease-out;
  }

  .animate-slide-in {
    animation: slideIn 0.3s ease-out;
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  @keyframes slideUp {
    from {
      opacity: 0;
      transform: translateY(20px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateX(20px);
    }
    to {
      opacity: 1;
      transform: translateX(0);
    }
  }

  .enhancement-badge {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: linear-gradient(135deg, var(--accent-hex) 0%, #e85a45 100%);
    color: white;
    border-radius: var(--radius-md);
    font-size: 13px;
    font-weight: 500;
    width: fit-content;
  }

  .enhancing-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-md);
    padding: var(--space-xl);
    text-align: center;
  }

  .enhancing-spinner {
    width: 48px;
    height: 48px;
    border: 3px solid var(--border);
    border-top-color: var(--accent-hex);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  .enhancing-text {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: var(--text-lg);
    font-weight: 600;
    color: var(--text-primary);
  }

  .enhancing-text :global(svg) {
    color: var(--accent-hex);
  }
</style>
