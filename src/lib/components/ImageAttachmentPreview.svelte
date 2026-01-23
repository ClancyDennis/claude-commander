<script lang="ts">
  import type { ImageAttachment } from "../types";

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
      <svg
        width="14"
        height="14"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <line x1="18" y1="6" x2="6" y2="18"></line>
        <line x1="6" y1="6" x2="18" y2="18"></line>
      </svg>
    </button>
  {/if}
</div>

<style>
  .attachment-preview {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    background: rgba(240, 112, 90, 0.1);
    border: 1px solid rgba(240, 112, 90, 0.3);
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
    color: #e0e0e0;
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .size {
    font-size: 11px;
    color: #999;
  }

  .remove-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    padding: 0;
    background: rgba(239, 68, 68, 0.2);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: 50%;
    color: #ef4444;
    cursor: pointer;
    transition: all 0.2s ease;
    flex-shrink: 0;
  }

  .remove-btn:hover {
    background: rgba(239, 68, 68, 0.3);
    border-color: rgba(239, 68, 68, 0.5);
    transform: scale(1.1);
  }

  .remove-btn:active {
    transform: scale(0.95);
  }
</style>
