// Claude CLI discovery and elevation path utilities
//
// This module handles finding the Claude CLI binary in various installation
// locations and managing the elevation-bin path for sudo wrapper functionality.

/// Attempt to find the Claude CLI in common installation locations
pub(crate) fn find_claude_cli() -> Result<String, std::env::VarError> {
    #[cfg(windows)]
    {
        // Check npm global install location on Windows
        if let Ok(appdata) = std::env::var("APPDATA") {
            let npm_path = std::path::PathBuf::from(&appdata)
                .join("npm")
                .join("claude.cmd");
            if npm_path.exists() {
                return Ok(npm_path.to_string_lossy().to_string());
            }
        }
        // Check Program Files
        let program_files =
            std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());
        let pf_path = std::path::PathBuf::from(&program_files)
            .join("nodejs")
            .join("claude.cmd");
        if pf_path.exists() {
            return Ok(pf_path.to_string_lossy().to_string());
        }
    }

    #[cfg(not(windows))]
    {
        // Check common Unix locations
        if let Ok(home) = std::env::var("HOME") {
            // Check ~/.local/bin (common for user installs)
            let local_bin = std::path::PathBuf::from(&home).join(".local/bin/claude");
            if local_bin.exists() {
                return Ok(local_bin.to_string_lossy().to_string());
            }

            // Check nvm locations (try to find any node version)
            let nvm_dir = std::path::PathBuf::from(&home).join(".nvm/versions/node");
            if nvm_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&nvm_dir) {
                    for entry in entries.flatten() {
                        let claude_path = entry.path().join("bin/claude");
                        if claude_path.exists() {
                            return Ok(claude_path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        // Check /usr/local/bin
        let usr_local = std::path::PathBuf::from("/usr/local/bin/claude");
        if usr_local.exists() {
            return Ok(usr_local.to_string_lossy().to_string());
        }
    }

    Err(std::env::VarError::NotPresent)
}

/// Get the path to the elevation-bin directory for the current platform
pub(crate) fn get_elevation_bin_path() -> Option<std::path::PathBuf> {
    // First try the app's data directory (where elevation scripts are copied on startup)
    if let Some(data_dir) = dirs::data_local_dir() {
        let app_elevation_dir = data_dir.join("claude-commander").join("elevation-bin");

        #[cfg(target_os = "linux")]
        let platform_dir = app_elevation_dir.join("linux");

        #[cfg(target_os = "macos")]
        let platform_dir = app_elevation_dir.join("macos");

        #[cfg(target_os = "windows")]
        let platform_dir = app_elevation_dir.join("windows");

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        let platform_dir = app_elevation_dir.join("linux"); // Fallback

        if platform_dir.exists() {
            return Some(platform_dir);
        }
    }

    // Fallback: try relative to executable (development mode)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            #[cfg(target_os = "linux")]
            let dev_dir = exe_dir.join("elevation-bin").join("linux");

            #[cfg(target_os = "macos")]
            let dev_dir = exe_dir.join("elevation-bin").join("macos");

            #[cfg(target_os = "windows")]
            let dev_dir = exe_dir.join("elevation-bin").join("windows");

            #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
            let dev_dir = exe_dir.join("elevation-bin").join("linux");

            if dev_dir.exists() {
                return Some(dev_dir);
            }
        }
    }

    None
}
