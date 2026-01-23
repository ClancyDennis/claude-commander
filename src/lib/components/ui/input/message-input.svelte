<script lang="ts" module>
  import type { ImageAttachment } from "$lib/types";

  export type MessageInputProps = {
    placeholder?: string;
    disabled?: boolean;
    showSendButton?: boolean;
    enableImageAttachment?: boolean;
    maxHeight?: number;
    class?: string;
  };
</script>

<script lang="ts">
  import { onDestroy } from "svelte";
  import { cn } from "$lib/utils";
  import { Send } from "$lib/components/ui/icons";
  import ImageAttachmentPreview from "./image-attachment-preview.svelte";
  import {
    validateImageFile,
    createImageAttachment,
    revokePreviewUrl,
    getImageFileFromDrop,
    getImageFileFromPaste,
    SUPPORTED_TYPES,
  } from "./image-utils";
  import type { Snippet } from "svelte";

  let {
    placeholder = "Type a message...",
    disabled = false,
    showSendButton = true,
    enableImageAttachment = false,
    maxHeight = 200,
    class: className,
    onSend,
    prefix,
    suffix,
  }: MessageInputProps & {
    onSend: (text: string, image?: ImageAttachment | null) => void;
    prefix?: Snippet;
    suffix?: Snippet;
  } = $props();

  let input = $state("");
  let textarea: HTMLTextAreaElement | null = $state(null);
  let attachedImage = $state<ImageAttachment | null>(null);
  let isDragging = $state(false);
  let error = $state<string | null>(null);

  // Cleanup object URLs on destroy
  onDestroy(() => {
    revokePreviewUrl(attachedImage);
  });

  function adjustInputHeight() {
    if (textarea) {
      textarea.style.height = "auto";
      textarea.style.height = Math.min(textarea.scrollHeight, maxHeight) + "px";
    }
  }

  async function addImageFile(file: File) {
    if (!enableImageAttachment) return;

    const validation = validateImageFile(file);
    if (!validation.valid) {
      error = validation.error ?? "Invalid image";
      return;
    }

    try {
      // Revoke previous preview URL if any
      revokePreviewUrl(attachedImage);
      attachedImage = await createImageAttachment(file);
      error = null;
    } catch (e) {
      console.error("Failed to process image:", e);
      error = "Failed to process image";
    }
  }

  function handlePaste(event: ClipboardEvent) {
    if (!enableImageAttachment) return;

    const file = getImageFileFromPaste(event);
    if (file) {
      event.preventDefault();
      addImageFile(file);
    }
  }

  function handleDragOver(event: DragEvent) {
    if (!enableImageAttachment) return;
    event.preventDefault();
    isDragging = true;
  }

  function handleDragLeave(event: DragEvent) {
    if (!enableImageAttachment) return;
    event.preventDefault();
    isDragging = false;
  }

  function handleDrop(event: DragEvent) {
    if (!enableImageAttachment) return;
    event.preventDefault();
    isDragging = false;

    const file = getImageFileFromDrop(event);
    if (file) {
      addImageFile(file);
    } else {
      error = "No supported image file found. Use PNG, JPEG, GIF, or WebP.";
    }
  }

  function removeImage() {
    revokePreviewUrl(attachedImage);
    attachedImage = null;
  }

  function handleSend() {
    if ((!input.trim() && !attachedImage) || disabled) return;

    const messageText = input.trim();
    const imageToSend = attachedImage;

    // Reset input state
    input = "";
    if (textarea) textarea.style.height = "auto";
    error = null;

    // Clear attached image reference (URL will be used by parent)
    attachedImage = null;

    // Call parent send handler
    onSend(messageText, imageToSend);
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      handleSend();
    }
  }

  $effect(() => {
    if (input !== undefined) adjustInputHeight();
  });

  const canSend = $derived(!disabled && (!!input.trim() || !!attachedImage));
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class={cn("input-area", className)}
  class:dragging={isDragging && enableImageAttachment}
  ondragover={handleDragOver}
  ondragleave={handleDragLeave}
  ondrop={handleDrop}
  role="region"
  aria-label="Message input area"
>
  {#if error}
    <div class="error-banner">
      {error}
    </div>
  {/if}

  {#if prefix}
    {@render prefix()}
  {/if}

  {#if enableImageAttachment && attachedImage}
    <div class="attachment-area">
      <ImageAttachmentPreview
        image={attachedImage}
        onRemove={removeImage}
        {disabled}
      />
    </div>
  {/if}

  <div class="input-wrapper">
    <textarea
      bind:this={textarea}
      bind:value={input}
      onkeydown={handleKeydown}
      oninput={adjustInputHeight}
      onpaste={handlePaste}
      {placeholder}
      {disabled}
      rows="1"
    ></textarea>

    {#if suffix}
      {@render suffix()}
    {/if}

    {#if showSendButton}
      <button
        onclick={handleSend}
        disabled={!canSend}
        class="send-btn"
      >
        <Send size={20} />
        <span class="send-label">Send</span>
      </button>
    {/if}
  </div>

  {#if enableImageAttachment && isDragging}
    <div class="drop-overlay">
      <div class="drop-message">Drop image here</div>
    </div>
  {/if}
</div>

<style>
  .input-area {
    padding: var(--space-4);
    background: var(--bg-secondary);
    border-top: 1px solid var(--border-hex);
    position: relative;
    transition: all var(--transition-fast);
  }

  .input-area.dragging {
    background: rgba(232, 102, 77, 0.08);
    border-top-color: var(--accent-hex);
  }

  .attachment-area {
    margin-bottom: var(--space-3);
  }

  .drop-overlay {
    position: absolute;
    inset: 0;
    background: rgba(232, 102, 77, 0.15);
    border: 2px dashed var(--accent-hex);
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10;
    pointer-events: none;
  }

  .drop-message {
    font-size: var(--text-base);
    font-weight: var(--font-semibold);
    color: var(--accent-hex);
    padding: var(--space-3) var(--space-5);
    background: var(--bg-primary);
    border-radius: var(--radius-md);
  }

  .error-banner {
    background: rgba(255, 59, 48, 0.1);
    border: 1px solid var(--error);
    color: var(--error);
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-md);
    font-size: var(--text-sm);
    margin-bottom: var(--space-3);
  }

  .input-wrapper {
    display: flex;
    gap: var(--space-3);
    align-items: flex-end;
  }

  textarea {
    flex: 1;
    min-width: 0;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-hex);
    border-radius: var(--radius-lg);
    padding: var(--space-3) var(--space-4);
    color: var(--text-primary);
    font-size: var(--text-base);
    font-family: inherit;
    resize: none;
    min-height: 44px;
    max-height: 180px;
    line-height: var(--leading-normal);
    transition: border-color var(--transition-fast), box-shadow var(--transition-fast);
  }

  textarea:focus {
    outline: none;
    border-color: var(--accent-hex);
    box-shadow: 0 0 0 3px rgba(232, 102, 77, 0.15);
  }

  textarea:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  textarea::placeholder {
    color: var(--text-muted);
  }

  .send-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-4);
    background: var(--accent-hex);
    border: none;
    border-radius: var(--radius-md);
    color: white;
    font-size: var(--text-sm);
    font-weight: var(--font-semibold);
    cursor: pointer;
    transition: all var(--transition-fast);
    min-width: 44px;
    height: 44px;
    flex-shrink: 0;
  }

  .send-btn:hover:not(:disabled) {
    filter: brightness(1.1);
    transform: translateY(-1px);
  }

  .send-btn:active:not(:disabled) {
    transform: translateY(0);
  }

  .send-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .send-label {
    display: none;
  }

  @media (min-width: 640px) {
    .send-label {
      display: inline;
    }

    .send-btn {
      padding: var(--space-3) var(--space-5);
    }
  }
</style>
