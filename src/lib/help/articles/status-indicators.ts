export const statusIndicators = {
  id: 'status-indicators',
  title: 'Understanding Status Indicators',
  summary: 'Learn what each status badge means.',
  keywords: ['status', 'badge', 'running', 'waiting', 'error', 'success', 'state'],
  content: `
## Status Types

- **Running** (blue) - Helper is actively working
- **Waiting** (yellow) - Waiting for your input or approval
- **Success** (green) - Task completed successfully
- **Error** (red) - Something went wrong
- **Idle** (gray) - Not currently active

## What to do when...

**Waiting**: Check the chat panel - the helper may have a question or need approval for changes.

**Error**: Read the error message in the agent view. Common fixes:
- Check your API key is valid
- Verify the working directory exists
- Review the task description for clarity

**Success**: Review the changes made before accepting them.
  `,
};
