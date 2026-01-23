<script lang="ts">
  import type { AutoPipeline } from '$lib/types';
  import { ViewHeader } from "$lib/components/ui/layout";
  import { CheckCircle } from "$lib/components/ui/icons";

  let { pipeline }: { pipeline: AutoPipeline } = $props();

  function getStatusClass(status: string): string {
    switch (status) {
      case 'Completed': return 'status-completed';
      case 'Running': return 'status-running';
      case 'Failed': return 'status-failed';
      default: return 'status-default';
    }
  }
</script>

<ViewHeader
  icon={CheckCircle}
  title={pipeline.user_request}
  subtitle={pipeline.working_dir}
>
  {#snippet status()}
    <div class="status-badge {getStatusClass(pipeline.status)}">
      {pipeline.status}
    </div>
  {/snippet}
</ViewHeader>

<style>
  .status-badge {
    font-size: 11px;
    font-weight: 600;
    padding: 4px 12px;
    border-radius: 6px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .status-completed {
    color: #fff;
    background: var(--success-hex);
    box-shadow: 0 0 10px var(--success-glow);
  }

  .status-running {
    color: #fff;
    background: #3b82f6;
    box-shadow: 0 0 10px rgba(59, 130, 246, 0.4);
  }

  .status-failed {
    color: #fff;
    background: var(--error);
    box-shadow: 0 0 10px var(--error-glow);
  }

  .status-default {
    color: var(--text-muted);
    background: var(--bg-tertiary);
  }
</style>
