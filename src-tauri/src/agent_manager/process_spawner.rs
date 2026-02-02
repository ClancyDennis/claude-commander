// Claude process spawning utilities
//
// This module handles creating the hooks configuration and spawning
// the Claude CLI process with the appropriate environment settings.

use std::collections::HashMap;
use std::process::Stdio;
use tokio::process::Command;

use super::claude_cli::{find_claude_cli, get_elevation_bin_path};

/// Environment variables to exclude from Claude Code child processes
/// when CLAUDE_CODE_API_KEY_MODE is set to "blocked".
/// This allows meta agents to use the Anthropic API while Claude Code uses OAuth.
pub(crate) const SENSITIVE_ENV_VARS: &[&str] = &["ANTHROPIC_API_KEY"];

/// Create hooks configuration file for the agent
pub(crate) fn create_hooks_config(
    hook_port: u16,
    agent_id: &str,
) -> Result<std::path::PathBuf, String> {
    let settings_path = std::env::temp_dir().join(format!("claude_hooks_{}.json", agent_id));
    // Include agent_id in hook URL to avoid race condition where hooks arrive
    // before session_id is mapped from Claude CLI stdout
    let hooks_config = serde_json::json!({
        "hooks": {
            "PreToolUse": [{
                "matcher": "*",
                "hooks": [{
                    "type": "command",
                    "command": format!("curl -s -X POST 'http://127.0.0.1:{}/hook?agent_id={}' -H 'Content-Type: application/json' -d @-", hook_port, agent_id)
                }]
            }],
            "PostToolUse": [{
                "matcher": "*",
                "hooks": [{
                    "type": "command",
                    "command": format!("curl -s -X POST 'http://127.0.0.1:{}/hook?agent_id={}' -H 'Content-Type: application/json' -d @-", hook_port, agent_id)
                }]
            }],
            "Stop": [{
                "hooks": [{
                    "type": "command",
                    "command": format!("curl -s -X POST 'http://127.0.0.1:{}/hook?agent_id={}' -H 'Content-Type: application/json' -d @-", hook_port, agent_id)
                }]
            }]
        }
    });

    std::fs::write(
        &settings_path,
        serde_json::to_string_pretty(&hooks_config).unwrap(),
    )
    .map_err(|e| format!("Failed to create settings file: {}", e))?;

    Ok(settings_path)
}

/// Spawn the Claude CLI process with appropriate configuration
pub(crate) fn spawn_claude_process(
    settings_path: &std::path::Path,
    working_dir: &str,
    agent_id: &str,
    model: Option<String>,
) -> Result<tokio::process::Child, String> {
    let claude_path = std::env::var("CLAUDE_PATH")
        .or_else(|_| find_claude_cli())
        .unwrap_or_else(|_| {
            if cfg!(windows) {
                "claude.cmd".to_string()
            } else {
                "claude".to_string()
            }
        });

    // Check if we should block API keys from Claude Code
    // Default to "blocked" if not set (Claude Code uses OAuth)
    let api_key_mode =
        std::env::var("CLAUDE_CODE_API_KEY_MODE").unwrap_or_else(|_| "blocked".to_string());

    let mut cmd = Command::new(&claude_path);

    // Build base args
    let mut args = vec![
        "-p",
        "--verbose",
        "--permission-mode",
        "bypassPermissions",
        "--input-format",
        "stream-json",
        "--output-format",
        "stream-json",
        "--settings",
        settings_path.to_str().unwrap(),
    ];

    // Determine model: use passed model parameter, or fall back to CLAUDE_CODE_MODEL env var
    let model_arg: Option<String> = model.or_else(|| {
        std::env::var("CLAUDE_CODE_MODEL")
            .ok()
            .filter(|m| {
                let m = m.trim().to_lowercase();
                !m.is_empty() && m != "auto"
            })
            .map(|m| m.trim().to_string())
    });

    if let Some(ref model) = model_arg {
        args.push("--model");
        args.push(model);
    }

    cmd.args(&args)
        .current_dir(working_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Get elevation-bin path based on platform
    let elevation_bin_path = get_elevation_bin_path();

    // Apply environment filtering based on mode
    if api_key_mode.to_lowercase() == "blocked" {
        // Filter out sensitive API keys so Claude Code uses OAuth authentication
        let mut filtered_env: HashMap<String, String> = std::env::vars()
            .filter(|(key, _)| !SENSITIVE_ENV_VARS.contains(&key.as_str()))
            .collect();

        // Inject elevation-bin into PATH (prepend so our sudo wrapper is found first)
        if let Some(elevation_path) = &elevation_bin_path {
            let existing_path = filtered_env.get("PATH").cloned().unwrap_or_default();
            let new_path = format!("{}:{}", elevation_path.display(), existing_path);
            filtered_env.insert("PATH".to_string(), new_path);
        }

        // Set CLAUDE_AGENT_ID so wrapper script knows which agent is calling
        filtered_env.insert("CLAUDE_AGENT_ID".to_string(), agent_id.to_string());

        cmd.env_clear().envs(&filtered_env);
    } else {
        // "passthrough" mode: inherit all env vars but still inject elevation
        if let Some(elevation_path) = &elevation_bin_path {
            let existing_path = std::env::var("PATH").unwrap_or_default();
            let new_path = format!("{}:{}", elevation_path.display(), existing_path);
            cmd.env("PATH", new_path);
        }
        cmd.env("CLAUDE_AGENT_ID", agent_id);
    }

    cmd.spawn()
        .map_err(|e| format!("Failed to spawn claude: {}", e))
}
