# Configuration

This document covers all configuration options for Claude Commander.

---

## Environment Variables

Create a `.env` file in the project root (copy from `.env.example`):

```bash
# Required: At least one AI provider
ANTHROPIC_API_KEY=sk-ant-...
OPENAI_API_KEY=sk-...

# Primary model for the meta-agent
# Provider is auto-detected from model name
PRIMARY_MODEL=claude-sonnet-4-5-20250929
```

### API Keys

| Variable | Required | Description |
|----------|----------|-------------|
| `ANTHROPIC_API_KEY` | Yes* | Your Anthropic API key |
| `OPENAI_API_KEY` | Yes* | Your OpenAI API key |

*At least one is required. Both can be set for flexibility.

### Model Selection

| Variable | Default | Description |
|----------|---------|-------------|
| `PRIMARY_MODEL` | `claude-sonnet-4-5-20250929` | Main model for meta-agent |
| `SECURITY_MODEL` | (same as PRIMARY_MODEL) | Model for security analysis |
| `LIGHT_TASK_MODEL` | `claude-haiku-4-5` | Model for lightweight tasks |
| `CLAUDE_CODE_MODEL` | `auto` | Model for Claude Code workers |

**Provider auto-detection**: The provider is inferred from the model name:
- `gpt-*`, `o1-*`, `o3-*` → OpenAI
- `claude-*` or aliases (`sonnet`, `opus`, `haiku`) → Anthropic

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

### Elevation Wrapper Scripts

Wrapper scripts for sudo command interception are installed on startup:

| Platform | Path |
|----------|------|
| Linux | `~/.local/share/claude-commander/elevation-bin/linux/sudo` |
| macOS | `~/Library/Application Support/claude-commander/elevation-bin/macos/sudo` |
| Windows | `%LOCALAPPDATA%\claude-commander\elevation-bin\windows\sudo` |

These scripts are automatically injected into agent PATH environments.

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

## Voice Features (Beta)

Voice features require an OpenAI API key with access to the Realtime API.

### Requirements

| Variable | Required | Description |
|----------|----------|-------------|
| `OPENAI_API_KEY` | Yes | OpenAI API key for Realtime API access |

### Voice Modes

| Mode | Model | Purpose |
|------|-------|---------|
| Dictate | `gpt-realtime-mini` | Speech-to-text transcription |
| Discuss | `gpt-realtime-mini` | Bidirectional voice conversation with tool access |
| Attention | `gpt-realtime-mini` | Task completion notifications (auto-closes after 10s) |

### Settings

| Setting | Default | Location | Description |
|---------|---------|----------|-------------|
| Attention notifications | On | Settings panel | Auto-announce when tasks complete |
| Audio mute | Off | Voice sidebar | Mute AI voice output |

### Audio Configuration (Internal)

These settings are hardcoded in the backend:

- **Format**: PCM16 (base64-encoded)
- **Transcription model**: Whisper-1
- **Turn detection**: Server-side VAD
  - Threshold: 0.5
  - Silence duration: 700ms (dictate: 500ms)
  - Prefix padding: 300ms

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

## Elevated Command Approval (Sudo)

When agents need to run commands with elevated privileges, Claude Commander intercepts the request and prompts for approval.

### Platform Requirements

| Platform | Requirement | Installation |
|----------|-------------|--------------|
| Linux | polkit (pkexec) | Usually pre-installed on desktop systems |
| macOS | None | Uses built-in osascript |
| Windows | gsudo | `winget install gsudo` or download from [github.com/gerardog/gsudo](https://github.com/gerardog/gsudo) |

### How It Works

1. Agent runs a command with `sudo` (e.g., `sudo apt install nginx`)
2. Claude Commander's wrapper script intercepts the call
3. A dialog appears showing the command and risk level
4. You approve or deny the request
5. If approved, your OS prompts for password/biometrics
6. The command executes with actual elevated privileges

### Risk Levels

| Level | Examples | UI Treatment |
|-------|----------|--------------|
| Normal | `apt install`, `systemctl restart` | Standard approval dialog |
| Suspicious | `curl ... \| bash`, `bash -c "..."` | Warning banner displayed |
| High Risk | `rm -rf /`, `dd if=...`, `mkfs` | Extra confirmation checkbox required |

### Script-Scoped Approval

For installer scripts that need multiple sudo calls (like Homebrew), you can select "Approve All for This Script" to approve all sudo commands from the same parent process without repeated prompts.

---

## See Also

- [ARCHITECTURE.md](./ARCHITECTURE.md) — Technical internals
- [CONTRIBUTING.md](./CONTRIBUTING.md) — Development setup
- [README.md](./README.md) — Getting started
