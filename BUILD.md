# Building Claude Commander

This document covers building Claude Commander for various platforms.

## Prerequisites

### All Platforms
- Node.js 20 LTS or later
- npm (comes with Node.js)
- Rust toolchain (install from https://rustup.rs)

### Linux
```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y \
    libwebkit2gtk-4.1-dev \
    libappindicator3-dev \
    librsvg2-dev \
    patchelf \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev
```

### macOS
- Xcode Command Line Tools: `xcode-select --install`

### Windows
- Visual Studio Build Tools with C++ workload
- WebView2 (usually pre-installed on Windows 10/11)

## Quick Build

### Using Build Scripts

**Linux/macOS:**
```bash
./scripts/build.sh            # Build for current platform
./scripts/build.sh release    # Build optimized release
./scripts/build.sh clean      # Clean artifacts
```

**Windows (PowerShell):**
```powershell
.\scripts\build.ps1           # Build for Windows
.\scripts\build.ps1 release   # Build optimized release
.\scripts\build.ps1 clean     # Clean artifacts
```

### Manual Build

```bash
# Install dependencies
npm ci

# Build frontend
npm run build

# Build Tauri application
npm run tauri build
```

## Build Artifacts

After building, artifacts are located in `src-tauri/target/release/bundle/`:

| Platform | Format | Location |
|----------|--------|----------|
| Linux | AppImage | `bundle/appimage/*.AppImage` |
| Linux | Debian | `bundle/deb/*.deb` |
| macOS | DMG | `bundle/dmg/*.dmg` |
| macOS | App | `bundle/macos/*.app` |
| Windows | Installer | `bundle/msi/*.msi` |
| Windows | Executable | `bundle/nsis/*.exe` |

## Docker Builds (Linux)

Build Linux versions using Docker without installing dependencies locally.

### Prerequisites
- Docker installed and running
- (Optional) Docker buildx for ARM64 builds

### Build Commands

**Linux x86_64:**
```bash
./scripts/build.sh docker
# Artifacts in: dist-docker/
```

**Linux ARM64:**
```bash
./scripts/build.sh docker-arm64
# Artifacts in: dist-docker-arm64/
```

### Using Docker Compose

```bash
# Build Linux x86_64
docker compose -f docker-compose.build.yml build linux

# Build Linux ARM64 (native, requires buildx)
docker compose -f docker-compose.build.yml build linux-arm64-native
```

### Manual Docker Build

```bash
# Build the image
docker build -f scripts/Dockerfile.linux -t claude-commander-build .

# Run and extract artifacts
docker run --rm -v $(pwd)/dist-docker:/output claude-commander-build \
    cp -r /app/src-tauri/target/release/bundle /output/
```

## GitHub Actions CI/CD

The project includes GitHub Actions workflows for automated builds.

### Workflows

1. **CI** (`.github/workflows/ci.yml`)
   - Runs on: Push to main/develop, Pull Requests
   - Actions: Lint, type check, Rust checks, build verification
   - Platforms: Linux, Windows, macOS

2. **Release** (`.github/workflows/release.yml`)
   - Runs on: Version tags (`v*`), Manual trigger
   - Actions: Full release builds with artifacts
   - Platforms: Linux (x86_64, ARM64), Windows, macOS (Intel, Apple Silicon)

### Creating a Release

1. **Tag the release:**
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. **Manual trigger:**
   - Go to Actions → Release Build → Run workflow

3. **Artifacts:**
   - Build artifacts are uploaded to GitHub
   - A draft release is created with all binaries

### Code Signing (Optional)

For production releases, add these secrets to your GitHub repository:

**macOS:**
- `APPLE_CERTIFICATE` - Base64 encoded .p12 certificate
- `APPLE_CERTIFICATE_PASSWORD` - Certificate password
- `APPLE_SIGNING_IDENTITY` - Signing identity name
- `APPLE_ID` - Apple ID for notarization
- `APPLE_PASSWORD` - App-specific password
- `APPLE_TEAM_ID` - Team ID

**Windows:**
- `TAURI_SIGNING_PRIVATE_KEY` - Private key for signing
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` - Key password

## Cross-Compilation Notes

### Linux to Linux ARM64

Works well using Docker with buildx:
```bash
docker buildx build --platform linux/arm64 -f scripts/Dockerfile.linux .
```

### Linux to Windows

Not directly supported. Use:
- GitHub Actions (Windows runner)
- Windows VM or machine

### Linux to macOS

Not directly supported. Use:
- GitHub Actions (macOS runner)
- macOS VM (requires Apple hardware)

### macOS Universal Binary

To build a universal binary (Intel + Apple Silicon):
```bash
npm run tauri build -- --target universal-apple-darwin
```

Requires both targets installed:
```bash
rustup target add x86_64-apple-darwin aarch64-apple-darwin
```

## Troubleshooting

### Build fails with missing webkit libraries (Linux)

Install all required dependencies:
```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev
```

### Cargo build takes too long

The first build downloads and compiles all Rust dependencies. Subsequent builds use cached artifacts and are much faster.

### Docker build fails with permission errors

Ensure Docker daemon is running and your user has permissions:
```bash
sudo usermod -aG docker $USER
# Log out and back in
```

### Windows build fails with missing Visual Studio

Install Visual Studio Build Tools:
1. Download from https://visualstudio.microsoft.com/visual-cpp-build-tools/
2. Select "Desktop development with C++" workload

### macOS build fails with code signing

For local development, you can skip signing:
```bash
npm run tauri build -- --no-bundle
```

For distribution, you need an Apple Developer account and valid certificates.
