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
    width: 20px;
    height: 20px;
    flex: 0 0 auto;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 9999px;
    background: rgba(124, 58, 237, 0.16);
    border: 1px solid rgba(124, 58, 237, 0.35);
    color: var(--text-primary);
    font-size: 13px;
    font-weight: 700;
    line-height: 1;
    cursor: help;
    position: relative;
    user-select: none;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.25);
    opacity: 0.95;
    padding: 0;
  }

  .help-tip:hover {
    color: var(--text-primary);
    border-color: var(--accent);
    background: rgba(124, 58, 237, 0.22);
    opacity: 1;
  }

  .help-tip:focus-visible {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-glow);
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
    padding: 10px 12px;
    border-radius: 12px;
    box-shadow: var(--shadow-md);
    width: max-content;
    max-width: 320px;
    white-space: normal;
    z-index: 50;
  }

  .help-tip[data-placement="top"]::after,
  .help-tip:not([data-placement])::after {
    left: 50%;
    bottom: calc(100% + 10px);
    transform: translateX(-50%) translateY(4px);
  }

  .help-tip[data-placement="right"]::after {
    left: calc(100% + 10px);
    top: 50%;
    transform: translateX(-4px) translateY(-50%);
  }

  .help-tip[data-placement="bottom"]::after {
    left: 50%;
    top: calc(100% + 10px);
    transform: translateX(-50%) translateY(-4px);
  }

  .help-tip[data-placement="left"]::after {
    right: calc(100% + 10px);
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
