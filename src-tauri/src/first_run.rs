// First-run initialization module
// Handles copying template files to user config directory on first startup

use serde::{Deserialize, Serialize};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

// Embed files at compile time
const ENV_EXAMPLE: &str = include_str!("../../.env.example");
const PLAYWRIGHT_SKILL: &str = include_str!("../../.instructions/PLAYWRIGHT_MCP_SKILL.md");
const GMAIL_INTEGRATION: &str = include_str!("../../.instructions/GMAIL_INTEGRATION.md");
const GOOGLE_DRIVE_INTEGRATION: &str =
    include_str!("../../.instructions/GOOGLE_DRIVE_INTEGRATION.md");

// Embed elevation wrapper scripts at compile time
const LINUX_SUDO_WRAPPER: &str = include_str!("../../elevation-bin/linux/sudo");

#[cfg(target_os = "macos")]
const MACOS_SUDO_WRAPPER: &str = include_str!("../../elevation-bin/macos/sudo");

#[cfg(target_os = "windows")]
const WINDOWS_SUDO_WRAPPER: &str = include_str!("../../elevation-bin/windows/sudo");

#[derive(Debug, Serialize, Deserialize)]
pub struct InitializationMarker {
    pub version: String,
    pub timestamp: String,
    pub files_copied: Vec<String>,
}

pub struct FirstRunConfig {
    pub config_dir: PathBuf,
    pub instructions_dir: PathBuf,
    pub app_version: String,
}

impl FirstRunConfig {
    pub fn new() -> Option<Self> {
        let config_dir = dirs::config_dir()?.join("claude-commander");
        let instructions_dir = dirs::home_dir()?.join(".instructions");

        Some(Self {
            config_dir,
            instructions_dir,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }
}

/// Check if first-run initialization is needed
pub fn needs_initialization(config: &FirstRunConfig) -> bool {
    let marker_path = config.config_dir.join(".initialized");
    !marker_path.exists()
}

/// Perform first-run initialization
pub fn initialize(config: &FirstRunConfig) -> Result<(), String> {
    let mut files_copied = Vec::new();

    // Ensure config directory exists
    fs::create_dir_all(&config.config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    // Ensure instructions directory exists
    fs::create_dir_all(&config.instructions_dir)
        .map_err(|e| format!("Failed to create instructions directory: {}", e))?;

    // Copy env.example to config directory
    let env_example_path = config.config_dir.join("env.example");
    if !env_example_path.exists() {
        fs::write(&env_example_path, ENV_EXAMPLE)
            .map_err(|e| format!("Failed to write env.example: {}", e))?;
        files_copied.push(env_example_path.to_string_lossy().to_string());
        println!("  ✓ Copied env.example to {:?}", env_example_path);
    } else {
        println!("  - env.example already exists, skipping");
    }

    // Copy instruction files to ~/.instructions/
    let instruction_files = [
        ("PLAYWRIGHT_MCP_SKILL.md", PLAYWRIGHT_SKILL),
        ("GMAIL_INTEGRATION.md", GMAIL_INTEGRATION),
        ("GOOGLE_DRIVE_INTEGRATION.md", GOOGLE_DRIVE_INTEGRATION),
    ];

    for (filename, content) in instruction_files {
        let dest_path = config.instructions_dir.join(filename);
        if !dest_path.exists() {
            fs::write(&dest_path, content)
                .map_err(|e| format!("Failed to write {}: {}", filename, e))?;
            files_copied.push(dest_path.to_string_lossy().to_string());
            println!("  ✓ Copied {} to {:?}", filename, dest_path);
        } else {
            println!("  - {} already exists, skipping", filename);
        }
    }

    // Write initialization marker
    let marker = InitializationMarker {
        version: config.app_version.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        files_copied,
    };

    let marker_path = config.config_dir.join(".initialized");
    let marker_json = serde_json::to_string_pretty(&marker)
        .map_err(|e| format!("Failed to serialize marker: {}", e))?;

    fs::write(&marker_path, marker_json)
        .map_err(|e| format!("Failed to write initialization marker: {}", e))?;

    println!("  ✓ Initialization marker written to {:?}", marker_path);

    Ok(())
}

/// Run first-run initialization if needed
pub fn run_if_needed() {
    match FirstRunConfig::new() {
        Some(config) => {
            if needs_initialization(&config) {
                println!("📦 First-run initialization starting...");
                match initialize(&config) {
                    Ok(()) => println!("✓ First-run initialization completed"),
                    Err(e) => eprintln!("⚠ First-run initialization failed: {}", e),
                }
            }
        }
        None => {
            eprintln!("⚠ Could not determine config directories for first-run initialization");
        }
    }

    // Always ensure elevation scripts are installed (they may need updating)
    match install_elevation_scripts() {
        Ok(()) => println!("✓ Elevation scripts ready"),
        Err(e) => eprintln!("⚠ Failed to install elevation scripts: {}", e),
    }
}

/// Install elevation wrapper scripts to app data directory
/// These scripts intercept sudo calls from Claude Code agents
pub fn install_elevation_scripts() -> Result<(), String> {
    let data_dir = dirs::data_local_dir()
        .ok_or("Could not determine data directory")?
        .join("claude-commander")
        .join("elevation-bin");

    // Install Linux scripts
    let linux_dir = data_dir.join("linux");
    fs::create_dir_all(&linux_dir)
        .map_err(|e| format!("Failed to create Linux elevation dir: {}", e))?;

    let linux_sudo = linux_dir.join("sudo");
    fs::write(&linux_sudo, LINUX_SUDO_WRAPPER)
        .map_err(|e| format!("Failed to write Linux sudo wrapper: {}", e))?;

    // Make executable on Unix
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&linux_sudo)
            .map_err(|e| format!("Failed to get metadata: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&linux_sudo, perms)
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }

    // Install macOS scripts
    #[cfg(target_os = "macos")]
    {
        let macos_dir = data_dir.join("macos");
        fs::create_dir_all(&macos_dir)
            .map_err(|e| format!("Failed to create macOS elevation dir: {}", e))?;

        let macos_sudo = macos_dir.join("sudo");
        fs::write(&macos_sudo, MACOS_SUDO_WRAPPER)
            .map_err(|e| format!("Failed to write macOS sudo wrapper: {}", e))?;

        let mut perms = fs::metadata(&macos_sudo)
            .map_err(|e| format!("Failed to get metadata: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&macos_sudo, perms)
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }

    // Install Windows scripts
    #[cfg(target_os = "windows")]
    {
        let windows_dir = data_dir.join("windows");
        fs::create_dir_all(&windows_dir)
            .map_err(|e| format!("Failed to create Windows elevation dir: {}", e))?;

        let windows_sudo = windows_dir.join("sudo");
        fs::write(&windows_sudo, WINDOWS_SUDO_WRAPPER)
            .map_err(|e| format!("Failed to write Windows sudo wrapper: {}", e))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_needs_initialization() {
        let temp = tempdir().unwrap();
        let config = FirstRunConfig {
            config_dir: temp.path().join("config"),
            instructions_dir: temp.path().join("instructions"),
            app_version: "0.1.0".to_string(),
        };

        // Should need initialization (no marker file)
        assert!(needs_initialization(&config));

        // Create the directories and marker
        fs::create_dir_all(&config.config_dir).unwrap();
        fs::write(config.config_dir.join(".initialized"), "{}").unwrap();

        // Should not need initialization anymore
        assert!(!needs_initialization(&config));
    }

    #[test]
    fn test_initialize() {
        let temp = tempdir().unwrap();
        let config = FirstRunConfig {
            config_dir: temp.path().join("config"),
            instructions_dir: temp.path().join("instructions"),
            app_version: "0.1.0".to_string(),
        };

        // Run initialization
        initialize(&config).unwrap();

        // Check files were created
        assert!(config.config_dir.join("env.example").exists());
        assert!(config.config_dir.join(".initialized").exists());
        assert!(config
            .instructions_dir
            .join("PLAYWRIGHT_MCP_SKILL.md")
            .exists());
        assert!(config
            .instructions_dir
            .join("GMAIL_INTEGRATION.md")
            .exists());
        assert!(config
            .instructions_dir
            .join("GOOGLE_DRIVE_INTEGRATION.md")
            .exists());
    }

    #[test]
    fn test_initialize_skips_existing() {
        let temp = tempdir().unwrap();
        let config = FirstRunConfig {
            config_dir: temp.path().join("config"),
            instructions_dir: temp.path().join("instructions"),
            app_version: "0.1.0".to_string(),
        };

        // Create directories
        fs::create_dir_all(&config.config_dir).unwrap();
        fs::create_dir_all(&config.instructions_dir).unwrap();

        // Create an existing file with custom content
        let custom_content = "# My Custom Playwright Guide\n\nCustom content here.";
        fs::write(
            config.instructions_dir.join("PLAYWRIGHT_MCP_SKILL.md"),
            custom_content,
        )
        .unwrap();

        // Run initialization
        initialize(&config).unwrap();

        // Custom content should be preserved
        let content =
            fs::read_to_string(config.instructions_dir.join("PLAYWRIGHT_MCP_SKILL.md")).unwrap();
        assert_eq!(content, custom_content);
    }
}
