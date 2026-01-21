# Contributing to Claude Commander

Thanks for your interest in contributing! This guide will help you get set up for development.

---

## Prerequisites

Before you start, make sure you have:

- **Node.js 18+** and npm
- **Rust 1.70+** and Cargo
- **Claude Code** installed ([install guide](https://claude.ai/code))
- **API Keys**: Anthropic and/or OpenAI

### Check Your Setup

```bash
node --version    # Should be 18+
npm --version     # Should be 8+
rustc --version   # Should be 1.70+
cargo --version   # Should match rustc
```

---

## Development Setup

### 1. Clone the Repository

```bash
git clone https://github.com/ClancyDennis/claude-commander.git
cd claude-commander
```

### 2. Install Dependencies

```bash
npm install
```

### 3. Configure Environment

```bash
cp .env.example .env
# Edit .env and add your API keys
```

### 4. Run Development Server

```bash
npm run tauri dev
```

This starts both the Vite dev server (frontend) and Tauri (backend).

---

## Project Structure

```
claude-commander/
├── src/                      # Frontend (Svelte + TypeScript)
│   ├── lib/
│   │   ├── components/       # UI components
│   │   ├── stores/           # State management
│   │   └── types.ts          # Type definitions
│   ├── App.svelte
│   └── main.ts
├── src-tauri/                # Backend (Rust)
│   ├── src/
│   │   ├── agent_manager.rs
│   │   ├── meta_agent.rs
│   │   ├── pipeline_manager.rs
│   │   ├── hook_server.rs
│   │   └── ...
│   └── Cargo.toml
├── package.json
└── vite.config.ts
```

---

## Running Tests

### Backend (Rust)

```bash
cd src-tauri
cargo test
```

### Frontend

```bash
npm test
```

### Type Checking

```bash
npm run check
```

---

## Building for Production

### Quick Build

```bash
# Linux/macOS
./scripts/build.sh release

# Windows (PowerShell)
.\scripts\build.ps1 release
```

### Manual Build

```bash
npm run tauri build
```

Output: `src-tauri/target/release/bundle/`

### Docker Build (Linux)

```bash
./scripts/build.sh docker         # x86_64
./scripts/build.sh docker-arm64   # ARM64
```

---

## Code Style

### Rust

- Use `rustfmt` for formatting
- Run `cargo clippy` before committing
- Follow Rust naming conventions

### TypeScript/Svelte

- Use TypeScript strict mode
- Prefer Svelte 5 runes ($state, $derived)
- Keep components focused and small

---

## Making Changes

### 1. Create a Branch

```bash
git checkout -b feature/my-feature
```

### 2. Make Your Changes

- Write clear commit messages
- Add tests for new functionality
- Update documentation if needed

### 3. Test Locally

```bash
# Run the app
npm run tauri dev

# Run tests
cargo test
npm test
```

### 4. Submit a Pull Request

1. Push your branch
2. Open a PR against `main`
3. Describe your changes clearly
4. Link any related issues

---

## Areas for Contribution

### Good First Issues

- Improve error messages
- Add keyboard shortcuts
- Fix UI bugs
- Add documentation

### Feature Ideas

- New verification strategies
- Additional AI provider support
- Import/export functionality
- Plugin system

### Documentation

- Improve existing docs
- Add tutorials
- Create video guides
- Translate to other languages

---

## Troubleshooting

### "Cannot find module" on Windows

```bash
npm install --save-dev --save-exact @tauri-apps/cli-win32-x64-msvc @rollup/rollup-win32-x64-msvc
npm cache clean --force
rm -rf node_modules package-lock.json
npm install
```

### Missing Icons

```bash
npm run tauri icon src-tauri/icons/icon.png
```

### Rust Compilation Errors

```bash
cd src-tauri
cargo clean
cargo build
```

---

## Getting Help

- **Issues**: [GitHub Issues](https://github.com/ClancyDennis/claude-commander/issues)
- **Discussions**: [GitHub Discussions](https://github.com/ClancyDennis/claude-commander/discussions)

---

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

## See Also

- [ARCHITECTURE.md](./ARCHITECTURE.md) — Technical internals
- [CONFIGURATION.md](./CONFIGURATION.md) — Configuration options
- [BUILD.md](./BUILD.md) — Detailed build instructions
