<script lang="ts">
  import type { TestAnalysisResult, FindingType } from "../../types";
  import { Button } from "$lib/components/ui/button";
  import {
    AlertTriangle,
    AlertCircle,
    Info,
    CheckCircle,
    RefreshCw,
    Pencil,
    Save,
    Terminal,
    Key,
    Shield,
    Settings,
    HelpCircle,
    Sparkles,
  } from "lucide-svelte";
  import { WizardHeader, WizardNavigation } from "$lib/components/wizard";

  interface Props {
    results: TestAnalysisResult;
    onEditDraft: () => void;
    onRunAgain: () => void;
    onSave: () => void;
  }

  let { results, onEditDraft, onRunAgain, onSave }: Props = $props();

  // Group findings by severity
  let criticalFindings = $derived(
    results.findings.filter((f) => f.severity === "critical")
  );
  let warningFindings = $derived(
    results.findings.filter((f) => f.severity === "warning")
  );
  let infoFindings = $derived(
    results.findings.filter((f) => f.severity === "info")
  );

  function formatDuration(ms: number): string {
    const seconds = Math.floor(ms / 1000);
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    if (mins > 0) {
      return `${mins}m ${secs}s`;
    }
    return `${secs}s`;
  }

  function getStatusInfo(status: string): { icon: typeof CheckCircle; label: string; class: string } {
    switch (status) {
      case "completed":
        return { icon: CheckCircle, label: "Completed", class: "text-green-500" };
      case "failed":
        return { icon: AlertCircle, label: "Failed", class: "text-destructive" };
      case "timeout":
        return { icon: AlertTriangle, label: "Timed Out", class: "text-yellow-500" };
      default:
        return { icon: Info, label: status, class: "text-muted-foreground" };
    }
  }

  function getFindingIcon(type: FindingType): typeof Terminal {
    switch (type) {
      case "MissingTool":
        return Terminal;
      case "AuthRequired":
        return Key;
      case "PermissionDenied":
        return Shield;
      case "EnvironmentSetup":
        return Settings;
      case "InstructionAmbiguity":
        return HelpCircle;
      case "SuccessPattern":
        return Sparkles;
      default:
        return Info;
    }
  }

  function getSeverityIcon(severity: string): typeof AlertCircle {
    switch (severity) {
      case "critical":
        return AlertCircle;
      case "warning":
        return AlertTriangle;
      default:
        return Info;
    }
  }

  function getSeverityClass(severity: string): string {
    switch (severity) {
      case "critical":
        return "border-destructive/50 bg-destructive/10";
      case "warning":
        return "border-yellow-500/50 bg-yellow-500/10";
      default:
        return "border-blue-500/50 bg-blue-500/10";
    }
  }

  let statusInfo = $derived(getStatusInfo(results.status));
  let hasIssues = $derived(criticalFindings.length > 0 || warningFindings.length > 0);
</script>

<WizardHeader
  title="Test Results"
  description="Review what the test agent discovered"
/>

<div class="space-y-4 mt-4">
  <!-- Status Summary -->
  <div class="flex items-center justify-between p-4 rounded-lg border border-border bg-card">
    <div class="flex items-center gap-3">
      <svelte:component this={statusInfo.icon} class="w-5 h-5 {statusInfo.class}" />
      <div>
        <span class="font-medium">{statusInfo.label}</span>
        <span class="text-muted-foreground ml-2">in {formatDuration(results.durationMs)}</span>
      </div>
    </div>
    <div class="flex items-center gap-2 text-sm text-muted-foreground">
      {#if criticalFindings.length > 0}
        <span class="flex items-center gap-1 text-destructive">
          <AlertCircle class="w-4 h-4" />
          {criticalFindings.length} critical
        </span>
      {/if}
      {#if warningFindings.length > 0}
        <span class="flex items-center gap-1 text-yellow-500">
          <AlertTriangle class="w-4 h-4" />
          {warningFindings.length} warnings
        </span>
      {/if}
      {#if infoFindings.length > 0}
        <span class="flex items-center gap-1 text-blue-500">
          <Info class="w-4 h-4" />
          {infoFindings.length} info
        </span>
      {/if}
    </div>
  </div>

  <!-- Findings List -->
  {#if results.findings.length > 0}
    <div class="space-y-2 max-h-[200px] overflow-auto">
      {#each results.findings as finding}
        {@const FindingIcon = getFindingIcon(finding.findingType)}
        <div class="p-3 rounded-lg border {getSeverityClass(finding.severity)}">
          <div class="flex items-start gap-3">
            <FindingIcon class="w-5 h-5 mt-0.5 flex-shrink-0" />
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2">
                <span class="font-medium">{finding.title}</span>
                <span class="text-xs px-1.5 py-0.5 rounded bg-background/50 text-muted-foreground">
                  {finding.findingType}
                </span>
              </div>
              <p class="text-sm text-muted-foreground mt-1">{finding.description}</p>
              {#if finding.resolutionHint}
                <div class="mt-2 p-2 rounded bg-background/50 font-mono text-xs">
                  {finding.resolutionHint}
                </div>
              {/if}
            </div>
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <div class="p-6 rounded-lg border border-border bg-card text-center">
      <CheckCircle class="w-8 h-8 mx-auto text-green-500 mb-2" />
      <p class="font-medium">No issues found!</p>
      <p class="text-sm text-muted-foreground mt-1">
        The test completed without detecting any missing tools or authentication issues.
      </p>
    </div>
  {/if}

  <!-- Recommendations -->
  {#if results.recommendations.length > 0}
    <div class="p-4 rounded-lg border border-border bg-card">
      <h4 class="font-medium mb-2">Recommendations</h4>
      <ul class="space-y-1 text-sm text-muted-foreground">
        {#each results.recommendations as rec}
          <li class="flex items-start gap-2">
            <span class="text-primary">-</span>
            {rec}
          </li>
        {/each}
      </ul>
    </div>
  {/if}

  <!-- Raw Output Summary (collapsible) -->
  {#if results.rawOutputSummary}
    <details class="rounded-lg border border-border">
      <summary class="p-3 cursor-pointer text-sm font-medium hover:bg-muted/50">
        View Raw Output Summary
      </summary>
      <div class="p-3 pt-0 text-xs font-mono text-muted-foreground whitespace-pre-wrap border-t border-border mt-2">
        {results.rawOutputSummary}
      </div>
    </details>
  {/if}
</div>

<!-- Navigation -->
<WizardNavigation
  showBack={true}
  onBack={onEditDraft}
  backLabel="Edit Draft"
  onNext={onSave}
  nextLabel={hasIssues ? "Save Anyway" : "Save Instruction"}
>
  <Button variant="outline" onclick={onRunAgain} class="gap-2">
    <RefreshCw class="w-4 h-4" />
    Run Again
  </Button>
</WizardNavigation>
