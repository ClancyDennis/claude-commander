<script lang="ts">
  import type { ImageAttachment } from "$lib/types";
  import { MessageInput } from "$lib/components/ui/input";

  interface Props {
    disabled: boolean;
    hasApiKey: boolean;
    onSend: (message: string, image: ImageAttachment | null) => void;
  }

  let { disabled, hasApiKey, onSend }: Props = $props();

  function handleSend(text: string, image?: ImageAttachment | null) {
    onSend(text, image ?? null);
  }
</script>

<MessageInput
  enableImageAttachment={true}
  placeholder={!hasApiKey ? "API key required..." : "Type a message... (Paste or drop images, Enter to send)"}
  disabled={disabled || !hasApiKey}
  onSend={handleSend}
/>
