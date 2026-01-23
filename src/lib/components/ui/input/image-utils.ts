import type { ImageAttachment } from "$lib/types";

// Constants
export const MAX_IMAGE_SIZE = 20 * 1024 * 1024; // 20MB
export const SUPPORTED_TYPES = ["image/png", "image/jpeg", "image/gif", "image/webp"];

/**
 * Convert a File to base64 string
 */
export function fileToBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const result = reader.result as string;
      // Remove the data URL prefix (e.g., "data:image/png;base64,")
      const base64 = result.split(",")[1];
      resolve(base64);
    };
    reader.onerror = reject;
    reader.readAsDataURL(file);
  });
}

/**
 * Validate an image file for size and type
 */
export function validateImageFile(file: File): { valid: boolean; error?: string } {
  if (file.size > MAX_IMAGE_SIZE) {
    return { valid: false, error: `Image too large: ${file.name} (max 20MB)` };
  }

  if (!SUPPORTED_TYPES.includes(file.type)) {
    return { valid: false, error: `Unsupported image format. Use PNG, JPEG, GIF, or WebP.` };
  }

  return { valid: true };
}

/**
 * Create an ImageAttachment from a File
 */
export async function createImageAttachment(file: File): Promise<ImageAttachment> {
  const base64Data = await fileToBase64(file);

  return {
    id: crypto.randomUUID(),
    filename: file.name || "pasted-image",
    mimeType: file.type as ImageAttachment["mimeType"],
    base64Data,
    previewUrl: URL.createObjectURL(file),
    sizeBytes: file.size,
  };
}

/**
 * Revoke a preview URL to free memory
 */
export function revokePreviewUrl(image: ImageAttachment | null): void {
  if (image?.previewUrl) {
    URL.revokeObjectURL(image.previewUrl);
  }
}

/**
 * Extract first image file from a drop event
 */
export function getImageFileFromDrop(event: DragEvent): File | null {
  const files = event.dataTransfer?.files;
  if (!files || files.length === 0) return null;

  for (const file of files) {
    if (SUPPORTED_TYPES.includes(file.type)) {
      return file;
    }
  }

  return null;
}

/**
 * Extract image file from a paste event
 */
export function getImageFileFromPaste(event: ClipboardEvent): File | null {
  const items = event.clipboardData?.items;
  if (!items) return null;

  for (const item of items) {
    if (item.type.startsWith("image/")) {
      return item.getAsFile();
    }
  }

  return null;
}
