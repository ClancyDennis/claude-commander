<script lang="ts">
  import AgentView from "./AgentView.svelte";
  import { selectedAgentIds } from "../stores/agents";

  let { direction = "horizontal" }: { direction?: "horizontal" | "vertical" } = $props();

  let splitRatio = $state(50); // Percentage for first panel
  let isDragging = $state(false);
  let containerRef: HTMLDivElement | null = $state(null);

  const agentArray = $derived(Array.from($selectedAgentIds));
  const firstAgentId = $derived(agentArray[0] || null);
  const secondAgentId = $derived(agentArray[1] || null);

  function handleMouseDown(e: MouseEvent) {
    isDragging = true;
    e.preventDefault();
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isDragging || !containerRef) return;

    const rect = containerRef.getBoundingClientRect();
    let newRatio: number;

    if (direction === "horizontal") {
      const x = e.clientX - rect.left;
      newRatio = (x / rect.width) * 100;
    } else {
      const y = e.clientY - rect.top;
      newRatio = (y / rect.height) * 100;
    }

    // Constrain between 30% and 70%
    splitRatio = Math.max(30, Math.min(70, newRatio));
  }

  function handleMouseUp() {
    isDragging = false;
  }

  $effect(() => {
    if (isDragging) {
      window.addEventListener("mousemove", handleMouseMove);
      window.addEventListener("mouseup", handleMouseUp);

      return () => {
        window.removeEventListener("mousemove", handleMouseMove);
        window.removeEventListener("mouseup", handleMouseUp);
      };
    }
  });

  function swapPanels() {
    if (firstAgentId && secondAgentId) {
      selectedAgentIds.update(() => new Set([secondAgentId, firstAgentId]));
    }
  }
</script>

<div class="split-view {direction}" class:dragging={isDragging} bind:this={containerRef}>
  <div
    class="panel first-panel"
    style="{direction === 'horizontal' ? 'width' : 'height'}: {splitRatio}%"
  >
    {#if firstAgentId}
      <AgentView agentId={firstAgentId} />
    {:else}
      <div class="empty-panel">
        <div class="empty-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="3" y="3" width="18" height="18" rx="2"/>
            <path d="M12 8v8M8 12h8"/>
          </svg>
        </div>
        <p>Select a second agent to split view</p>
      </div>
    {/if}
  </div>

  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="divider {direction}"
    onmousedown={handleMouseDown}
    role="separator"
    tabindex="0"
    aria-label="Draggable splitter"
  >
    <div class="divider-handle">
      {#if direction === "horizontal"}
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M9 18l6-6-6-6"/>
        </svg>
      {:else}
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M18 9l-6 6-6-6"/>
        </svg>
      {/if}
    </div>
    {#if firstAgentId && secondAgentId}
      <button class="swap-btn" onclick={swapPanels} title="Swap panels">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M7 16V4M7 4L3 8M7 4l4 4M17 8v12m0 0l4-4m-4 4l-4-4"/>
        </svg>
      </button>
    {/if}
  </div>

  <div class="panel second-panel">
    {#if secondAgentId}
      <AgentView agentId={secondAgentId} />
    {:else}
      <div class="empty-panel">
        <div class="empty-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="3" y="3" width="18" height="18" rx="2"/>
            <path d="M12 8v8M8 12h8"/>
          </svg>
        </div>
        <p>Select an agent from the sidebar</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .split-view {
    display: flex;
    width: 100%;
    height: 100%;
    position: relative;
  }

  .split-view.horizontal {
    flex-direction: row;
  }

  .split-view.vertical {
    flex-direction: column;
  }

  .split-view.dragging {
    user-select: none;
  }

  .panel {
    overflow: hidden;
    position: relative;
  }

  .first-panel {
    flex-shrink: 0;
  }

  .second-panel {
    flex: 1;
    min-width: 0;
  }

  .divider {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    transition: background-color 0.2s ease;
    z-index: 10;
  }

  .divider.horizontal {
    width: 8px;
    cursor: col-resize;
    border-left: 1px solid var(--border);
    border-right: 1px solid var(--border);
  }

  .divider.vertical {
    height: 8px;
    cursor: row-resize;
    border-top: 1px solid var(--border);
    border-bottom: 1px solid var(--border);
  }

  .divider:hover {
    background-color: var(--accent-glow);
  }

  .divider:active {
    background-color: var(--accent);
  }

  .divider-handle {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    transition: color 0.2s ease;
  }

  .divider:hover .divider-handle {
    color: var(--accent);
  }

  .divider-handle svg {
    width: 16px;
    height: 16px;
  }

  .swap-btn {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 8px;
    cursor: pointer;
    color: var(--text-muted);
    opacity: 0;
    transition: all 0.2s ease;
  }

  .divider:hover .swap-btn {
    opacity: 1;
  }

  .swap-btn:hover {
    background-color: var(--accent-glow);
    border-color: var(--accent);
    color: var(--accent);
  }

  .swap-btn svg {
    width: 16px;
    height: 16px;
  }

  .empty-panel {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-xl);
    text-align: center;
    color: var(--text-muted);
    background-color: var(--bg-primary);
  }

  .empty-icon {
    width: 80px;
    height: 80px;
    border-radius: 24px;
    background: linear-gradient(135deg, var(--bg-secondary) 0%, var(--bg-tertiary) 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: var(--space-lg);
    border: 1px solid var(--border);
  }

  .empty-icon svg {
    width: 40px;
    height: 40px;
    color: var(--text-muted);
  }

  .empty-panel p {
    font-size: 14px;
    color: var(--text-muted);
  }
</style>
