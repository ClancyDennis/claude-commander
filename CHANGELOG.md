# Changelog

All notable changes to Claude Commander will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Cross-compilation scripts for release builds
- GitHub Actions workflows for CI and multi-platform releases
- Docker support for local Linux builds
- Comprehensive build documentation (BUILD.md)
- GitHub issue templates and security policy
- Dependabot configuration for automated dependency updates

## [0.2.1] - 2026-01-25

### Added

#### Voice Mode Features (Beta)
- **Dictate mode** — Speech-to-text transcription using OpenAI Realtime API
- **Discuss mode** — Bidirectional voice conversation with AI tool access
- **Attention mode** — Auto-closing task completion notifications (10s timeout)
- Voice sidebar with mode selection (Dictate vs Discuss)
- Audio playback controls (mute/unmute)
- Message selection in Discuss mode for chat integration

#### Instruction Wizard
- AI-assisted instruction file generation from natural language goals
- 4-step workflow: Describe Goal → Review Draft → Edit → Test
- Live agent testing with real-time output streaming
- Automated test analysis identifying missing tools, auth issues, permissions
- AI-powered instruction enhancement based on test findings

#### Meta-Agent Task Progress
- Visual task orchestration display in chat view
- Real-time progress bar with completion percentage
- Current task highlighting with status indicators
- Pending/In-Progress/Completed state tracking

### Changed
- Refactored meta-agent into modular components:
  - `conversation_manager.rs` — History management with image support
  - `tool_loop_engine.rs` — Iterative tool execution with safety limits
  - `todo_tools.rs` — Task orchestration updates
- Refactored `agent_runs_db` into `crud.rs` and `queries.rs` modules
- Refactored security monitor with builder pattern and preset configurations
- Centralized error handling with structured `AppError` types
- New `db_utils.rs` for cleaner database operations
- Improved code organization with extracted helper modules

### Security
- Enhanced LLM-based threat analysis with dedicated prompts module
- Added security monitor presets: default, strict, human-review
- Expectation-based anomaly detection for suspicious tool usage

## [0.1.0] - 2024-XX-XX

### Added
- Initial release
- Multi-workspace agent hub for running Claude Code agents
- Real-time monitoring of tool usage and agent activity
- Persistent session history and logs
- Ralphline (4-phase pipeline) workflow system
  - Phase 1: Planning with human review checkpoint
  - Phase 2: Implementation with parallel execution
  - Phase 3: Best-of-N verification with multiple strategies
  - Phase 4: Final review and approval
- Cost tracking with persistent history
  - Per-model and per-project breakdowns
  - Daily/monthly analytics
- Meta-agent integration (Anthropic and OpenAI support)
- Prompt injection monitoring for safety
- Agent pool management (P-Thread)
- Task orchestration (B-Thread)
- Verification engine (F-Thread)
- Hook server for Claude Code integration (port 19832)

### Security
- Active prompt injection monitoring
- Human checkpoints for high-impact operations
- Local-first data storage (no telemetry)

[Unreleased]: https://github.com/ClancyDennis/claude-commander/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/ClancyDennis/claude-commander/releases/tag/v0.1.0
