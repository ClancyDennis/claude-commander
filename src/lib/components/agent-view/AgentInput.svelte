<script lang="ts">
  import { MessageInput } from "$lib/components/ui/input";

  let { status, onSend }: { status: string; onSend: (text: string) => void } = $props();

  function handleSend(text: string) {
    onSend(text);
  }
</script>

<MessageInput
  placeholder="Type your prompt here..."
  disabled={status !== "running"}
  onSend={handleSend}
/>
