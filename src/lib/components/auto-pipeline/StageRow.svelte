<script lang="ts">
  import type { AutoPipelineStep } from '$lib/types';

  let {
    steps,
    orchestratorActiveStage
  }: {
    steps: AutoPipelineStep[];
    orchestratorActiveStage: number | null;
  } = $props();

  function getRoleIcon(role: string): string {
    switch (role) {
      case 'Orchestrator': return '🎛️';
      case 'Planning': return '📋';
      case 'Building': return '🔨';
      case 'Verifying': return '✅';
      default: return '⚙️';
    }
  }
</script>

<div class="stages-row">
  {#each steps as step, i}
    {@const isOrchestratorDriven = orchestratorActiveStage === i}
    {@const isStepActive = step.status === 'Running' || isOrchestratorDriven}

    <div class="stage"
         class:completed={step.status === 'Completed'}
         class:active={isStepActive}
         class:orchestrator-active={isOrchestratorDriven}
         class:pending={!isStepActive && step.status !== 'Completed'}>
      <div class="stage-icon-wrapper"
           class:completed={step.status === 'Completed'}
           class:active={isStepActive}
           class:orchestrator-active={isOrchestratorDriven}>
        {#if step.status === 'Completed'}
          <svg class="check-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
            <polyline points="20 6 9 17 4 12"/>
          </svg>
        {:else if step.status === 'Running'}
          <span class="stage-emoji">{getRoleIcon(step.role)}</span>
        {:else}
          <span class="stage-emoji dimmed">{getRoleIcon(step.role)}</span>
        {/if}
      </div>
      <span class="stage-name">{step.role}</span>
    </div>
    {#if i < steps.length - 1}
      <div class="stage-connector" class:filled={step.status === 'Completed'}></div>
    {/if}
  {/each}
</div>

<style>
  .stages-row {
    display: flex;
    align-items: center;
    flex: 1;
    justify-content: center;
  }

  .stage {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    min-width: 80px;
  }

  .stage-icon-wrapper {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-tertiary);
    border: 2px solid var(--border);
    transition: all 0.3s ease;
  }

  .stage-icon-wrapper.completed {
    background: var(--success);
    border-color: var(--success);
  }

  .stage-icon-wrapper.active {
    background: var(--accent);
    border-color: var(--accent);
    box-shadow: 0 0 20px var(--accent-glow);
    animation: pulseGlow 2s ease-in-out infinite;
  }

  @keyframes pulseGlow {
    0%, 100% { box-shadow: 0 0 20px var(--accent-glow); }
    50% { box-shadow: 0 0 30px var(--accent-glow), 0 0 40px var(--accent-glow); }
  }

  /* Orchestrator-driven stage animation (different style) */
  .stage-icon-wrapper.orchestrator-active {
    background: var(--accent);
    border-color: var(--accent);
    box-shadow: 0 0 20px var(--accent-glow);
    animation: pulseGlowOrchestrator 1.5s ease-in-out infinite;
  }

  @keyframes pulseGlowOrchestrator {
    0%, 100% {
      box-shadow: 0 0 20px var(--accent-glow);
      transform: scale(1);
    }
    50% {
      box-shadow: 0 0 35px var(--accent-glow), 0 0 45px var(--accent-glow);
      transform: scale(1.05);
    }
  }

  .stage-emoji {
    font-size: 18px;
  }

  .stage-emoji.dimmed {
    opacity: 0.5;
  }

  .check-icon {
    width: 20px;
    height: 20px;
    color: white;
  }

  .stage-name {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .stage.pending .stage-name {
    color: var(--text-muted);
  }

  .stage-connector {
    width: 40px;
    height: 2px;
    background: var(--border);
    margin: 0 var(--space-xs);
    margin-bottom: 20px; /* Align with icon center */
    position: relative;
  }

  .stage-connector::after {
    content: '';
    position: absolute;
    left: 0;
    top: 0;
    height: 100%;
    width: 0;
    background: linear-gradient(90deg, var(--success), var(--accent));
    transition: width 0.5s ease;
  }

  .stage-connector.filled::after {
    width: 100%;
  }
</style>
