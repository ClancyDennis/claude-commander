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

<Collapsible.Root bind:open class={cn("border border-border rounded-[10px]", className)}>
  <Collapsible.Trigger
    class="flex w-full items-center justify-between p-3 hover:bg-white/5 transition-colors rounded-[10px]"
  >
    <div class="flex items-center gap-2 min-w-0">
      <span class="font-medium text-foreground text-sm">{title}</span>
      {#if badge}
        <span class="text-[11px] text-muted-foreground bg-muted px-2 py-0.5 rounded-md flex-shrink-0">
          {badge}
        </span>
      {/if}
    </div>
    <ChevronDown
      class={cn(
        "h-4 w-4 text-muted-foreground transition-transform duration-200 flex-shrink-0",
        open && "rotate-180"
      )}
    />
  </Collapsible.Trigger>
  <Collapsible.Content class="px-3 pb-3 overflow-x-hidden">
    {#if children}
      {@render children()}
    {/if}
  </Collapsible.Content>
</Collapsible.Root>
