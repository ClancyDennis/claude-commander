//! Elevation module for sudo command approval system
//!
//! This module provides:
//! - Command risk classification (Normal, Suspicious, High)
//! - Compound command parsing (detecting &&, ||, ;, pipes)
//! - Known installer detection (Homebrew, Rust, etc.)

use crate::types::CommandRiskLevel;

/// Information about a parsed compound command
#[derive(Debug, Clone)]
pub struct CompoundCommandInfo {
    pub full_command: String,
    pub pre_commands: Vec<String>,
    pub sudo_command: Option<String>,
    pub post_commands: Vec<String>,
    pub has_compound_operators: bool,
}

/// Known installer patterns (URL pattern, friendly name)
const KNOWN_INSTALLERS: &[(&str, &str)] = &[
    ("raw.githubusercontent.com/Homebrew", "Homebrew"),
    ("sh.rustup.rs", "Rust"),
    ("raw.githubusercontent.com/nvm-sh", "NVM"),
    ("get.docker.com", "Docker"),
    ("deb.nodesource.com", "Node.js"),
    ("pyenv.run", "pyenv"),
    ("get.rvm.io", "RVM"),
    ("install.python-poetry.org", "Poetry"),
];

/// High-risk command patterns that require extra warning
const HIGH_RISK_PATTERNS: &[&str] = &[
    "rm -rf /",
    "rm -rf /*",
    "rm -rf ~",
    "dd if=",
    "mkfs",
    "chmod -R 777 /",
    "chmod 777 /",
    "> /dev/sd",
    "> /dev/nvme",
    ":(){ :|:& };:", // Fork bomb
    "mv /* ",
    "mv / ",
    "--no-preserve-root",
];

/// Suspicious patterns that warrant a warning banner
const SUSPICIOUS_PATTERNS: &[&str] = &[
    "| bash",
    "|bash",
    "| sh",
    "|sh",
    "curl -s |",
    "wget -q |",
    "bash -c",
    "sh -c",
    "eval ",
    "-S", // sudo -S reads password from stdin
];

/// Classify the risk level of a command
pub fn classify_risk_level(command: &str) -> CommandRiskLevel {
    let cmd_lower = command.to_lowercase();
    let cmd_normalized = command.replace(" ", "");

    // Check for high-risk patterns first
    for pattern in HIGH_RISK_PATTERNS {
        if cmd_lower.contains(&pattern.to_lowercase())
            || cmd_normalized.contains(&pattern.replace(" ", ""))
        {
            return CommandRiskLevel::High;
        }
    }

    // Check for suspicious patterns
    for pattern in SUSPICIOUS_PATTERNS {
        if cmd_lower.contains(&pattern.to_lowercase())
            || cmd_normalized.contains(&pattern.replace(" ", ""))
        {
            return CommandRiskLevel::Suspicious;
        }
    }

    CommandRiskLevel::Normal
}

/// Check if command matches a known installer and return its name
pub fn detect_known_installer(command: &str) -> Option<&'static str> {
    for (pattern, name) in KNOWN_INSTALLERS {
        if command.contains(pattern) {
            return Some(name);
        }
    }
    None
}

/// Generate warnings for a command based on detected patterns
pub fn generate_warnings(command: &str) -> Vec<String> {
    let mut warnings = Vec::new();
    let cmd_lower = command.to_lowercase();

    // Check for stdin password
    if cmd_lower.contains("-s") && cmd_lower.contains("sudo") {
        warnings.push("Uses -S flag which reads password from stdin".to_string());
    }

    // Check for shell escape
    if cmd_lower.contains("bash -c") || cmd_lower.contains("sh -c") {
        warnings.push("Uses shell -c which can hide actual commands".to_string());
    }

    // Check for eval
    if cmd_lower.contains("eval ") {
        warnings.push("Uses eval for dynamic command execution".to_string());
    }

    // Check for piped input to sudo
    if cmd_lower.contains("|") && cmd_lower.contains("sudo") {
        let pipe_pos = cmd_lower.find('|').unwrap_or(0);
        let sudo_pos = cmd_lower.find("sudo").unwrap_or(usize::MAX);
        if pipe_pos < sudo_pos {
            warnings.push("Pipes input to sudo command".to_string());
        }
    }

    // Check for remote script execution
    if (cmd_lower.contains("curl") || cmd_lower.contains("wget"))
        && (cmd_lower.contains("|bash")
            || cmd_lower.contains("|sh")
            || cmd_lower.contains("| bash")
            || cmd_lower.contains("| sh"))
    {
        if detect_known_installer(command).is_none() {
            warnings.push("Downloads and executes remote script".to_string());
        }
    }

    // Check for compound commands
    if cmd_lower.contains("&&") || cmd_lower.contains("||") || cmd_lower.contains(";") {
        warnings.push("Contains multiple chained commands".to_string());
    }

    warnings
}

/// Parse a compound command to extract pre/post sudo components
pub fn parse_compound_command(full_cmd: &str) -> CompoundCommandInfo {
    let mut pre_commands = Vec::new();
    let mut sudo_command = None;
    let mut post_commands = Vec::new();
    let mut found_sudo = false;

    // Split by shell operators while trying to respect quotes
    // This is a simplified parser - a full shell parser would be more complex
    let parts = split_shell_command(full_cmd);

    for part in parts {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }

        if !found_sudo && (trimmed.starts_with("sudo ") || trimmed == "sudo") {
            found_sudo = true;
            // Extract the command after sudo
            let sudo_cmd = trimmed.strip_prefix("sudo ").unwrap_or("").trim();
            if !sudo_cmd.is_empty() {
                sudo_command = Some(sudo_cmd.to_string());
            }
        } else if found_sudo {
            post_commands.push(trimmed.to_string());
        } else {
            pre_commands.push(trimmed.to_string());
        }
    }

    let has_compound = full_cmd.contains("&&")
        || full_cmd.contains("||")
        || full_cmd.contains(";")
        || full_cmd.contains("|");

    CompoundCommandInfo {
        full_command: full_cmd.to_string(),
        pre_commands,
        sudo_command,
        post_commands,
        has_compound_operators: has_compound,
    }
}

/// Split a shell command by operators (&&, ||, ;) while respecting quotes
/// This is a simplified implementation
fn split_shell_command(cmd: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut chars = cmd.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
                current.push(c);
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
                current.push(c);
            }
            '&' if !in_single_quote && !in_double_quote => {
                if chars.peek() == Some(&'&') {
                    chars.next(); // consume second &
                    if !current.trim().is_empty() {
                        parts.push(current.trim().to_string());
                    }
                    current = String::new();
                } else {
                    current.push(c);
                }
            }
            '|' if !in_single_quote && !in_double_quote => {
                if chars.peek() == Some(&'|') {
                    chars.next(); // consume second |
                    if !current.trim().is_empty() {
                        parts.push(current.trim().to_string());
                    }
                    current = String::new();
                } else {
                    // Single pipe - treat as part of command for now
                    // (piped commands are handled differently)
                    current.push(c);
                }
            }
            ';' if !in_single_quote && !in_double_quote => {
                if !current.trim().is_empty() {
                    parts.push(current.trim().to_string());
                }
                current = String::new();
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }

    parts
}

/// Extract inner command from bash -c or sh -c
pub fn extract_inner_command(command: &str) -> Option<String> {
    // Look for patterns like: bash -c "..." or sh -c '...'
    let patterns = [
        "bash -c ",
        "sh -c ",
        "bash -c\"",
        "sh -c\"",
        "bash -c'",
        "sh -c'",
    ];

    for pattern in patterns {
        if let Some(pos) = command.find(pattern) {
            let after_pattern = &command[pos + pattern.len()..];
            let trimmed = after_pattern.trim();

            // Find the quoted content
            if let Some(first_char) = trimmed.chars().next() {
                if first_char == '"' || first_char == '\'' {
                    // Find matching closing quote
                    if let Some(end_pos) = trimmed[1..].find(first_char) {
                        return Some(trimmed[1..end_pos + 1].to_string());
                    }
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_classification() {
        assert_eq!(
            classify_risk_level("apt install nginx"),
            CommandRiskLevel::Normal
        );
        assert_eq!(classify_risk_level("rm -rf /"), CommandRiskLevel::High);
        assert_eq!(
            classify_risk_level("curl https://example.com | bash"),
            CommandRiskLevel::Suspicious
        );
    }

    #[test]
    fn test_known_installer_detection() {
        assert_eq!(
            detect_known_installer(
                "curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh"
            ),
            Some("Homebrew")
        );
        assert_eq!(detect_known_installer("apt install nginx"), None);
    }

    #[test]
    fn test_compound_command_parsing() {
        let info = parse_compound_command("cd /tmp && sudo apt install nginx && echo done");
        assert_eq!(info.pre_commands, vec!["cd /tmp"]);
        assert_eq!(info.sudo_command, Some("apt install nginx".to_string()));
        assert_eq!(info.post_commands, vec!["echo done"]);
        assert!(info.has_compound_operators);
    }

    #[test]
    fn test_inner_command_extraction() {
        assert_eq!(
            extract_inner_command("bash -c \"apt update && apt upgrade\""),
            Some("apt update && apt upgrade".to_string())
        );
    }
}
