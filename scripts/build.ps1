# Build script for Claude Commander (Windows PowerShell)
# Usage: .\scripts\build.ps1 [command]
#
# Commands:
#   local    - Build for current platform (default)
#   release  - Build optimized release
#   clean    - Clean build artifacts
#   help     - Show this help message

param(
    [Parameter(Position=0)]
    [ValidateSet("local", "release", "clean", "help")]
    [string]$Command = "local"
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectDir = Split-Path -Parent $ScriptDir

Set-Location $ProjectDir

function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$Type = "Info"
    )

    switch ($Type) {
        "Info"    { Write-Host "[INFO] $Message" -ForegroundColor Blue }
        "Success" { Write-Host "[SUCCESS] $Message" -ForegroundColor Green }
        "Warning" { Write-Host "[WARN] $Message" -ForegroundColor Yellow }
        "Error"   { Write-Host "[ERROR] $Message" -ForegroundColor Red }
    }
}

function Test-Dependencies {
    $missing = @()

    if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
        $missing += "node"
    }

    if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
        $missing += "npm"
    }

    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        $missing += "cargo (Rust)"
    }

    if ($missing.Count -gt 0) {
        Write-ColorOutput "Missing dependencies: $($missing -join ', ')" "Error"
        Write-Host "Please install the missing dependencies and try again."
        exit 1
    }
}

function Build-Local {
    Write-ColorOutput "Building for Windows..."
    Test-Dependencies

    Write-ColorOutput "Installing npm dependencies..."
    npm ci

    Write-ColorOutput "Building frontend..."
    npm run build

    Write-ColorOutput "Building Tauri application..."
    npm run tauri build

    Write-ColorOutput "Build complete! Artifacts are in src-tauri\target\release\bundle\" "Success"
}

function Build-Release {
    Write-ColorOutput "Building optimized release..."
    Test-Dependencies

    Write-ColorOutput "Installing npm dependencies..."
    npm ci

    Write-ColorOutput "Building frontend (production)..."
    npm run build

    Write-ColorOutput "Building Tauri application (release)..."
    npm run tauri build

    Write-ColorOutput "Release build complete!" "Success"

    # Show built files
    $bundleDir = "src-tauri\target\release\bundle"
    if (Test-Path $bundleDir) {
        Write-ColorOutput "Artifacts location: $bundleDir"
        Write-Host ""
        Write-ColorOutput "Built artifacts:"
        Get-ChildItem -Path $bundleDir -Recurse -Include "*.exe", "*.msi" | ForEach-Object {
            Write-Host "  - $($_.FullName)"
        }
    }
}

function Clean-Build {
    Write-ColorOutput "Cleaning build artifacts..."

    if (Test-Path "src-tauri\target") {
        Write-ColorOutput "Removing src-tauri\target..."
        Remove-Item -Recurse -Force "src-tauri\target"
    }

    if (Test-Path "dist") {
        Write-ColorOutput "Removing dist..."
        Remove-Item -Recurse -Force "dist"
    }

    Write-ColorOutput "Clean complete!" "Success"
}

function Show-Help {
    @"
Claude Commander Build Script (Windows)

Usage: .\scripts\build.ps1 [command]

Commands:
  local    - Build for Windows (default)
  release  - Build optimized release
  clean    - Clean all build artifacts
  help     - Show this help message

Examples:
  .\scripts\build.ps1                # Build for Windows
  .\scripts\build.ps1 release        # Build optimized release
  .\scripts\build.ps1 clean          # Clean all artifacts

Output:
  - .exe executable in src-tauri\target\release\
  - .msi installer in src-tauri\target\release\bundle\msi\

For cross-platform builds, use GitHub Actions or Docker on Linux.
See .github\workflows\release.yml for CI/CD configuration.

"@
}

# Main command handler
switch ($Command) {
    "local"   { Build-Local }
    "release" { Build-Release }
    "clean"   { Clean-Build }
    "help"    { Show-Help }
}
