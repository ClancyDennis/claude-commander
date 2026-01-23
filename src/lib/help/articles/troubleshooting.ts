export const troubleshooting = {
  id: 'troubleshooting',
  title: 'Troubleshooting',
  summary: 'Common issues and how to fix them.',
  keywords: ['error', 'problem', 'fix', 'issue', 'help', 'stuck', 'broken'],
  content: `
## Common Issues

### "API key not found"
1. Open Settings (gear icon)
2. Enter your Anthropic API key
3. Click Save

### "Helper stuck on waiting"
The helper needs your input. Check the chat panel for questions.

### "Changes didn't apply"
- Check if the helper finished (green status)
- Review the file diff in the agent view
- Use git status to see pending changes

### "Helper made wrong changes"
1. Use chat to correct: "That's not quite right, instead..."
2. Or cancel and start fresh with a clearer description

### "Can't connect to helper"
- Check your internet connection
- Verify the API key is valid
- Try restarting Claude Commander
  `,
};
