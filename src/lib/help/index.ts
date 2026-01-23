import { gettingStarted } from './articles/getting-started';
import { creatingHelpers } from './articles/creating-helpers';
import { statusIndicators } from './articles/status-indicators';
import { chatInterface } from './articles/chat-interface';
import { troubleshooting } from './articles/troubleshooting';
import { apiCosts } from './articles/api-costs';

export interface HelpArticle {
  id: string;
  title: string;
  summary: string;
  keywords: string[];
  content: string;
}

export const helpArticles: HelpArticle[] = [
  gettingStarted,
  creatingHelpers,
  statusIndicators,
  chatInterface,
  troubleshooting,
  apiCosts,
];

export function searchHelp(query: string): HelpArticle[] {
  const q = query.toLowerCase();
  return helpArticles.filter(article =>
    article.title.toLowerCase().includes(q) ||
    article.summary.toLowerCase().includes(q) ||
    article.keywords.some(k => k.toLowerCase().includes(q))
  );
}

export function getArticleById(id: string): HelpArticle | undefined {
  return helpArticles.find(a => a.id === id);
}
