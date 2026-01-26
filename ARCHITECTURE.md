# Architecture

This document describes the technical architecture of Claude Commander for developers who want to understand the internals, contribute, or build integrations.

---

## Overview

Claude Commander is a desktop application built with:
- **Frontend**: Svelte 5 + TypeScript + Vite
- **Backend**: Rust + Tauri v2
- **AI Integration**: Anthropic Claude API + OpenAI API
- **State Management**: Svelte 5 runes ($state, $derived)
- **HTTP Server**: Tokio + Axum (for hook server)
- **Persistence**: SQLite + JSON files

---

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Claude Commander                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   ┌──────────────────────┐    ┌──────────────────────┐              │
│   │   Frontend (Svelte)  │◄──►│   Backend (Rust)     │              │
│   │                      │    │                      │              │
│   │  • AgentList         │    │  • agent_manager     │              │
│   │  • AgentView         │    │  • meta_agent        │              │
│   │  • ChatView          │    │  • pipeline_manager  │              │
│   │  • PhaseProgress     │    │  • hook_server       │              │
│   │  • CostTracker       │    │  • ai_client         │              │
│   └──────────────────────┘    └──────────────────────┘              │
│                                        │                             │
│                                        ▼                             │
│                          ┌─────────────────────────┐                │
│                          │   Hook Server (19832)   │                │
│                          │   Receives tool events  │                │
│                          └─────────────────────────┘                │
│                                        ▲                             │
│                                        │                             │
│   ┌────────────────────────────────────┼────────────────────────┐   │
│   │                                    │                        │   │
│   ▼                                    ▼                        ▼   │
│  Claude Code Agent 1           Claude Code Agent 2         Agent N  │
│  (subprocess)                  (subprocess)              (subprocess)│
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
                    ┌───────────────────────────────┐
                    │      External APIs            │
                    │  • Anthropic Claude API       │
                    │  • OpenAI API                 │
                    │  • GitHub API                 │
                    └───────────────────────────────┘
```

---

## Backend (Rust)

### Core Modules

| Module | File | Purpose |
|--------|------|---------|
| Agent Manager | `agent_manager/mod.rs` | Agent lifecycle (create, start, stop, monitor) |
| Meta Agent | `meta_agent/mod.rs` | LLM-based supervisor that orchestrates agents |
| Hook Server | `hook_server/mod.rs` | HTTP server receiving Claude Code tool events |
| AI Client | `ai_client/mod.rs` | Unified client for Claude + OpenAI APIs |
| Types | `types.rs` | Shared type definitions |
| Tool Registry | `tool_registry.rs` | Meta-agent tool definitions |
| Error Types | `error.rs` | Unified structured error handling |
| DB Utilities | `db_utils.rs` | Database operation helpers |

#### Agent Manager (`agent_manager/`)

| Module | Purpose |
|--------|---------|
| `mod.rs` | Main agent manager logic |
| `process_spawner.rs` | Claude CLI subprocess management |
| `message_handlers.rs` | Processes agent messages |
| `result_handlers.rs` | Handles agent completion results |
| `stream_handler.rs` | Real-time output streaming |
| `stream_parser.rs` | Parses agent output streams |
| `statistics.rs` | Agent metrics and stats |
| `database_ops.rs` | Agent persistence operations |

#### Meta Agent (`meta_agent/`)

| Module | Purpose |
|--------|---------|
| `mod.rs` | Main meta-agent orchestration |
| `conversation_manager.rs` | History management with image support |
| `tool_loop_engine.rs` | Iterative tool execution cycle |
| `system_prompt.rs` | System prompt generation |
| `result_queue.rs` | Queued results for display |
| `action_logger.rs` | Action logging utilities |
| `tools/agent_tools.rs` | Agent management tools |
| `tools/fs_tools.rs` | File system tools |
| `tools/todo_tools.rs` | Task orchestration updates |

### Pipeline System

The "Ralphline" pipeline system implements a 4-phase workflow:

![Ralph Wheel](ralph_wheel_small.gif)

| Module | File | Pattern | Purpose |
|--------|------|---------|---------|
| Pipeline Manager | `pipeline_manager.rs` | — | Orchestrates the 4 phases |
| Orchestrator | `orchestrator.rs` | B-Thread | Task decomposition |
| Agent Pool | `agent_pool.rs` | P-Thread | Shared agent pool for parallel execution |
| Verification Engine | `verification_engine.rs` | F-Thread | Best-of-N verification |

#### Pipeline Phases

```
Phase 1: Planning
    └─► Meta-agent analyzes request, creates execution plan
    └─► Human review checkpoint (optional)

Phase 2: Implementation
    └─► Task decomposition (B-Thread)
    └─► Parallel execution via agent pool (P-Thread)
    └─► Automatic validation (e.g., cargo check)

Phase 3: Testing
    └─► Best-of-N verification (F-Thread)
    └─► Multiple consensus strategies

Phase 4: Final Review
    └─► LLM approval checkpoint
    └─► Final validation
    └─► Loop back if needed
```

#### Verification Strategies (F-Thread)

The verification engine supports multiple fusion strategies:

- **Majority Vote**: Simple majority among N agents
- **Weighted Consensus**: Confidence-weighted voting
- **Meta-Agent Review**: LLM reviews all outputs, picks best
- **First Correct**: Returns first output passing validation

### Security Monitoring (`security_monitor/`)

| Module | Purpose |
|--------|---------|
| `mod.rs` | Main monitor orchestration |
| `builder.rs` | Fluent API for monitor construction |
| `pattern_matcher.rs` | Fast regex threat detection |
| `llm_analyzer.rs` | AI-based threat analysis |
| `prompts.rs` | LLM prompts for threat analysis |
| `expectation_generator.rs` | Predicts expected tool usage |
| `session_expectations.rs` | Tracks expected vs actual tool calls |
| `response_handler.rs` | Threat response actions |
| `anomaly_detection.rs` | Detects unusual behavior patterns |
| `collector.rs` | Collects security events |
| `rules.rs` | Security rule definitions |
| `path_matching.rs` | File path security matching |
| `parsing_utils.rs` | Security data parsing utilities |

The security system uses a hybrid approach:
1. **Fast regex patterns**: Known threat signatures (pattern_matcher)
2. **LLM semantic analysis**: Sophisticated attack detection (llm_analyzer + prompts)
3. **Session expectations**: Tracks expected vs. actual tool calls (session_expectations)
4. **Anomaly detection**: Flags unusual behavioral patterns

#### Preset Configurations

| Preset | Behavior |
|--------|----------|
| Default | Balanced monitoring with warnings |
| Strict | Auto-terminate on critical threats |
| Human Review | All actions require manual approval |

### Elevation System (Sudo Approval)

The elevation system allows agents to request and execute commands with elevated privileges.

| Component | Location | Purpose |
|-----------|----------|---------|
| Elevation API | `hook_server.rs` | HTTP endpoints for wrapper communication |
| Elevation Types | `types.rs` | `PendingElevatedCommand` struct |
| Tauri Commands | `commands/security.rs` | `approve_elevated_command`, `deny_elevated_command` |
| First Run | `first_run.rs` | Installs wrapper scripts on startup |

#### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│  Agent runs: sudo apt install nginx                             │
│                      │                                          │
│                      ▼                                          │
│  ┌───────────────────────────────────────────┐                 │
│  │  elevation-bin/linux/sudo (wrapper)        │                 │
│  │  - Intercepts sudo command                 │                 │
│  │  - POSTs to /elevated/request              │                 │
│  └───────────────────────────────────────────┘                 │
│                      │                                          │
│                      ▼                                          │
│  ┌───────────────────────────────────────────┐                 │
│  │  Hook Server (:19832)                      │                 │
│  │  - Queues request, emits elevated:request  │                 │
│  └───────────────────────────────────────────┘                 │
│                      │                                          │
│                      ▼                                          │
│  ┌───────────────────────────────────────────┐                 │
│  │  Frontend (ElevatedCommandModal)           │                 │
│  │  - Shows approval dialog with risk level   │                 │
│  │  - User clicks Approve/Deny                │                 │
│  └───────────────────────────────────────────┘                 │
│                      │                                          │
│                      ▼                                          │
│  ┌───────────────────────────────────────────┐                 │
│  │  Wrapper polls /elevated/status/:id        │                 │
│  │  - On approved: calls pkexec/gsudo         │                 │
│  │  - OS shows native auth dialog             │                 │
│  │  - Command executes with elevation         │                 │
│  └───────────────────────────────────────────┘                 │
└─────────────────────────────────────────────────────────────────┘
```

#### Platform-Specific Elevation

| Platform | Wrapper Script | Elevation Tool | Notes |
|----------|----------------|----------------|-------|
| Linux | `elevation-bin/linux/sudo` | pkexec (polkit) | Uses system polkit policy |
| macOS | `elevation-bin/macos/sudo` | osascript | File-based command passing |
| Windows | `elevation-bin/windows/sudo` | gsudo | Runs in Git Bash environment |

#### PATH Injection

Agents receive a modified PATH with the elevation-bin directory prepended:

```rust
// In agent_manager.rs
let new_path = format!("{}:{}", elevation_bin_path, existing_path);
cmd.env("PATH", new_path);
cmd.env("CLAUDE_AGENT_ID", agent_id);
```

This ensures our `sudo` wrapper is found before the system `sudo`.

#### Risk Classification

Commands are classified into risk levels:
- **Normal**: Standard elevation (package installs, service management)
- **Suspicious**: Warning for `curl | bash`, `bash -c`, etc.
- **High Risk**: Extra confirmation for `rm -rf`, `dd`, `mkfs`, etc.

#### Script-Scoped Approval

For installer scripts with multiple sudo calls, users can approve all commands from the same parent process:

```bash
# Wrapper tracks parent process
PARENT_PID=$PPID
SCRIPT_HASH=$(echo "$PARENT_PID-$PARENT_CMD" | md5sum)

# Check if scope is pre-approved
curl "$COMMANDER_URL/elevated/check-scope/$SCRIPT_HASH"
```

### Voice System (Beta)

Real-time voice interaction powered by OpenAI Realtime API.

| Module | File | Purpose |
|--------|------|---------|
| Session Registry | `voice/session_registry.rs` | Manages all voice session types |
| Session Manager | `voice/session_manager.rs` | Common traits and event types |
| Realtime (Dictate) | `voice/realtime.rs` | Speech-to-text transcription |
| Discuss | `voice/discuss.rs` | Bidirectional conversation with tool access |
| Attention | `voice/attention.rs` | Task completion notification overlay |
| Voice Tools | `voice/tools.rs` | Tool execution routing for voice sessions |
| Voice Commands | `voice/commands.rs` | Tauri command handlers |

#### Voice Modes

```
┌─────────────────────────────────────────────────────────────────┐
│  Dictate Mode                                                    │
│  User speaks → Transcription → Text returned to chat            │
├─────────────────────────────────────────────────────────────────┤
│  Discuss Mode                                                    │
│  User speaks ↔ AI responds (audio) ↔ Can call tools             │
│  Tool: talk_to_mission_control → Routes to MetaAgent            │
├─────────────────────────────────────────────────────────────────┤
│  Attention Mode                                                  │
│  Task completes → AI announces result → Auto-closes (10s)       │
│  User can ask brief follow-ups before timeout                   │
└─────────────────────────────────────────────────────────────────┘
```

### Agent Runs Database (`agent_runs_db/`)

| Module | Purpose |
|--------|---------|
| `mod.rs` | Main database interface |
| `crud.rs` | Create, read, update, delete operations |
| `queries.rs` | Query builders and search |
| `models.rs` | Data models and structs |
| `schema.rs` | Database schema definitions |
| `cost.rs` | Cost tracking persistence |
| `orchestrator_events.rs` | Pipeline event storage |

### Supporting Systems

| Module | File | Purpose |
|--------|------|---------|
| Instruction Manager | `instruction_manager.rs` | Markdown instruction handling |
| Instruction Wizard | `commands/instruction_wizard.rs` | AI-assisted instruction generation |
| Skill Generator | `skill_generator.rs` | Auto-generate skills from instructions |
| Subagent Generator | `subagent_generator.rs` | Auto-generate sub-agents |
| Claude.md Generator | `claudemd_generator.rs` | Task-specific context generation |
| GitHub Integration | `github.rs` | GitHub API integration |
| Logger | `logger.rs` | Centralized logging |
| First Run | `first_run.rs` | Initial setup and wrapper installation |
| Elevation | `elevation.rs` | Sudo/admin privilege handling |

---

## Frontend (Svelte 5)

### Component Structure

```
src/
├── App.svelte                    # Main application shell
├── main.ts                       # Entry point
└── lib/
    ├── components/
    │   ├── AgentList.svelte      # Agent sidebar with controls
    │   ├── AgentView.svelte      # Main agent display with output
    │   ├── ChatView.svelte       # Meta-agent chat interface
    │   ├── NewAgentDialog.svelte # Agent/pipeline creation
    │   ├── PhaseProgress.svelte  # Pipeline visualization
    │   ├── AgentSettings.svelte  # Pipeline settings UI
    │   ├── PoolDashboard.svelte  # Agent pool metrics
    │   ├── CostTracker.svelte    # Cost tracking interface
    │   ├── ToolActivity.svelte   # Tool execution tracking
    │   ├── OutputControls.svelte # Output search and export
    │   ├── InstructionWizard.svelte  # Instruction file wizard
    │   ├── chat/
    │   │   └── MetaTaskProgress.svelte  # Task progress display
    │   ├── voice/
    │   │   ├── VoiceSidebar.svelte      # Main voice interface
    │   │   ├── DiscussMode.svelte       # Standalone discuss component
    │   │   └── AttentionOverlay.svelte  # Notification overlay
    │   ├── instruction-wizard/
    │   │   ├── GoalInput.svelte         # Step 1: Goal input
    │   │   ├── DraftPreview.svelte      # Step 2: Draft review
    │   │   ├── TestRunner.svelte        # Step 4: Live testing
    │   │   └── TestResults.svelte       # Test analysis display
    │   └── wizard/
    │       ├── WizardContainer.svelte   # Dialog wrapper
    │       ├── WizardStep.svelte        # Step renderer
    │       ├── WizardHeader.svelte      # Step header
    │       └── WizardNavigation.svelte  # Navigation buttons
    ├── stores/
    │   ├── agents.ts             # Agent state and actions
    │   ├── pipelines.ts          # Pipeline state and progress
    │   ├── pipelineSettings.ts   # Pipeline configuration
    │   ├── costTracking.ts       # Cost data management
    │   ├── activity.ts           # Activity timeline
    │   ├── security.ts           # Security monitoring state
    │   ├── voice.ts              # Voice state (3 modes)
    │   └── metaTodos.ts          # Meta-agent task tracking
    └── types.ts                  # TypeScript type definitions
```

### State Management

Claude Commander uses Svelte 5 runes for reactive state:

```typescript
// Example from stores/agents.ts
let agents = $state<Agent[]>([]);
let selectedAgentId = $state<string | null>(null);

// Derived state
let selectedAgent = $derived(
  agents.find(a => a.id === selectedAgentId)
);
```

---

## Hook Server

The hook server listens on port **19832** for Claude Code tool events.

### Event Types

| Event | Timing | Data |
|-------|--------|------|
| `PreToolUse` | Before tool execution | Tool name, arguments, timestamp |
| `PostToolUse` | After tool execution | Result, execution time, success/failure |

### Event ID Format

```
{agent_id}_{session_id}_{tool_name}_{timestamp}
```

### Configuration

Claude Code agents are configured to post events:

```bash
claude-agent-manager --hook-url http://localhost:19832/hook
```

---

## Data Storage

### SQLite Database

Agent run history is persisted in SQLite:

- **Location**: `~/.local/share/claude-commander/agent_runs.db`
- **Contents**: Run history, session metadata, tool events

### Cost History

API costs are stored in JSON:

| Platform | Location |
|----------|----------|
| Linux/macOS | `~/.local/share/claude-commander/cost_history.json` |
| Windows | `%LOCALAPPDATA%\claude-commander\cost_history.json` |

---

## Project Structure

```
claude-commander/
├── src/                           # Frontend source
│   ├── lib/
│   │   ├── components/            # Svelte components
│   │   │   ├── voice/             # Voice UI components
│   │   │   ├── chat/              # Chat view components
│   │   │   ├── wizard/            # Reusable wizard components
│   │   │   └── instruction-wizard/ # Instruction wizard steps
│   │   ├── stores/                # State management
│   │   │   ├── voice.ts           # Voice state (3 modes)
│   │   │   └── metaTodos.ts       # Meta-agent tasks
│   │   ├── hooks/                 # Svelte hooks
│   │   └── types.ts               # TypeScript types
│   ├── App.svelte                 # Main app component
│   └── main.ts                    # Entry point
├── src-tauri/                     # Backend source
│   ├── src/
│   │   ├── agent_manager/         # Agent lifecycle (modular)
│   │   ├── meta_agent/            # Meta-agent orchestration
│   │   │   └── tools/             # Meta-agent tools
│   │   ├── security_monitor/      # Security monitoring
│   │   ├── agent_runs_db/         # SQLite persistence
│   │   ├── voice/                 # Voice/Realtime API
│   │   ├── hook_server/           # Hook server
│   │   ├── ai_client/             # AI client
│   │   ├── commands/              # Tauri command handlers
│   │   ├── auto_pipeline/         # Pipeline orchestration
│   │   ├── utils/                 # Shared utilities
│   │   ├── tool_registry.rs       # Tool definitions
│   │   ├── types.rs               # Type definitions
│   │   ├── error.rs               # Error types
│   │   ├── db_utils.rs            # Database utilities
│   │   └── lib.rs                 # Tauri commands
│   └── Cargo.toml
├── package.json
└── vite.config.ts
```

---

## Dependencies

### Rust (Backend)

| Crate | Version | Purpose |
|-------|---------|---------|
| tauri | v2 | Desktop app framework |
| tokio | full | Async runtime |
| axum | 0.7 | Web framework (hook server) |
| reqwest | 0.12 | HTTP client |
| rusqlite | 0.32 | SQLite database |
| serde | — | Serialization |
| chrono | — | Date/time handling |
| uuid | v4 | ID generation |
| regex | — | Pattern matching |

### JavaScript (Frontend)

| Package | Purpose |
|---------|---------|
| svelte | UI framework |
| typescript | Type safety |
| vite | Build tool |
| @tauri-apps/api | Tauri integration |
| dompurify | XSS protection |
| marked | Markdown rendering |
| highlight.js | Code highlighting |

---

## See Also

- [CONFIGURATION.md](./CONFIGURATION.md) — Pipeline and environment settings
- [CONTRIBUTING.md](./CONTRIBUTING.md) — Development setup and guidelines
- [BUILD.md](./BUILD.md) — Build instructions for all platforms
