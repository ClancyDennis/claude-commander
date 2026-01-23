<script lang="ts">
  import type { ImageAttachment } from "$lib/types";
  import { X } from "$lib/components/ui/icons";

  let {
    image,
    onRemove,
    disabled = false,
  }: {
    image: ImageAttachment;
    onRemove: () => void;
    disabled?: boolean;
  } = $props();

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }
</script>

<div class="attachment-preview">
  <img src={image.previewUrl} alt={image.filename} class="preview-image" />
  <div class="attachment-info">
    <span class="filename" title={image.filename}>
      {image.filename.length > 20 ? image.filename.slice(0, 17) + "..." : image.filename}
    </span>
    <span class="size">{formatSize(image.sizeBytes)}</span>
  </div>
  {#if !disabled}
    <button
      class="remove-btn"
      onclick={onRemove}
      title="Remove image"
      aria-label="Remove image"
    >
      <X size={14} />
    </button>
  {/if}
</div>

<style>
  .attachment-preview {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    background: var(--accent-glow);
    border: 1px solid var(--accent-hex);
    border-radius: 8px;
    max-width: 300px;
  }

  .preview-image {
    width: 48px;
    height: 48px;
    object-fit: cover;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  .attachment-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .filename {
    font-size: 13px;
    color: var(--text-primary);
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .size {
    font-size: 11px;
    color: var(--text-secondary);
  }

  .remove-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    padding: 0;
    background: var(--error-glow);
    border: 1px solid var(--error);
    border-radius: 50%;
    color: var(--error);
    cursor: pointer;
    transition: all 0.2s ease;
    flex-shrink: 0;
  }

  .remove-btn:hover {
    background: rgba(224, 85, 85, 0.3);
    transform: scale(1.1);
  }

  .remove-btn:active {
    transform: scale(0.95);
  }
</style>
