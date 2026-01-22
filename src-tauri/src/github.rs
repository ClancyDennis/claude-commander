use std::process::Command;
use regex::Regex;
use crate::types::GitHubContext;

/// Parse a GitHub URL to extract owner and repo
/// Supports formats:
/// - https://github.com/owner/repo
/// - https://github.com/owner/repo.git
/// - git@github.com:owner/repo.git
pub fn parse_github_url(url: &str) -> Option<(String, String)> {
    // HTTPS format
    let https_re = Regex::new(r"https?://github\.com/([^/]+)/([^/\.]+)").ok()?;
    if let Some(caps) = https_re.captures(url) {
        let owner = caps.get(1)?.as_str().to_string();
        let repo = caps.get(2)?.as_str().to_string();
        return Some((owner, repo));
    }

    // SSH format
    let ssh_re = Regex::new(r"git@github\.com:([^/]+)/([^/\.]+)").ok()?;
    if let Some(caps) = ssh_re.captures(url) {
        let owner = caps.get(1)?.as_str().to_string();
        let repo = caps.get(2)?.as_str().to_string();
        return Some((owner, repo));
    }

    None
}

/// Check if a directory is a git repository
pub fn detect_git_repo(path: &str) -> bool {
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(path)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Get the current git branch
pub fn get_current_branch(path: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(path)
        .output()
        .ok()?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .ok()
            .map(|s| s.trim().to_string())
    } else {
        None
    }
}

/// Get the current git commit SHA
pub fn get_current_commit(path: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(path)
        .output()
        .ok()?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .ok()
            .map(|s| s.trim().to_string())
    } else {
        None
    }
}

/// Get the remote origin URL
pub fn get_remote_url(path: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["config", "--get", "remote.origin.url"])
        .current_dir(path)
        .output()
        .ok()?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .ok()
            .map(|s| s.trim().to_string())
    } else {
        None
    }
}

/// Build GitHub context from a directory
/// First checks for a git repo, then extracts GitHub info
pub fn build_github_context(path: &str, provided_url: Option<String>) -> Option<GitHubContext> {
    // Use provided URL or try to detect from git remote
    let github_url = provided_url.or_else(|| {
        if detect_git_repo(path) {
            get_remote_url(path)
        } else {
            None
        }
    })?;

    // Parse owner and repo from URL
    let (owner, repo) = parse_github_url(&github_url)?;

    // Get branch and commit if it's a git repo
    let (branch, commit_sha) = if detect_git_repo(path) {
        (
            get_current_branch(path).unwrap_or_else(|| "main".to_string()),
            get_current_commit(path),
        )
    } else {
        ("main".to_string(), None)
    };

    let now = chrono::Utc::now().to_rfc3339();

    Some(GitHubContext {
        repository_url: github_url,
        owner,
        repo,
        branch,
        commit_sha,
        last_synced: Some(now),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_url_https() {
        let (owner, repo) = parse_github_url("https://github.com/anthropics/anthropic-sdk-python").unwrap();
        assert_eq!(owner, "anthropics");
        assert_eq!(repo, "anthropic-sdk-python");
    }

    #[test]
    fn test_parse_github_url_https_with_git() {
        let (owner, repo) = parse_github_url("https://github.com/anthropics/anthropic-sdk-python.git").unwrap();
        assert_eq!(owner, "anthropics");
        assert_eq!(repo, "anthropic-sdk-python");
    }

    #[test]
    fn test_parse_github_url_ssh() {
        let (owner, repo) = parse_github_url("git@github.com:anthropics/anthropic-sdk-python.git").unwrap();
        assert_eq!(owner, "anthropics");
        assert_eq!(repo, "anthropic-sdk-python");
    }

    #[test]
    fn test_parse_github_url_invalid() {
        assert!(parse_github_url("not a github url").is_none());
    }
}
