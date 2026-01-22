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
| Agent Manager | `agent_manager.rs` | Agent lifecycle (create, start, stop, monitor) |
| Meta Agent | `meta_agent.rs` | LLM-based supervisor that orchestrates agents |
| Hook Server | `hook_server.rs` | HTTP server receiving Claude Code tool events |
| AI Client | `ai_client.rs` | Unified client for Claude + OpenAI APIs |
| Types | `types.rs` | Shared type definitions |
| Tool Registry | `tool_registry.rs` | Meta-agent tool definitions |

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

### Security Monitoring

| Module | File | Purpose |
|--------|------|---------|
| Security Monitor | `security_monitor/` | Prompt injection detection |

The security system uses a hybrid approach:
1. **Fast regex patterns**: Known threat signatures
2. **LLM semantic analysis**: Sophisticated attack detection
3. **Session expectations**: Tracks expected vs. actual tool calls

### Supporting Systems

| Module | File | Purpose |
|--------|------|---------|
| Cost Tracker | `cost_tracker.rs` | API cost tracking and persistence |
| Instruction Manager | `instruction_manager.rs` | Markdown instruction handling |
| Skill Generator | `skill_generator.rs` | Auto-generate skills from instructions |
| Subagent Generator | `subagent_generator.rs` | Auto-generate sub-agents |
| Claude.md Generator | `claudemd_generator.rs` | Task-specific context generation |
| Agent Runs DB | `agent_runs_db.rs` | SQLite persistence for run history |
| GitHub Integration | `github.rs` | GitHub API integration |

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
    │   └── OutputControls.svelte # Output search and export
    ├── stores/
    │   ├── agents.ts             # Agent state and actions
    │   ├── pipelines.ts          # Pipeline state and progress
    │   ├── pipelineSettings.ts   # Pipeline configuration
    │   ├── costTracking.ts       # Cost data management
    │   ├── activity.ts           # Activity timeline
    │   └── security.ts           # Security monitoring state
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
│   │   ├── stores/                # State management
│   │   └── types.ts               # TypeScript types
│   ├── App.svelte                 # Main app component
│   └── main.ts                    # Entry point
├── src-tauri/                     # Backend source
│   ├── src/
│   │   ├── agent_manager.rs       # Agent lifecycle
│   │   ├── pipeline_manager.rs    # Pipeline orchestration
│   │   ├── orchestrator.rs        # Task decomposition
│   │   ├── agent_pool.rs          # Agent pool
│   │   ├── verification_engine.rs # Verification
│   │   ├── meta_agent.rs          # Meta-agent
│   │   ├── security_monitor/      # Security monitoring
│   │   ├── cost_tracker.rs        # Cost tracking
│   │   ├── hook_server.rs         # Hook server
│   │   ├── ai_client.rs           # AI client
│   │   ├── github.rs              # GitHub integration
│   │   ├── tool_registry.rs       # Tool definitions
│   │   ├── types.rs               # Type definitions
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
