<script lang="ts">
  interface Props {
    goalDescription: string;
    context: string;
    disabled?: boolean;
  }

  let {
    goalDescription = $bindable(""),
    context = $bindable(""),
    disabled = false,
  }: Props = $props();

  const examples = [
    "I want to connect to Google Drive",
    "I want to use the GitHub API to manage issues",
    "I want to scrape data from websites",
    "I want to send emails via Gmail",
  ];

  function useExample(example: string) {
    goalDescription = example;
  }
</script>

<div class="space-y-4">
  <div>
    <label for="goal" class="text-sm font-medium mb-2 block">
      What do you want to accomplish?
    </label>
    <textarea
      id="goal"
      bind:value={goalDescription}
      {disabled}
      placeholder="Describe your goal in plain language...

Example: 'I want to connect to Google Drive and list my files'"
      class="w-full min-h-[120px] p-3 rounded-md border border-input bg-background text-sm resize-none focus:outline-none focus:ring-2 focus:ring-ring"
    ></textarea>
  </div>

  <div class="flex flex-wrap gap-2">
    <span class="text-xs text-muted-foreground">Try:</span>
    {#each examples as example}
      <button
        type="button"
        onclick={() => useExample(example)}
        class="text-sm px-3 py-2.5 rounded-lg bg-muted hover:bg-muted/80 text-muted-foreground hover:text-foreground transition-colors"
        {disabled}
      >
        {example}
      </button>
    {/each}
  </div>

  <div>
    <label for="context" class="text-sm font-medium mb-2 block">
      Additional context <span class="text-muted-foreground">(optional)</span>
    </label>
    <input
      id="context"
      type="text"
      bind:value={context}
      {disabled}
      placeholder="Any specific requirements or constraints..."
      class="w-full h-10 px-3 rounded-md border border-input bg-background text-sm focus:outline-none focus:ring-2 focus:ring-ring"
    />
  </div>
</div>

<style>
  textarea,
  input {
    font-family: inherit;
  }
</style>
