# Configuration

This document covers all configuration options for Claude Commander.

---

## Environment Variables

Create a `.env` file in the project root (copy from `.env.example`):

```bash
# Required: At least one AI provider
ANTHROPIC_API_KEY=sk-ant-...
OPENAI_API_KEY=sk-...

# Optional: Model selection
ANTHROPIC_MODEL=claude-sonnet-4-5-20241022
OPENAI_MODEL=gpt-4o

# Optional: Meta-agent provider (defaults to anthropic)
META_AGENT_PROVIDER=anthropic  # or "openai"
```

### API Keys

| Variable | Required | Description |
|----------|----------|-------------|
| `ANTHROPIC_API_KEY` | Yes* | Your Anthropic API key |
| `OPENAI_API_KEY` | Yes* | Your OpenAI API key |

*At least one is required. Both can be set for flexibility.

### Model Selection

| Variable | Default | Options |
|----------|---------|---------|
| `ANTHROPIC_MODEL` | `claude-sonnet-4-5-20241022` | Any Claude model |
| `OPENAI_MODEL` | `gpt-4o` | Any OpenAI model |
| `META_AGENT_PROVIDER` | `anthropic` | `anthropic` or `openai` |

---

## Pipeline Settings

Configure via the gear icon in the agent list (AgentSettings component).

### Phase A: Agent Pool (P-Thread)

| Setting | Default | Description |
|---------|---------|-------------|
| Enable shared pool | Off | Use shared agent pool for parallel execution |
| Pool priority | Normal | `low`, `normal`, or `high` |

The agent pool allows multiple pipelines to share agents, improving resource utilization.

### Phase B: Orchestration (B-Thread)

| Setting | Default | Description |
|---------|---------|-------------|
| Enable task decomposition | Off | Automatically break tasks into subtasks |
| Max parallel tasks | 3 | Maximum concurrent subtasks |

When enabled, the orchestrator analyzes your request and creates a dependency graph of subtasks that can run in parallel.

### Phase C: Verification (F-Thread)

| Setting | Default | Description |
|---------|---------|-------------|
| Enable Best-of-N | Off | Run task with multiple agents |
| Strategy | `majority` | `majority`, `weighted`, `meta`, `first` |
| N (agent count) | 3 | Number of agents to run |
| Confidence threshold | 0.7 | Minimum confidence for auto-approval |

**Verification Strategies:**

- **Majority**: Simple majority vote among agents
- **Weighted**: Confidence-weighted voting
- **Meta-agent**: LLM reviews all outputs, picks best
- **First correct**: Returns first output passing validation

### Phase D: Checkpoints (C-Thread)

| Setting | Default | Description |
|---------|---------|-------------|
| Require plan review | On | Human must approve plan before execution |
| Require final review | On | Human must approve before completion |
| Auto-validation command | — | Command to run (e.g., `cargo check`) |
| Auto-approve on pass | Off | Skip final review if validation passes |

---

## Hook Server

The hook server listens for Claude Code tool events.

| Setting | Value | Notes |
|---------|-------|-------|
| Port | 19832 | Fixed, not configurable |
| Endpoint | `/hook` | POST endpoint for events |

### Configuring Claude Code

Add to your Claude Code configuration:

```bash
claude-agent-manager --hook-url http://localhost:19832/hook
```

Or set in Claude Code's hooks config:

```json
{
  "hooks": {
    "PreToolUse": "http://localhost:19832/hook",
    "PostToolUse": "http://localhost:19832/hook"
  }
}
```

---

## Data Locations

### Cost History

| Platform | Path |
|----------|------|
| Linux | `~/.local/share/claude-commander/cost_history.json` |
| macOS | `~/Library/Application Support/claude-commander/cost_history.json` |
| Windows | `%LOCALAPPDATA%\claude-commander\cost_history.json` |

### Agent Run History (SQLite)

| Platform | Path |
|----------|------|
| Linux | `~/.local/share/claude-commander/agent_runs.db` |
| macOS | `~/Library/Application Support/claude-commander/agent_runs.db` |
| Windows | `%LOCALAPPDATA%\claude-commander\agent_runs.db` |

### Instruction Files

Default location for instruction files:

```
<project-root>/.claude-commander-instructions/
```

Or specify a custom path when creating agents.

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` / `Cmd+N` | New Agent/Pipeline |
| `Ctrl+Shift+$` / `Cmd+Shift+$` | Toggle Cost Tracker |
| `Ctrl+P` / `Cmd+P` | Toggle Pool Dashboard |
| `Ctrl+Shift+C` / `Cmd+Shift+C` | Open Chat View |
| `Ctrl+/` / `Cmd+/` | Cycle layouts (Single → Split → Grid) |
| `Ctrl+1-9` / `Cmd+1-9` | Jump to agent 1-9 |
| `Escape` | Close dialogs |

---

## Security Monitoring

Security monitoring is enabled by default. Configure via the security settings:

| Setting | Default | Description |
|---------|---------|-------------|
| Enable monitoring | On | Active prompt-injection detection |
| Block suspicious | Off | Automatically block flagged operations |
| LLM analysis | On | Use LLM for semantic analysis |

### Detection Methods

1. **Regex patterns**: Fast matching for known threat signatures
2. **LLM semantic analysis**: Detects sophisticated attacks
3. **Session expectations**: Flags unexpected tool usage

---

## See Also

- [ARCHITECTURE.md](./ARCHITECTURE.md) — Technical internals
- [CONTRIBUTING.md](./CONTRIBUTING.md) — Development setup
- [README.md](./README.md) — Getting started
