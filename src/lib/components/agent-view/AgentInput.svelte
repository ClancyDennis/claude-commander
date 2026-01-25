<script lang="ts">
  import { MessageInput } from "$lib/components/ui/input";
  import { pendingAgentPrompt } from "$lib/stores/agents";

  let { status, agentId, onSend }: { status: string; agentId?: string; onSend: (text: string) => void } = $props();

  // Compute pending prompt for this agent
  let pendingPrompt = $derived.by(() => {
    const pending = $pendingAgentPrompt;
    if (pending && agentId && pending.agentId === agentId) {
      // Clear the store after reading
      pendingAgentPrompt.set(null);
      return pending.prompt;
    }
    return null;
  });

  function handleSend(text: string) {
    onSend(text);
  }
</script>

<MessageInput
  placeholder="Type your prompt here..."
  disabled={status !== "running"}
  {pendingPrompt}
  onSend={handleSend}
/>
