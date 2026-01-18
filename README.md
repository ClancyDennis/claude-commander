# Claude Commander

Mission control for Claude Code agents. A desktop application for AI-native agent orchestration with pipelines, cost tracking, and multi-agent workflows.

## Overview

Claude Commander is a comprehensive platform for:
- **Running multiple Claude Code agents** in parallel workspaces
- **Pipeline workflows** with 4-phase development (Planning → Implementation → Testing → Review)
- **Cost tracking** with persistent analytics across sessions
- **Multi-agent orchestration** with task decomposition and verification
- **Real-time monitoring** of agent activity, tools, and outputs

## Key Features

### 🔄 Pipeline System (4-Phase Workflow)

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
- Human approval checkpoint
- Final validation before completion

### 💰 Cost Tracking

Comprehensive API cost monitoring:
- Persistent cost history across app restarts
- Per-model and per-project cost breakdowns
- Daily/monthly analytics with visualizations
- Real-time cost monitoring
- **Keyboard shortcut**: `Ctrl+Shift+$` (or `Cmd+Shift+$` on Mac)

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

### 📊 Real-Time Monitoring

**Phase 1-4 Features** (Original Core Features):
- **Phase 1**: Real-time statistics and token tracking
- **Phase 2**: GitHub repository integration and cloning
- **Phase 3**: Tool execution tracking with timing and status
- **Phase 4**: Output management with search, filter, and multi-format export

### 🎨 Modern UI

- Dark theme design
- Multiple layout modes (single, split, grid)
- Toast notifications for key events
- Pipeline progress visualization
- Pool dashboard with metrics
- Cost tracking interface
- Settings panels for configuration

## Installation

### Prerequisites

- Node.js 18+ and npm
- Rust 1.70+ and Cargo
- Anthropic API key (for meta-agent features)

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
# Edit .env and add your ANTHROPIC_API_KEY
```

4. Run development server:
```bash
npm run tauri dev
```

## Building for Production

```bash
npm run tauri build
```

The built application will be in `src-tauri/target/release`.

## Usage

### Creating a Pipeline

1. Click "New Agent" button (or press `Ctrl+N`)
2. Select **"Pipeline"** creation type
3. Choose working directory
4. Optionally configure pipeline settings (gear icon):
   - Enable/disable verification
   - Set checkpoint requirements
   - Configure agent pool
   - Set orchestration options
5. Click "Create Pipeline"

### Pipeline Flow Visualization

```
User Request
    ↓
Phase 1: Planning
    ├─ Meta-agent creates execution plan
    └─ ✋ Human review checkpoint
    ↓
Phase 2: Implementation
    ├─ Task decomposition (B-Thread)
    ├─ Parallel execution (P-Thread)
    └─ ✓ Automatic validation
    ↓
Phase 3: Testing
    ├─ Best-of-N verification (F-Thread)
    └─ ✓ Consensus from multiple agents
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
- Linux/macOS: `~/.grove_agent_manager/cost_history.json`
- Windows: `%USERPROFILE%\.grove_agent_manager\cost_history.json`

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

### Pipeline Settings

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

### Safety Defaults

All expensive features are **disabled by default** for safety:
- Pipeline system: OFF
- Agent pool: OFF
- Orchestration: OFF
- Verification: OFF (N=1 if enabled)
- Checkpoints: ON (human gates enabled)

See [PIPELINE_SETTINGS_GUIDE.md](PIPELINE_SETTINGS_GUIDE.md) for configuration details.

## Documentation

Comprehensive guides available:

- **[PIPELINE_ARCHITECTURE.md](PIPELINE_ARCHITECTURE.md)** - Complete pipeline system design
- **[PIPELINE_SETTINGS_GUIDE.md](PIPELINE_SETTINGS_GUIDE.md)** - Configuration and testing procedures
- **[COST_TRACKING_GUIDE.md](COST_TRACKING_GUIDE.md)** - Cost tracking implementation details
- **[AI_NATIVE_FEATURES.md](AI_NATIVE_FEATURES.md)** - Advanced AI pattern documentation
- **[MODEL_SELECTION.md](MODEL_SELECTION.md)** - Model configuration guide
- **[LOGGING.md](LOGGING.md)** - Logging system documentation
- **[QUICK_START.md](QUICK_START.md)** - Getting started guide
- **[IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md)** - Implementation timeline

## Known Limitations

- Agent pool requires careful configuration to avoid resource exhaustion
- Pipeline features are experimental - test incrementally
- No virtualization for very long output lists (>1000 items may cause slowdown)
- Hook server runs on fixed port 19832
- Cost data relies on API responses including `total_cost_usd` field

## Performance Optimizations

- Efficient reactive updates with Svelte 5 runes
- Minimal array operations and filtering
- Single-pass data transformations
- Conditional rendering based on state
- Optimized timestamp formatting
- Agent pool auto-scaling

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

## Additional Tools

### OneDrive PDF Connector

A Python library for listing and downloading PDF files from Microsoft OneDrive, located in the `one_drive_connector/` directory.

**Features:**
- OAuth 2.0 device code flow authentication
- Persistent token caching
- Recursive PDF search
- Batch downloads with progress tracking
- Comprehensive error handling

**Quick Start:**
```bash
cd one_drive_connector
pip install -r requirements.txt
cp .env.example .env
# Edit .env and add your Azure AD client ID
python example.py list-pdfs
```

See [one_drive_connector/README.md](one_drive_connector/README.md) for complete documentation.

## Project History

Claude Commander is a comprehensive AI-native orchestration platform featuring:

- Multi-phase pipeline workflows
- Advanced agent patterns (P/B/F/C threads)
- Cost tracking and analytics
- Meta-agent coordination via System Commander

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions welcome! Please open an issue first to discuss what you'd like to change.

---

**Built with Claude Sonnet 4.5** | **Powered by Tauri + Svelte 5** | **AI-Native Architecture**
