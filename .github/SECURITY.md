# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

We take security vulnerabilities seriously. If you discover a security issue, please report it responsibly.

### How to Report

**Please do NOT report security vulnerabilities through public GitHub issues.**

Instead, please report them via email to the maintainers or use GitHub's private vulnerability reporting feature:

1. Go to the repository's **Security** tab
2. Click **Report a vulnerability**
3. Fill out the form with details about the vulnerability

### What to Include

When reporting a vulnerability, please include:

- A description of the vulnerability
- Steps to reproduce the issue
- Potential impact of the vulnerability
- Any suggested fixes (if you have them)

### What to Expect

- **Acknowledgment**: We will acknowledge receipt of your report within 48 hours
- **Updates**: We will provide updates on the progress of addressing the vulnerability
- **Resolution**: We aim to resolve critical vulnerabilities within 7 days
- **Credit**: We will credit you in release notes (unless you prefer to remain anonymous)

## Security Considerations

### API Keys

Claude Commander requires API keys for AI functionality. Please note:

- API keys are stored locally on your machine
- Keys are never transmitted to any server other than the respective AI provider (Anthropic/OpenAI)
- We recommend using environment variables or the `.env` file (which is gitignored)

### Agent Execution

Claude Commander executes Claude Code agents that can:

- Read and write files in selected directories
- Execute shell commands
- Make network requests

**Recommendations:**

- Only run agents in directories you trust
- Review agent actions before approval (when using checkpoints)
- Use the built-in prompt injection monitoring for untrusted inputs
- Keep the application updated to receive security patches

### Data Storage

- Session logs and cost history are stored locally
- No telemetry or usage data is collected
- All AI interactions go directly to the configured provider

## Security Features

Claude Commander includes several security features:

1. **Prompt Injection Monitoring**: Active scanning for prompt injection attempts
2. **Human Checkpoints**: Optional review gates before critical operations
3. **Permission Controls**: Designed to reduce blanket permission grants
4. **Local-First**: All data stays on your machine

## Updates

Security updates will be released as patch versions. We recommend:

- Enabling GitHub notifications for releases
- Regularly updating to the latest version
- Subscribing to security advisories for this repository
