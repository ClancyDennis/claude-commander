export const apiCosts = {
  id: 'api-costs',
  title: 'Understanding API Costs',
  summary: 'How API usage and billing works.',
  keywords: ['cost', 'price', 'money', 'billing', 'tokens', 'usage', 'api'],
  content: `
## How Costs Work

Claude Commander uses the Anthropic API, which charges based on tokens (roughly 4 characters = 1 token).

## What affects cost
- **Task complexity** - Bigger tasks = more tokens
- **Conversation length** - Longer chats = more context
- **File sizes** - Large files use more tokens when read

## Tips to reduce costs
- Be specific in task descriptions
- Break large tasks into smaller ones
- Avoid unnecessary back-and-forth

## Monitoring usage
Check your usage at console.anthropic.com

## Typical costs
- Simple file edits: $0.01-0.05
- Feature implementation: $0.10-0.50
- Complex refactoring: $0.50-2.00

*Costs vary based on model and usage patterns.*
  `,
};
