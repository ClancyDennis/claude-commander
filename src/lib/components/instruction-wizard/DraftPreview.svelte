<script lang="ts">
  import type { InstructionDraft } from "../../types";
  import { AlertCircle, CheckCircle, Clock } from "lucide-svelte";

  interface Props {
    draft: InstructionDraft;
    testPrompt: string;
  }

  let { draft, testPrompt = $bindable("") }: Props = $props();

  // Default to suggested test prompt
  $effect(() => {
    if (draft.suggestedTestPrompt && !testPrompt) {
      testPrompt = draft.suggestedTestPrompt;
    }
  });

  const complexityColors = {
    simple: "text-green-500 bg-green-500/10",
    moderate: "text-yellow-500 bg-yellow-500/10",
    complex: "text-red-500 bg-red-500/10",
  };

  const complexityIcons = {
    simple: CheckCircle,
    moderate: Clock,
    complex: AlertCircle,
  };
</script>

<div class="space-y-4">
  <!-- Metadata badges -->
  <div class="flex flex-wrap gap-2">
    <span class="text-xs px-2 py-1 rounded-md bg-muted text-muted-foreground">
      {draft.suggestedFilename}
    </span>
    <span class="text-xs px-2 py-1 rounded-md {complexityColors[draft.complexity] || complexityColors.moderate} flex items-center gap-1">
      <svelte:component
        this={complexityIcons[draft.complexity] || complexityIcons.moderate}
        class="w-3 h-3"
      />
      {draft.complexity}
    </span>
  </div>

  <!-- Draft content preview -->
  <div class="rounded-lg border border-border bg-muted/30 p-4 max-h-[300px] overflow-auto">
    <pre class="text-sm whitespace-pre-wrap font-mono">{draft.content}</pre>
  </div>

  <!-- Requirements -->
  {#if draft.potentialRequirements.length > 0}
    <div>
      <h4 class="text-sm font-medium mb-2">Potential Requirements</h4>
      <ul class="space-y-1">
        {#each draft.potentialRequirements as req}
          <li class="text-sm text-muted-foreground flex items-start gap-2">
            <AlertCircle class="w-4 h-4 mt-0.5 text-yellow-500 flex-shrink-0" />
            {req}
          </li>
        {/each}
      </ul>
    </div>
  {/if}

  <!-- Test prompt -->
  <div>
    <label for="test-prompt" class="text-sm font-medium mb-2 block">
      Test Prompt
    </label>
    <input
      id="test-prompt"
      type="text"
      bind:value={testPrompt}
      placeholder="A prompt to test this instruction..."
      class="w-full h-10 px-3 rounded-md border border-input bg-background text-sm focus:outline-none focus:ring-2 focus:ring-ring"
    />
    <p class="text-xs text-muted-foreground mt-1">
      This prompt will be sent to a test agent to verify the instruction works.
    </p>
  </div>
</div>

<style>
  input {
    font-family: inherit;
  }

  pre {
    tab-size: 2;
  }
</style>
