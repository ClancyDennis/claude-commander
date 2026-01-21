#!/bin/bash
# Build script for Claude Commander
# Usage: ./scripts/build.sh [command] [options]
#
# Commands:
#   local         Build for current platform (default)
#   docker        Build Linux version using Docker
#   docker-arm64  Build Linux ARM64 using Docker
#   release       Build optimized release for current platform
#   all           Build all supported platforms (requires native runners)
#   clean         Clean build artifacts
#   help          Show this help message

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_dependencies() {
    local missing=()

    if ! command -v node &> /dev/null; then
        missing+=("node")
    fi

    if ! command -v npm &> /dev/null; then
        missing+=("npm")
    fi

    if ! command -v cargo &> /dev/null; then
        missing+=("cargo (Rust)")
    fi

    if [ ${#missing[@]} -ne 0 ]; then
        log_error "Missing dependencies: ${missing[*]}"
        echo "Please install the missing dependencies and try again."
        exit 1
    fi
}

check_docker() {
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed. Please install Docker to use container builds."
        exit 1
    fi

    if ! docker info &> /dev/null; then
        log_error "Docker daemon is not running. Please start Docker."
        exit 1
    fi
}

build_local() {
    log_info "Building for current platform..."
    check_dependencies

    log_info "Installing npm dependencies..."
    npm ci

    log_info "Building frontend..."
    npm run build

    log_info "Building Tauri application..."
    npm run tauri build

    log_success "Build complete! Artifacts are in src-tauri/target/release/bundle/"
}

build_release() {
    log_info "Building optimized release..."
    check_dependencies

    log_info "Installing npm dependencies..."
    npm ci

    log_info "Building frontend (production)..."
    npm run build

    log_info "Building Tauri application (release)..."
    npm run tauri build

    log_success "Release build complete!"
    log_info "Artifacts location: src-tauri/target/release/bundle/"

    # Show built files
    if [ -d "src-tauri/target/release/bundle" ]; then
        echo ""
        log_info "Built artifacts:"
        find src-tauri/target/release/bundle -type f \( -name "*.AppImage" -o -name "*.deb" -o -name "*.dmg" -o -name "*.exe" -o -name "*.msi" \) 2>/dev/null | while read -r file; do
            echo "  - $file"
        done
    fi
}

build_docker() {
    log_info "Building Linux version using Docker..."
    check_docker

    log_info "Building Docker image and compiling..."
    docker build -f scripts/Dockerfile.linux -t claude-commander-build .

    log_info "Extracting artifacts..."
    mkdir -p dist-docker

    # Create container, copy artifacts, remove container
    CONTAINER_ID=$(docker create claude-commander-build)
    docker cp "$CONTAINER_ID:/app/src-tauri/target/release/bundle" dist-docker/ 2>/dev/null || true
    docker rm "$CONTAINER_ID"

    log_success "Docker build complete! Artifacts are in dist-docker/"
}

build_docker_arm64() {
    log_info "Building Linux ARM64 version using Docker..."
    check_docker

    # Check if buildx is available for true ARM64 builds
    if docker buildx version &> /dev/null; then
        log_info "Using Docker buildx for native ARM64 build..."
        docker buildx build \
            --platform linux/arm64 \
            -f scripts/Dockerfile.linux \
            -t claude-commander-build-arm64 \
            --load \
            .
    else
        log_warn "Docker buildx not available. Using cross-compilation (limited support)."
        docker build -f scripts/Dockerfile.linux-arm64 -t claude-commander-build-arm64 .
    fi

    log_info "Extracting artifacts..."
    mkdir -p dist-docker-arm64

    CONTAINER_ID=$(docker create claude-commander-build-arm64)
    docker cp "$CONTAINER_ID:/app/src-tauri/target/release/bundle" dist-docker-arm64/ 2>/dev/null || \
    docker cp "$CONTAINER_ID:/app/src-tauri/target/aarch64-unknown-linux-gnu/release" dist-docker-arm64/ 2>/dev/null || true
    docker rm "$CONTAINER_ID"

    log_success "ARM64 build complete! Artifacts are in dist-docker-arm64/"
}

clean_build() {
    log_info "Cleaning build artifacts..."

    # Clean Rust build
    if [ -d "src-tauri/target" ]; then
        log_info "Removing src-tauri/target..."
        rm -rf src-tauri/target
    fi

    # Clean frontend build
    if [ -d "dist" ]; then
        log_info "Removing dist/..."
        rm -rf dist
    fi

    # Clean Docker outputs
    for dir in dist-docker dist-docker-arm64 dist-docker-arm64-native; do
        if [ -d "$dir" ]; then
            log_info "Removing $dir/..."
            rm -rf "$dir"
        fi
    done

    # Clean node_modules (optional, commented out by default)
    # rm -rf node_modules

    log_success "Clean complete!"
}

show_help() {
    cat << EOF
Claude Commander Build Script

Usage: ./scripts/build.sh [command] [options]

Commands:
  local         Build for current platform (default)
  release       Build optimized release for current platform
  docker        Build Linux x86_64 version using Docker
  docker-arm64  Build Linux ARM64 using Docker (uses buildx if available)
  clean         Clean all build artifacts
  help          Show this help message

Examples:
  ./scripts/build.sh                  # Build for current platform
  ./scripts/build.sh release          # Build optimized release
  ./scripts/build.sh docker           # Build Linux in Docker container
  ./scripts/build.sh docker-arm64     # Build Linux ARM64 in Docker
  ./scripts/build.sh clean            # Clean all artifacts

Platform-specific notes:
  - macOS: Builds .dmg and .app bundle
  - Windows: Builds .exe and .msi installer
  - Linux: Builds .AppImage and .deb package

For CI/CD builds, see .github/workflows/release.yml

EOF
}

# Main command handler
case "${1:-local}" in
    local)
        build_local
        ;;
    release)
        build_release
        ;;
    docker)
        build_docker
        ;;
    docker-arm64)
        build_docker_arm64
        ;;
    clean)
        clean_build
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        log_error "Unknown command: $1"
        show_help
        exit 1
        ;;
esac
