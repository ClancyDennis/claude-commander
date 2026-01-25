<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";
  import type { TestAgentSession, AgentOutputEvent } from "../../types";
  import { Button } from "$lib/components/ui/button";
  import { Square, BarChart2 } from "lucide-svelte";

  interface Props {
    session: TestAgentSession;
    onStop: () => void;
    onAnalyze: () => void;
  }

  let { session, onStop, onAnalyze }: Props = $props();

  let outputs = $state<string[]>([]);
  let elapsedSeconds = $state(0);
  let outputContainer: HTMLDivElement | null = null;
  let unlistenOutput: (() => void) | null = null;
  let unlistenStatus: (() => void) | null = null;
  let timer: number | null = null;

  const MAX_TIMEOUT_SECONDS = 120; // 2 minutes

  onMount(async () => {
    // Listen for agent output
    unlistenOutput = await listen<AgentOutputEvent>("agent:output", (event) => {
      if (event.payload.agent_id === session.agentId) {
        outputs = [...outputs, event.payload.content];
        // Auto-scroll
        if (outputContainer) {
          outputContainer.scrollTop = outputContainer.scrollHeight;
        }
      }
    });

    // Listen for agent status changes
    unlistenStatus = await listen("agent:status", (event: any) => {
      if (event.payload.agent_id === session.agentId) {
        if (event.payload.status === "stopped" || event.payload.status === "error") {
          // Auto-analyze when done
          onAnalyze();
        }
      }
    });

    // Start timer
    timer = window.setInterval(() => {
      elapsedSeconds = Math.floor((Date.now() - session.startedAt) / 1000);

      // Auto-stop after timeout
      if (elapsedSeconds >= MAX_TIMEOUT_SECONDS) {
        onStop();
      }
    }, 1000);
  });

  onDestroy(() => {
    if (unlistenOutput) unlistenOutput();
    if (unlistenStatus) unlistenStatus();
    if (timer) clearInterval(timer);
  });

  function formatTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
  }

  let progressPercent = $derived(Math.min(100, (elapsedSeconds / MAX_TIMEOUT_SECONDS) * 100));
</script>

<div class="space-y-4">
  <!-- Progress bar -->
  <div class="space-y-2">
    <div class="flex items-center justify-between text-sm">
      <span class="text-muted-foreground">Elapsed: {formatTime(elapsedSeconds)}</span>
      <span class="text-muted-foreground">Max: {formatTime(MAX_TIMEOUT_SECONDS)}</span>
    </div>
    <div class="h-2 w-full bg-muted rounded-full overflow-hidden">
      <div
        class="h-full bg-primary transition-all duration-1000"
        style="width: {progressPercent}%"
      ></div>
    </div>
  </div>

  <!-- Output display -->
  <div
    bind:this={outputContainer}
    class="rounded-lg border border-border bg-black/90 p-4 h-[250px] overflow-auto font-mono text-sm text-green-400"
  >
    {#if outputs.length === 0}
      <p class="text-muted-foreground animate-pulse">Starting test agent...</p>
    {:else}
      {#each outputs as output}
        <pre class="whitespace-pre-wrap">{output}</pre>
      {/each}
    {/if}
  </div>

  <!-- Actions -->
  <div class="flex gap-2 justify-end">
    <Button variant="outline" onclick={onStop} class="gap-2">
      <Square class="w-4 h-4" />
      Stop Test
    </Button>
    <Button onclick={onAnalyze} class="gap-2">
      <BarChart2 class="w-4 h-4" />
      Analyze Results
    </Button>
  </div>
</div>

<style>
  pre {
    margin: 0;
  }
</style>
