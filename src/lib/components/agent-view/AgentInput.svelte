<script lang="ts">
  import { MessageInput } from "$lib/components/ui/input";
  import { pendingAgentPrompt } from "$lib/stores/agents";

  let { status, agentId, onSend }: { status: string; agentId?: string; onSend: (text: string) => void } = $props();

  // Local state to hold the prompt (cleared after consumed)
  let pendingPrompt = $state<string | null>(null);

  // Effect to consume pending prompt from store (side effects belong in $effect, not $derived)
  $effect(() => {
    const pending = $pendingAgentPrompt;
    if (pending && agentId && pending.agentId === agentId) {
      pendingPrompt = pending.prompt;
      // Clear the store after consuming
      pendingAgentPrompt.set(null);
    }
  });

  function handleSend(text: string) {
    // Clear local pending prompt after send
    pendingPrompt = null;
    onSend(text);
  }
</script>

<MessageInput
  placeholder="Type your prompt here..."
  disabled={status !== "running"}
  {pendingPrompt}
  onSend={handleSend}
/>
