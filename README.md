# Claude Agent Manager

A desktop application for managing multiple Claude Code agents with real-time monitoring, tool tracking, and output management.

## Features

### Phase 1: Stats & Enhanced Parsing
- Real-time agent statistics tracking
- Token usage monitoring
- Enhanced JSON parsing with metadata extraction
- Activity timeline visualization

### Phase 2: GitHub Integration  
- Browse and select GitHub repositories
- Clone repositories directly into agent workspaces
- Repository metadata display

### Phase 3: Tool Call Enhancement
- Real-time tool execution tracking with timing
- Status tracking (pending, success, failed)
- Pre/Post tool call matching
- Execution time measurement
- Advanced filtering by status, tool name, and search
- Tool usage statistics and aggregation

### Phase 4: Output Management
- Search and filter agent outputs
- Multi-format export (JSON, Markdown, HTML, Plain Text)
- Export customization with metadata options
- Real-time output statistics

## Tech Stack

- **Frontend**: Svelte 5 + TypeScript + Vite
- **Backend**: Rust + Tauri v2
- **AI Integration**: Claude Sonnet 4.5 via Anthropic API
- **Hook Server**: Custom HTTP server for Claude Code tool hooks

## Prerequisites

- Node.js 18+ and npm
- Rust 1.70+ and Cargo
- Anthropic API key (for meta-agent chat features)

## Installation

1. Clone the repository:
```bash
git clone <your-repo-url>
cd tauri_server
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

4. Run the development server:
```bash
npm run tauri dev
```

## Building for Production

```bash
npm run tauri build
```

The built application will be in `src-tauri/target/release`.

## Architecture

### Backend (Rust)
- `agent_manager.rs`: Core agent lifecycle management
- `hook_server.rs`: HTTP server for Claude Code hooks (Phase 3 enhancements)
- `meta_agent.rs`: Meta-agent for natural language control
- `ai_client.rs`: Unified AI client supporting Claude and OpenAI
- `github.rs`: GitHub API integration
- `types.rs`: Shared type definitions

### Frontend (Svelte)
- `AgentView.svelte`: Main agent display with output management
- `ToolActivity.svelte`: Tool execution tracking with Phase 3 enhancements
- `OutputControls.svelte`: Phase 4 search and filter controls
- `ExportDialog.svelte`: Phase 4 multi-format export
- `ChatView.svelte`: Meta-agent chat interface
- `stores/agents.ts`: Centralized state management

## Hook Server

The hook server listens on port 19832 for Claude Code tool events. Configure your Claude Code agents with:

```bash
claude-agent-manager --hook-url http://localhost:19832/hook
```

### Phase 3 Hook Enhancements
- PreToolUse: Captures tool call initiation with timestamp
- PostToolUse: Calculates execution time and determines status
- Unique tool call ID tracking: `{agent_id}_{session_id}_{tool_name}_{timestamp}`

## Development

### Running Tests
```bash
# Rust tests
cd src-tauri && cargo test

# Frontend (if tests are added)
npm test
```

### Code Structure
```
tauri_server/
├── src/                    # Frontend source
│   ├── lib/
│   │   ├── components/     # Svelte components
│   │   ├── stores/         # State management
│   │   └── types.ts        # TypeScript types
│   ├── App.svelte          # Main app component
│   └── main.ts             # Entry point
├── src-tauri/              # Backend source
│   ├── src/
│   │   ├── agent_manager.rs
│   │   ├── hook_server.rs  # Phase 3 enhancements
│   │   ├── meta_agent.rs
│   │   ├── ai_client.rs
│   │   ├── github.rs
│   │   └── types.rs        # Phase 3 & 4 types
│   └── Cargo.toml
└── package.json
```

## Performance Optimizations

- Efficient reactive updates with Svelte 5 runes
- Minimal array spreading and filtering
- Single-pass filter operations
- Conditional rendering based on filter state
- Optimized timestamp formatting

## Known Limitations

- No virtualization for very long output lists (>1000 items may be slow)
- Limited to single Claude Code instance per agent
- Hook server runs on fixed port 19832

## License

[Your License Here]

## Contributing

[Your Contributing Guidelines Here]
