<script lang="ts">
  import type { AgentRun } from "$lib/types";
  import { getStatusColorHex } from '$lib/utils/status';
  import { ViewHeader } from "$lib/components/ui/layout";
  import { History } from "$lib/components/ui/icons";

  interface Props {
    run: AgentRun;
  }

  let { run }: Props = $props();

  // Extract the directory name from the full path
  let directoryName = $derived(run.working_dir.split("/").pop() || run.working_dir);
</script>

<ViewHeader
  icon={History}
  title={directoryName}
  subtitle={run.working_dir}
>
  {#snippet status()}
    <span
      class="status-badge"
      style="background-color: {getStatusColorHex(run.status)}"
    >
      {run.status.toUpperCase()}
    </span>
  {/snippet}
</ViewHeader>

<style>
  .status-badge {
    padding: var(--space-1) var(--space-3);
    border-radius: var(--radius-sm);
    font-size: var(--text-xs);
    font-weight: var(--font-semibold);
    color: white;
    letter-spacing: 0.5px;
  }
</style>
