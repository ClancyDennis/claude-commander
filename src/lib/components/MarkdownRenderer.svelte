<script lang="ts">
  import { marked } from 'marked';
  import DOMPurify from 'dompurify';
  import hljs from 'highlight.js';
  import 'highlight.js/styles/atom-one-dark.css';

  let { content }: { content: string } = $props();
  let container: HTMLDivElement | null = $state(null);
  let cleanupFns: (() => void)[] = [];
  let lastRenderedContent: string | null = null;
  let pendingRender: number | null = null;

  /**
   * Escape angle brackets that look like placeholders (e.g., <title>, <name>)
   * but preserve valid HTML-like patterns used in markdown (e.g., code blocks).
   */
  function escapeAngleBracketPlaceholders(text: string): string {
    // Match <word> patterns that look like placeholders (single word, no attributes)
    // but NOT valid markdown/HTML like <pre>, <code>, <a href=...>, etc.
    return text.replace(/<([a-zA-Z_][a-zA-Z0-9_-]*)>/g, (match, word) => {
      // List of common HTML tags we want to preserve
      const htmlTags = ['a', 'b', 'i', 'u', 'p', 'br', 'hr', 'em', 'strong', 'code', 'pre', 'ul', 'ol', 'li', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'div', 'span', 'img', 'table', 'tr', 'td', 'th', 'thead', 'tbody', 'blockquote'];
      if (htmlTags.includes(word.toLowerCase())) {
        return match; // Keep valid HTML tags
      }
      return `&lt;${word}&gt;`; // Escape placeholder-style angle brackets
    });
  }

  /**
   * Fix unclosed code fences that would cause the rest of content to render as code.
   */
  function fixUnclosedCodeFences(text: string): string {
    // Count occurrences of ``` (code fence markers)
    const fenceMatches = text.match(/^```/gm);
    if (!fenceMatches) return text;

    // If odd number of fences, add a closing fence
    if (fenceMatches.length % 2 !== 0) {
      return text + '\n```';
    }
    return text;
  }

  function renderMarkdown(targetContainer: HTMLDivElement, markdownContent: string) {
    // Fix unclosed code fences first
    let preprocessed = fixUnclosedCodeFences(markdownContent);
    // Escape placeholder-style angle brackets before parsing
    preprocessed = escapeAngleBracketPlaceholders(preprocessed);
    // Parse markdown synchronously
    const rawHtml = marked.parse(preprocessed, { async: false }) as string;
    // Sanitize
    const cleanHtml = DOMPurify.sanitize(rawHtml);
    // Update DOM
    targetContainer.innerHTML = cleanHtml;

    // Defer syntax highlighting to next frame to avoid blocking input
    requestAnimationFrame(() => {
      if (!targetContainer.isConnected) return;

      // Apply syntax highlighting
      targetContainer.querySelectorAll('pre code').forEach((el) => {
        hljs.highlightElement(el as HTMLElement);
      });

      // Add copy buttons to code blocks
      targetContainer.querySelectorAll('pre').forEach((pre) => {
        if (pre.querySelector('.copy-btn')) return; // Already added

        const copyBtn = document.createElement('button');
        copyBtn.className = 'copy-btn';
        copyBtn.innerHTML = `
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2 2v1"></path>
          </svg>
        `;
        copyBtn.title = "Copy code";

        let timeoutId: ReturnType<typeof setTimeout> | null = null;

        const clickHandler = () => {
          const code = pre.querySelector('code')?.innerText || '';
          navigator.clipboard.writeText(code);

          // Clear any pending timeout
          if (timeoutId) clearTimeout(timeoutId);

          // Feedback
          const originalHTML = copyBtn.innerHTML;
          copyBtn.innerHTML = `
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
              <polyline points="20 6 9 17 4 12"></polyline>
            </svg>
          `;
          copyBtn.classList.add('copied');

          timeoutId = setTimeout(() => {
            if (copyBtn.isConnected) {
              copyBtn.innerHTML = originalHTML;
              copyBtn.classList.remove('copied');
            }
          }, 2000);
        };

        copyBtn.addEventListener('click', clickHandler);

        // Track cleanup for this button
        cleanupFns.push(() => {
          if (timeoutId) clearTimeout(timeoutId);
          copyBtn.removeEventListener('click', clickHandler);
        });

        pre.style.position = 'relative';
        pre.appendChild(copyBtn);
      });
    });
  }

  $effect(() => {
    if (container && content) {
      // Skip if content hasn't changed (memoization)
      if (content === lastRenderedContent) return;

      // Clean up previous event listeners
      cleanupFns.forEach(fn => fn());
      cleanupFns = [];

      // Cancel any pending render
      if (pendingRender !== null) {
        cancelAnimationFrame(pendingRender);
      }

      // Defer rendering to avoid blocking main thread during rapid updates
      pendingRender = requestAnimationFrame(() => {
        pendingRender = null;
        if (container && container.isConnected) {
          renderMarkdown(container, content);
          lastRenderedContent = content;
        }
      });
    }

    // Return cleanup function for when effect re-runs or component unmounts
    return () => {
      if (pendingRender !== null) {
        cancelAnimationFrame(pendingRender);
        pendingRender = null;
      }
      cleanupFns.forEach(fn => fn());
      cleanupFns = [];
    };
  });
</script>

<div class="markdown-content" bind:this={container}></div>

<style>
  :global(.markdown-content) {
    font-size: 14px;
    line-height: 1.6;
    color: var(--text-primary);
  }

  :global(.markdown-content h1),
  :global(.markdown-content h2),
  :global(.markdown-content h3),
  :global(.markdown-content h4) {
    margin-top: 1.5em;
    margin-bottom: 0.5em;
    font-weight: 600;
    color: var(--text-primary);
  }

  :global(.markdown-content h1) { font-size: 1.5em; }
  :global(.markdown-content h2) { font-size: 1.3em; }
  :global(.markdown-content h3) { font-size: 1.1em; }

  :global(.markdown-content p) {
    margin-bottom: 1em;
  }

  :global(.markdown-content ul),
  :global(.markdown-content ol) {
    margin-bottom: 1em;
    padding-left: 1.5em;
  }

  :global(.markdown-content li) {
    margin-bottom: 0.25em;
  }

  :global(.markdown-content code) {
    font-family: 'SF Mono', 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    font-size: 0.9em;
    background-color: rgba(255, 255, 255, 0.1);
    padding: 0.2em 0.4em;
    border-radius: 4px;
  }

  :global(.markdown-content pre) {
    margin: 1em 0;
    padding: 1em;
    background-color: #1e1e24 !important; /* Override theme slightly to match app */
    border-radius: 8px;
    overflow-x: auto;
    position: relative;
    border: 1px solid var(--border);
  }

  :global(.markdown-content pre code) {
    background-color: transparent;
    padding: 0;
    color: inherit;
    font-size: 13px;
  }

  :global(.markdown-content blockquote) {
    border-left: 4px solid var(--accent);
    margin: 1em 0;
    padding-left: 1em;
    color: var(--text-secondary);
    background: rgba(240, 112, 90, 0.05);
    padding: 0.5em 1em;
    border-radius: 0 4px 4px 0;
  }

  :global(.markdown-content a) {
    color: var(--accent);
    text-decoration: none;
  }

  :global(.markdown-content a:hover) {
    text-decoration: underline;
  }
  
  /* Copy Button Styles */
  :global(.copy-btn) {
    position: absolute;
    top: 8px;
    right: 8px;
    background: rgba(255, 255, 255, 0.1);
    border: none;
    border-radius: 4px;
    padding: 4px;
    cursor: pointer;
    color: var(--text-muted);
    transition: all 0.2s;
    opacity: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  :global(pre:hover .copy-btn) {
    opacity: 1;
  }

  :global(.copy-btn:hover) {
    background: rgba(255, 255, 255, 0.2);
    color: var(--text-primary);
  }

  :global(.copy-btn.copied) {
    color: var(--success);
  }
</style>
