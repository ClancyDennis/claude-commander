<script lang="ts">
  import type { ModelConfig } from "$lib/types";
  import ModelItem from "./ModelItem.svelte";

  let {
    models,
    isEditing,
    editedModels,
    availableModels,
    onModelChange,
  }: {
    models: ModelConfig[];
    isEditing: boolean;
    editedModels: Record<string, string>;
    availableModels: string[];
    onModelChange: (modelName: string, value: string) => void;
  } = $props();

  function getModelDescription(name: string): string {
    const descriptions: Record<string, string> = {
      "ANTHROPIC_MODEL": "Primary model for agent tasks and conversations",
      "SECURITY_MODEL": "Model for security-critical analysis (higher capability recommended)",
      "LIGHT_TASK_MODEL": "Model for lightweight tasks like title generation (faster/cheaper)",
      "OPENAI_MODEL": "OpenAI model for tasks using GPT (requires OpenAI API key)"
    };
    return descriptions[name] || "";
  }
</script>

<section class="config-section">
  <h3>Model Configuration</h3>
  <div class="models-list">
    {#each models as model}
      <ModelItem
        {model}
        {isEditing}
        {availableModels}
        editedValue={editedModels[model.name] || ""}
        description={getModelDescription(model.name)}
        onModelChange={(value) => onModelChange(model.name, value)}
      />
    {/each}
  </div>
</section>

<style>
  .config-section {
    background: var(--bg-secondary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-lg);
    padding: var(--space-6);
    margin-bottom: var(--space-6);
  }

  .config-section h3 {
    margin: 0 0 var(--space-4) 0;
    font-size: var(--text-lg);
    font-weight: var(--font-semibold);
    color: var(--text-primary);
  }

  .models-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }
</style>
