<script lang="ts">
  import { getRiskClass, getRiskLabel } from "$lib/utils/security";
  import type { CommandRiskLevel } from "$lib/types";

  interface Props {
    riskLevel: CommandRiskLevel;
  }

  let { riskLevel }: Props = $props();
</script>

<div class="risk-badge {getRiskClass(riskLevel)}">
  {getRiskLabel(riskLevel)}
</div>

<style>
  .risk-badge {
    display: inline-flex;
    align-self: flex-start;
    padding: 4px 12px;
    border-radius: 4px;
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.05em;
  }

  .risk-badge.risk-normal {
    background-color: var(--success-bg, rgba(34, 197, 94, 0.1));
    color: var(--success);
    border: 1px solid var(--success);
  }

  .risk-badge.risk-suspicious {
    background-color: var(--warning-bg, rgba(234, 179, 8, 0.1));
    color: var(--warning);
    border: 1px solid var(--warning);
  }

  .risk-badge.risk-high {
    background-color: var(--error-bg, rgba(239, 68, 68, 0.1));
    color: var(--error);
    border: 1px solid var(--error);
  }
</style>
