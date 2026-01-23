<script lang="ts">
  import { onDestroy } from "svelte";
  import type { ImageAttachment } from "../../types";
  import ImageAttachmentPreview from "../ImageAttachmentPreview.svelte";
  import {
    validateImageFile,
    createImageAttachment,
    revokePreviewUrl,
    getImageFileFromDrop,
    getImageFileFromPaste,
    SUPPORTED_TYPES,
  } from "./image-utils";

  interface Props {
    disabled: boolean;
    hasApiKey: boolean;
    onSend: (message: string, image: ImageAttachment | null) => void;
  }

  let { disabled, hasApiKey, onSend }: Props = $props();

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
      textarea.style.height = Math.min(textarea.scrollHeight, 200) + "px";
    }
  }

  async function addImageFile(file: File) {
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
    const file = getImageFileFromPaste(event);
    if (file) {
      event.preventDefault();
      addImageFile(file);
    }
  }

  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    isDragging = true;
  }

  function handleDragLeave(event: DragEvent) {
    event.preventDefault();
    isDragging = false;
  }

  function handleDrop(event: DragEvent) {
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
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="input-area"
  class:dragging={isDragging}
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
  {#if attachedImage}
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
      placeholder={!hasApiKey ? "API key required..." : "Type a message... (Paste or drop images, Enter to send)"}
      disabled={disabled || !hasApiKey}
      rows="1"
    ></textarea>
    <button
      onclick={handleSend}
      disabled={(!input.trim() && !attachedImage) || disabled || !hasApiKey}
      class="send-btn"
    >
      Send
    </button>
  </div>
  {#if isDragging}
    <div class="drop-overlay">
      <div class="drop-message">Drop image here</div>
    </div>
  {/if}
</div>

<style>
  .input-area {
    padding: 16px 20px;
    background: #1a1a1f;
    border-top: 1px solid rgba(124, 58, 237, 0.2);
    position: relative;
    transition: all 0.2s ease;
  }

  .input-area.dragging {
    background: rgba(124, 58, 237, 0.1);
    border-top-color: rgba(124, 58, 237, 0.5);
  }

  .attachment-area {
    margin-bottom: 12px;
  }

  .drop-overlay {
    position: absolute;
    inset: 0;
    background: rgba(124, 58, 237, 0.2);
    border: 2px dashed #7c3aed;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10;
    pointer-events: none;
  }

  .drop-message {
    font-size: 16px;
    font-weight: 600;
    color: #7c3aed;
    padding: 12px 24px;
    background: rgba(15, 15, 19, 0.9);
    border-radius: 8px;
  }

  .error-banner {
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    color: #ef4444;
    padding: 10px 14px;
    border-radius: 6px;
    font-size: 13px;
    margin-bottom: 12px;
  }

  .input-wrapper {
    display: flex;
    gap: 12px;
    align-items: flex-end;
  }

  textarea {
    flex: 1;
    background: #0f0f13;
    border: 1px solid rgba(124, 58, 237, 0.3);
    border-radius: 8px;
    padding: 12px 14px;
    color: #e0e0e0;
    font-size: 14px;
    font-family: inherit;
    resize: none;
    transition: border-color 0.2s ease;
    min-height: 44px;
    max-height: 200px;
    line-height: 1.5;
  }

  textarea:focus {
    outline: none;
    border-color: #7c3aed;
  }

  textarea:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  textarea::placeholder {
    color: #666;
  }

  .send-btn {
    padding: 12px 24px;
    background: linear-gradient(135deg, #7c3aed 0%, #6d28d9 100%);
    border: none;
    border-radius: 8px;
    color: white;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
    min-width: 80px;
    height: 44px;
  }

  .send-btn:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(124, 58, 237, 0.4);
  }

  .send-btn:active:not(:disabled) {
    transform: translateY(0);
  }

  .send-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
