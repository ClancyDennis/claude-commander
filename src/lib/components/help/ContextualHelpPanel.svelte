<script lang="ts">
  import * as Sheet from "$lib/components/ui/sheet";
  import { Button } from "$lib/components/ui/button";
  import { Search, ArrowLeft, ChevronRight } from "lucide-svelte";
  import { searchHelp, helpArticles, getArticleById, type HelpArticle } from "$lib/help";

  let {
    open = $bindable(false),
  }: {
    open?: boolean;
  } = $props();

  let searchQuery = $state("");
  let selectedArticleId = $state<string | null>(null);

  const filteredArticles = $derived(
    searchQuery.trim() ? searchHelp(searchQuery) : helpArticles
  );

  const selectedArticle = $derived(
    selectedArticleId ? getArticleById(selectedArticleId) : null
  );

  function selectArticle(id: string) {
    selectedArticleId = id;
  }

  function goBack() {
    selectedArticleId = null;
  }

  function renderMarkdown(content: string): string {
    // Simple markdown-style rendering without external libraries
    return content
      // Headers
      .replace(/^### (.+)$/gm, '<h3 class="text-base font-semibold mt-4 mb-2">$1</h3>')
      .replace(/^## (.+)$/gm, '<h2 class="text-lg font-semibold mt-5 mb-2">$1</h2>')
      .replace(/^# (.+)$/gm, '<h1 class="text-xl font-bold mt-6 mb-3">$1</h1>')
      // Bold
      .replace(/\*\*(.+?)\*\*/g, '<strong class="font-semibold">$1</strong>')
      // Italic
      .replace(/\*(.+?)\*/g, '<em>$1</em>')
      // Inline code
      .replace(/`([^`]+)`/g, '<code class="px-1.5 py-0.5 rounded bg-muted text-sm font-mono">$1</code>')
      // Unordered lists
      .replace(/^- (.+)$/gm, '<li class="ml-4 list-disc">$1</li>')
      // Ordered lists
      .replace(/^(\d+)\. (.+)$/gm, '<li class="ml-4 list-decimal">$2</li>')
      // Paragraphs (double newlines)
      .replace(/\n\n/g, '</p><p class="mb-3">')
      // Single newlines within content
      .replace(/\n/g, '<br/>');
  }
</script>

<Sheet.Root bind:open>
  <Sheet.Content side="right" class="w-[400px] flex flex-col">
    <Sheet.Header class="space-y-4">
      <Sheet.Title>Help</Sheet.Title>

      <!-- Search input with icon -->
      <div class="relative">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
        <input
          type="text"
          placeholder="Search help articles..."
          bind:value={searchQuery}
          class="w-full pl-10 pr-4 py-2 text-sm rounded-md border border-input bg-background focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2"
        />
      </div>
    </Sheet.Header>

    <!-- Scrollable article list or article detail -->
    <div class="flex-1 overflow-y-auto mt-4 pr-2">
      {#if selectedArticle}
        <!-- Back button + Article content -->
        <div class="space-y-4">
          <Button
            variant="ghost"
            size="sm"
            onclick={goBack}
            class="gap-2 -ml-2"
          >
            <ArrowLeft class="h-4 w-4" />
            Back to articles
          </Button>

          <article class="space-y-2">
            <h2 class="text-xl font-bold">{selectedArticle.title}</h2>
            <p class="text-sm text-muted-foreground">{selectedArticle.summary}</p>

            <div class="prose prose-sm dark:prose-invert pt-4">
              {@html renderMarkdown(selectedArticle.content)}
            </div>
          </article>
        </div>
      {:else}
        <!-- Article list filtered by search -->
        <div class="space-y-2">
          {#if filteredArticles.length === 0}
            <p class="text-sm text-muted-foreground text-center py-8">
              No articles found for "{searchQuery}"
            </p>
          {:else}
            {#each filteredArticles as article (article.id)}
              <button
                type="button"
                onclick={() => selectArticle(article.id)}
                class="w-full text-left p-3 rounded-lg border border-border hover:bg-accent hover:border-accent transition-colors group"
              >
                <div class="flex items-start justify-between gap-2">
                  <div class="flex-1 min-w-0">
                    <h3 class="font-medium text-sm truncate">{article.title}</h3>
                    <p class="text-xs text-muted-foreground mt-1 line-clamp-2">
                      {article.summary}
                    </p>
                  </div>
                  <ChevronRight class="h-4 w-4 text-muted-foreground group-hover:text-foreground flex-shrink-0 mt-0.5" />
                </div>
              </button>
            {/each}
          {/if}
        </div>
      {/if}
    </div>

    <Sheet.Footer class="border-t pt-4 mt-4">
      <p class="text-sm text-muted-foreground w-full text-center">
        Press <kbd class="px-1.5 py-0.5 rounded bg-muted text-xs font-mono">F1</kbd> anytime to open help
      </p>
    </Sheet.Footer>
  </Sheet.Content>
</Sheet.Root>

<style>
  .line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  /* Prose-like styling for rendered markdown */
  :global(.prose) {
    line-height: 1.6;
  }

  :global(.prose h1),
  :global(.prose h2),
  :global(.prose h3) {
    line-height: 1.3;
  }

  :global(.prose li) {
    margin-top: 0.25rem;
    margin-bottom: 0.25rem;
  }

  :global(.prose p) {
    margin-bottom: 0.75rem;
  }
</style>
