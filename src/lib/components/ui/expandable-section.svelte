<script lang="ts">
  import * as Collapsible from "$lib/components/ui/collapsible";
  import { ChevronDown } from "lucide-svelte";
  import { cn } from "$lib/utils";
  import type { Snippet } from "svelte";

  interface Props {
    title: string;
    badge?: string;
    defaultOpen?: boolean;
    class?: string;
    children?: Snippet;
  }

  let {
    title,
    badge = "",
    defaultOpen = false,
    class: className,
    children,
  }: Props = $props();

  let open = $state(defaultOpen);
</script>

<Collapsible.Root bind:open class={cn("border border-border rounded-lg", className)}>
  <Collapsible.Trigger
    class="flex w-full items-center justify-between p-4 hover:bg-muted/50 transition-colors rounded-lg"
  >
    <div class="flex items-center gap-2">
      <span class="font-medium text-foreground">{title}</span>
      {#if badge}
        <span class="text-xs text-muted-foreground bg-muted px-2 py-0.5 rounded">
          {badge}
        </span>
      {/if}
    </div>
    <ChevronDown
      class={cn(
        "h-4 w-4 text-muted-foreground transition-transform duration-200",
        open && "rotate-180"
      )}
    />
  </Collapsible.Trigger>
  <Collapsible.Content class="px-4 pb-4">
    {#if children}
      {@render children()}
    {/if}
  </Collapsible.Content>
</Collapsible.Root>
