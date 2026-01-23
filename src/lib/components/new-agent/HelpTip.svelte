<script lang="ts">
  let {
    text,
    placement = "top",
    className = "",
  }: {
    text: string;
    placement?: "top" | "right" | "bottom" | "left";
    className?: string;
  } = $props();

  function swallow(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
  }
</script>

<button
  type="button"
  class={"help-tip " + className}
  data-tip={text}
  data-placement={placement}
  aria-label={text}
  title={text}
  onclick={swallow}
  onmousedown={swallow}
>
  ?
</button>

<style>
  .help-tip {
    width: 16px;
    height: 16px;
    flex: 0 0 auto;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-full);
    background: rgba(255, 255, 255, 0.08);
    border: none;
    color: var(--text-muted);
    font-size: 10px;
    font-weight: var(--font-semibold);
    line-height: 1;
    cursor: help;
    position: relative;
    user-select: none;
    padding: 0;
    transition: all var(--transition-fast);
  }

  .help-tip:hover {
    color: var(--text-primary);
    background: rgba(255, 255, 255, 0.12);
  }

  .help-tip:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px rgba(232, 102, 77, 0.3);
  }

  .help-tip::after {
    content: attr(data-tip);
    position: absolute;
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.15s ease, transform 0.15s ease;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    color: var(--text-primary);
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-md);
    font-size: var(--text-xs);
    font-weight: var(--font-normal);
    width: max-content;
    max-width: 260px;
    white-space: normal;
    line-height: 1.4;
    z-index: 50;
  }

  .help-tip[data-placement="top"]::after,
  .help-tip:not([data-placement])::after {
    left: 50%;
    bottom: calc(100% + 8px);
    transform: translateX(-50%) translateY(4px);
  }

  .help-tip[data-placement="right"]::after {
    left: calc(100% + 8px);
    top: 50%;
    transform: translateX(-4px) translateY(-50%);
  }

  .help-tip[data-placement="bottom"]::after {
    left: 50%;
    top: calc(100% + 8px);
    transform: translateX(-50%) translateY(-4px);
  }

  .help-tip[data-placement="left"]::after {
    right: calc(100% + 8px);
    top: 50%;
    left: auto;
    transform: translateX(4px) translateY(-50%);
  }

  .help-tip:hover::after,
  .help-tip:focus::after {
    opacity: 1;
  }

  .help-tip[data-placement="top"]:hover::after,
  .help-tip[data-placement="top"]:focus::after,
  .help-tip:not([data-placement]):hover::after,
  .help-tip:not([data-placement]):focus::after {
    transform: translateX(-50%) translateY(0);
  }

  .help-tip[data-placement="right"]:hover::after,
  .help-tip[data-placement="right"]:focus::after {
    transform: translateX(0) translateY(-50%);
  }

  .help-tip[data-placement="bottom"]:hover::after,
  .help-tip[data-placement="bottom"]:focus::after {
    transform: translateX(-50%) translateY(0);
  }

  .help-tip[data-placement="left"]:hover::after,
  .help-tip[data-placement="left"]:focus::after {
    transform: translateX(0) translateY(-50%);
  }
</style>
