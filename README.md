# Claude Commander

A desktop “mission control” for running **multiple Claude Code agents** across directories without drowning in permission popups, juggling a dozen VS Code windows, or losing track of what each agent was doing.

Claude Commander centralizes agent workflows into one place: pick a directory, give it a task (via Markdown instructions), monitor tool activity and outputs in real time, and review everything later—optionally with multi-phase pipelines that help carry work through to completion with fewer interruptions.

---

## Screenshots

> Add screenshots/GIFs here (high impact for a desktop app):
> - `/assets/pipeline-overview.png`
> - `/assets/agent-grid.png`
> - `/assets/cost-tracker.png`
>
> A short GIF of “Create Pipeline → Approve Plan → Run → Verify → Complete” is ideal.

---

## Why I built this

As I started trusting Claude Code more, the main bottleneck became **me**:

- constantly mashing “Allow” prompts for tool access, and
- getting lost with **10–12 parallel Claude Code sessions** spread across multiple VS Code windows.

When I’d hit rate limits or step away, I’d come back and forget:
- which instance was working on what,
- which repo/directory it was in,
- what changes it was trying to make,
- and where the “good run” even lived.

Claude Commander exists to remove that friction:
- **Centralize** many agents + workspaces in one UI
- Reduce permission/interaction overhead (without enabling everything globally)
- Make runs **observable** (tools, timing, outputs)
- Make runs **reviewable** (logs you can come back to)

---

## What it does

- **Multi-workspace agent hub**: run multiple Claude Code agents across different directories from one place
- **Instruction-driven workflow**: select a directory → provide a Markdown instruction/task → monitor progress
- **Real-time monitoring**: track tool usage, timing, and status
- **Persistent history**: logs + session artifacts so you can review what happened later
- **Optional pipelines**: structured multi-phase workflows with checkpoints, validation, and verification
- **Cost tracking**: persistent spend analytics by model/project/session

---

## LLM orchestration + safety monitoring (Anthropic or OpenAI)

Claude Commander can use either an **Anthropic** or an **OpenAI** model as a supervising “system commander” LLM that:

- Launches Claude Code agents in selected directories
- Reads tool events + agent output
- Decides next actions (iterate, spawn another agent, validate, or request review)
- Routes work through optional pipelines with checkpoints

To reduce risk when agents are consuming untrusted text (issues, logs, web content, etc.), Claude Commander also includes an **active prompt-injection monitoring layer** powered by a **smaller LLM**. This safety monitor scans instructions and outputs for common prompt-injection / policy-override patterns and can flag or block suspicious steps depending on your settings.

> Defaults are conservative: expensive automation is opt-in, and human checkpoints remain available for high-impact phases.

---

## Key features

### ✅ Centralized multi-agent workflow
- One UI to manage many simultaneous Claude Code instances
- Less context switching than juggling many VS Code windows
- Clear visibility into “what’s running where”

### 🔒 Safer, less annoying permissions
- Designed to avoid the “enable everything forever” approach
- Reduce approval fatigue while keeping control

### 🧠 Instruction-driven development
- Add new instructions by creating a **Markdown file**
- Built-in assistant can review/refine instructions
- (Planned) guided helpers for login/setup flows

### 🔄 Pipeline mode (optional)
Inspired by community patterns (e.g., IndieDevDan workflows and popular Claude Code looping / verification approaches), Pipeline mode can run a structured flow like:

### 🔄 Ralphline System (4-Phase Workflow)

Create structured development workflows with automatic progression:

**Phase 1: Planning**
- Meta-agent analyzes request and creates execution plan
- Human review checkpoint for plan approval

**Phase 2: Implementation**
- Automatic task decomposition
- Parallel execution with agent pool
- Automatic validation (e.g., `cargo check`)

**Phase 3: Testing**
- Best-of-N verification with multiple agents
- Consensus strategies (majority, weighted, meta-agent, first-correct)
- Confidence scoring

**Phase 4: Final Review**
- LLM approval checkpoint
- Final validation before completion 
- loop if not up to spec

### 💰 Cost Tracking

Comprehensive API cost monitoring:
- Persistent cost history across app restarts
- Per-model and per-project cost breakdowns
- Daily/monthly analytics with visualizations
- Real-time cost monitoring
- **Keyboard shortcut**: `Ctrl+Shift+$` (or `Cmd+Shift+$` on Mac)





## Installation

### Prerequisites

- Node.js 18+ and npm
- Rust 1.70+ and Cargo
- Anthropic API key (for meta-agent features, can login claude code via web auth)

### Steps

1. Clone the repository:
```bash
git clone https://github.com/ClancyDennis/claude-commander.git
cd claude-commander
```

2. Install dependencies:
```bash
npm install
```

3. Set up environment variables:
```bash
cp .env.example .env
# Edit .env and add your OPENAI_API_KEY, set models
```

4. Run development server:
```bash
npm run tauri dev
```

### Troubleshooting

**Windows Installation Issues**

If you encounter errors like `Cannot find module ...` or `npm has a bug related to optional dependencies` when running `npm install` or `npm run build`, it may be due to a known npm bug on Windows where platform-specific binaries for optional dependencies (like `@tauri-apps/cli` and `@rollup/rollup`) are skipped.

To fix this:
1.  Explicitly install the missing Windows binaries as dev dependencies:
    ```bash
    npm install --save-dev --save-exact @tauri-apps/cli-win32-x64-msvc @rollup/rollup-win32-x64-msvc
    ```
2.  Clean the cache and reinstall:
    ```bash
    npm cache clean --force
    rm -rf node_modules package-lock.json
    npm install
    ```

**Missing Icon Error**

If you see an error like `` `icons/icon.ico` not found ``, you need to generate the platform-specific icons from the source image:

```bash
npm run tauri icon src-tauri/icons/icon.png
```

## Building for Production

```bash
npm run tauri build
```

The built application will be in `src-tauri/target/release`.

## Usage

### Creating a Ralphline (INspired by https://ghuntley.com/ralph/)

1. Click "New Agent" button (or press `Ctrl+N`)
2. Select **"Ralphline"** creation type
3. Choose working directory
4. Give the Ralphline instructions
5. Click "Create Pipeline"

### Ralphline Flow Visualization

```
User Request
    ↓
Phase 1: Planning
    ├─ Agent creates execution plan
    └─ Orchestrator reviews and replans or begins
    ↓
Phase 2: Implementation
    ├─ Task decomposition (Claude code subagents)
    ├─ Parallel execution
    ↓
Phase 3: Validation
    └─ ✓ Automatic 
    ↓
Phase 4: Final Review
    └─ ✋ Human approval
    ↓
✅ Completed Pipeline
```

### Viewing Cost Data

**Access Methods:**
- Press `Ctrl+Shift+$` (or `Cmd+Shift+$` on Mac)
- Click "💰 Costs" button in agent list footer

**Features:**
- **Overview Tab**: Total costs, today, this month, by model, by project
- **Sessions Tab**: Detailed session history with date range filtering
- **Analytics Tab**: Daily cost charts and spending trends

**Data Storage:**
- Linux/macOS: `~/.local/share/claude-commander/cost_history.json`
- Windows: `%LOCALAPPDATA%\claude-commander\cost_history.json`

### Creating a Single Agent

For direct Claude Code instances without pipeline overhead:

1. Click "New Agent" button
2. Select **"Single Agent"** creation type
3. Choose working directory
4. Optionally provide GitHub repository URL
5. Click "Create Agent"

## Architecture

### Backend (Rust)

**Core Modules:**
- `agent_manager.rs` - Agent lifecycle management
- `meta_agent.rs` - Meta-agent for natural language control
- `hook_server.rs` - HTTP server for Claude Code hooks

**Pipeline System:**
- `pipeline_manager.rs` - 4-phase pipeline orchestration
- `orchestrator.rs` - Task decomposition (B-Thread)
- `agent_pool.rs` - Agent pool management (P-Thread)
- `verification_engine.rs` - Best-of-N verification (F-Thread)

**Supporting Systems:**
- `cost_tracker.rs` - Cost tracking and persistence
- `ai_client.rs` - Unified AI client (Claude + OpenAI)
- `github.rs` - GitHub API integration
- `types.rs` - Shared type definitions
- `tool_registry.rs` - Meta-agent tool definitions

### Frontend (Svelte 5)

**Core Components:**
- `App.svelte` - Main application shell
- `AgentList.svelte` - Agent sidebar with controls
- `AgentView.svelte` - Main agent display with output
- `ChatView.svelte` - Meta-agent chat interface

**Pipeline Components:**
- `NewAgentDialog.svelte` - Agent/pipeline creation dialog
- `PhaseProgress.svelte` - Pipeline visualization and control
- `AgentSettings.svelte` - Pipeline settings UI

**Monitoring Components:**
- `PoolDashboard.svelte` - Agent pool metrics
- `CostTracker.svelte` - Cost tracking interface
- `ToolActivity.svelte` - Tool execution tracking
- `OutputControls.svelte` - Output search and export

**State Management:**
- `stores/agents.ts` - Agent state and actions
- `stores/pipelines.ts` - Pipeline state and progress
- `stores/pipelineSettings.ts` - Pipeline configuration
- `stores/costTracking.ts` - Cost data management
- `stores/activity.ts` - Activity timeline

## Hook Server

The hook server listens on port 19832 for Claude Code tool events.

Configure your Claude Code agents:
```bash
claude-agent-manager --hook-url http://localhost:19832/hook
```

**Event Types:**
- `PreToolUse` - Tool call initiation with timestamp
- `PostToolUse` - Tool completion with execution time
- Tool call ID tracking: `{agent_id}_{session_id}_{tool_name}_{timestamp}`

## Keyboard Shortcuts

- `Ctrl+N` - New Agent/Pipeline
- `Ctrl+Shift+$` - Toggle Cost Tracker
- `Ctrl+P` - Toggle Pool Dashboard
- `Ctrl+Shift+C` - Open Chat View
- `Escape` - Close dialogs

## Configuration

### Configurable Pipeline Settings (inspired by https://www.youtube.com/watch?v=-WBHNFAB0OE&pp=2AbVCA%3D%3D)
Configure via AgentSettings.svelte (gear icon in agent list):

**Phase A (Agent Pool):**
- Enable shared agent pool
- Set pool priority (low/normal/high)

**Phase B (Orchestration):**
- Enable automatic task decomposition
- Set max parallel tasks

**Phase C (Verification):**
- Enable Best-of-N verification
- Choose strategy (majority/weighted/meta/first)
- Set N (number of agents)
- Set confidence threshold

**Phase D (Checkpoints):**
- Require plan review
- Require final review
- Set auto-validation command
- Auto-approve on verification pass

### 🤖 Advanced Agent Patterns

**P-Thread (Agent Pool)**
- Shared pool for parallel task execution
- Auto-scaling based on load
- Agent acquisition and release
- Pool dashboard with real-time metrics

**B-Thread (Orchestration)**
- Automatic task decomposition
- Dependency management
- Parallel task execution
- Workflow state management

**F-Thread (Verification)**
- Best-of-N verification
- Multiple fusion strategies:
  - Majority vote
  - Weighted consensus
  - Meta-agent review
  - First-correct selection
- Confidence scoring

**C-Thread (Checkpoints)**
- Human review gates
- Automatic validation commands
- BestOfN verification checkpoints
- Conditional routing
## Known Limitations

- Agent pool requires careful configuration to avoid resource exhaustion
- Pipeline features are experimental - test incrementally
- No virtualization for very long output lists (>1000 items may cause slowdown)
- Hook server runs on fixed port 19832
- Cost data relies on API responses including `total_cost_usd` field

## Development

### Running Tests

```bash
# Rust backend tests
cd src-tauri && cargo test

# Frontend tests (if added)
npm test
```

### Project Structure

```
claude-commander/
├── src/                           # Frontend source
│   ├── lib/
│   │   ├── components/            # Svelte components
│   │   │   ├── AgentList.svelte
│   │   │   ├── AgentView.svelte
│   │   │   ├── ChatView.svelte
│   │   │   ├── PhaseProgress.svelte
│   │   │   ├── PoolDashboard.svelte
│   │   │   ├── CostTracker.svelte
│   │   │   └── ...
│   │   ├── stores/                # State management
│   │   │   ├── agents.ts
│   │   │   ├── pipelines.ts
│   │   │   ├── pipelineSettings.ts
│   │   │   ├── costTracking.ts
│   │   │   └── activity.ts
│   │   └── types.ts               # TypeScript types
│   ├── App.svelte                 # Main app component
│   └── main.ts                    # Entry point
├── src-tauri/                     # Backend source
│   ├── src/
│   │   ├── agent_manager.rs       # Agent lifecycle
│   │   ├── pipeline_manager.rs    # Pipeline orchestration
│   │   ├── orchestrator.rs        # Task decomposition (B-Thread)
│   │   ├── agent_pool.rs          # Agent pool (P-Thread)
│   │   ├── verification_engine.rs # Verification (F-Thread)
│   │   ├── meta_agent.rs          # Meta-agent
│   │   ├── cost_tracker.rs        # Cost tracking
│   │   ├── hook_server.rs         # Hook server
│   │   ├── ai_client.rs           # AI client
│   │   ├── github.rs              # GitHub integration
│   │   ├── tool_registry.rs       # Tool definitions
│   │   ├── types.rs               # Type definitions
│   │   └── lib.rs                 # Tauri commands
│   └── Cargo.toml
└── package.json
```

## Tech Stack

- **Frontend**: Svelte 5 + TypeScript + Vite
- **Backend**: Rust + Tauri v2
- **AI Integration**: Claude Sonnet 4.5 via Anthropic API
- **State Management**: Svelte 5 runes ($state, $derived)
- **HTTP Server**: Tokio + Axum for hook server
- **Persistence**: JSON files for cost history


## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions welcome! Please open an issue first to discuss what you'd like to change.

---

**Built with Claude Sonnet 4.5** | **Powered by Tauri + Svelte 5** | **AI-Native Architecture**
