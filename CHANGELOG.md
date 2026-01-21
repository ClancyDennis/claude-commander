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
