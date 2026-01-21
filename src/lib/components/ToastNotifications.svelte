<script context="module" lang="ts">
  import { writable } from "svelte/store";

  export interface Toast {
    id: string;
    type: "info" | "success" | "warning" | "error";
    message: string;
    action?: { label: string; onClick: () => void };
    secondaryAction?: { label: string; onClick: () => void };
    duration?: number;
  }

  const toasts = writable<Toast[]>([]);
  const MAX_TOASTS = 3;

  export function showToast(toast: Omit<Toast, "id">) {
    const id = `toast-${Date.now()}-${Math.random()}`;
    const duration = toast.duration ?? 4000;

    toasts.update((current) => {
      const newToasts = [...current, { ...toast, id }];
      return newToasts.slice(-MAX_TOASTS);
    });

    if (duration > 0) {
      setTimeout(() => {
        removeToast(id);
      }, duration);
    }
  }

  export function removeToast(id: string) {
    toasts.update((current) => current.filter((t) => t.id !== id));
  }
</script>

<script lang="ts">

  function getToastColor(type: Toast["type"]): string {
    switch (type) {
      case "success":
        return "var(--success)";
      case "warning":
        return "var(--warning)";
      case "error":
        return "var(--error)";
      default:
        return "var(--accent)";
    }
  }

  function getToastIcon(type: Toast["type"]): string {
    switch (type) {
      case "success":
        return "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z";
      case "warning":
        return "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z";
      case "error":
        return "M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z";
      default:
        return "M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z";
    }
  }
</script>

<div class="toast-container">
  {#each $toasts as toast (toast.id)}
    <div class="toast {toast.type}" style="border-left-color: {getToastColor(toast.type)}">
      <div class="toast-content">
        <svg class="toast-icon" style="color: {getToastColor(toast.type)}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d={getToastIcon(toast.type)} stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
        <span class="toast-message">{toast.message}</span>
      </div>
      <div class="toast-actions">
        {#if toast.secondaryAction}
          <button class="toast-action-btn secondary" onclick={toast.secondaryAction.onClick}>
            {toast.secondaryAction.label}
          </button>
        {/if}
        {#if toast.action}
          <button class="toast-action-btn" onclick={toast.action.onClick}>
            {toast.action.label}
          </button>
        {/if}
        <button class="toast-close" onclick={() => removeToast(toast.id)} aria-label="Close notification">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M6 18L18 6M6 6l12 12" stroke-linecap="round"/>
          </svg>
        </button>
      </div>
    </div>
  {/each}
</div>

<style>
  .toast-container {
    position: fixed;
    top: var(--space-lg);
    right: var(--space-lg);
    z-index: 9999;
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
    max-width: 400px;
  }

  .toast {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-md);
    padding: var(--space-md);
    background-color: var(--bg-elevated);
    border: 1px solid var(--border);
    border-left: 4px solid;
    border-radius: 12px;
    box-shadow: var(--shadow-lg);
    animation: toast-slide-in 0.3s ease-out;
  }

  .toast-content {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    flex: 1;
  }

  .toast-icon {
    width: 24px;
    height: 24px;
    flex-shrink: 0;
  }

  .toast-message {
    font-size: 14px;
    color: var(--text-primary);
    line-height: 1.4;
  }

  .toast-actions {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .toast-action-btn {
    padding: 6px 12px;
    font-size: 13px;
    font-weight: 600;
    color: var(--accent);
    background-color: var(--accent-glow);
    border: 1px solid var(--accent);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .toast-action-btn:hover {
    background-color: var(--accent);
    color: white;
  }

  .toast-action-btn.secondary {
    background-color: transparent;
    border-color: var(--border);
    color: var(--text-secondary);
  }

  .toast-action-btn.secondary:hover {
    background-color: var(--bg-tertiary);
    border-color: var(--text-muted);
    color: var(--text-primary);
  }

  .toast-close {
    width: 32px;
    height: 32px;
    padding: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: transparent;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    color: var(--text-muted);
    transition: all 0.2s ease;
  }

  .toast-close:hover {
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .toast-close svg {
    width: 16px;
    height: 16px;
  }
</style>
