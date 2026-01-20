//! Default security detection rules.

use super::pattern_matcher::{DetectionRule, PatternSpec, Severity, ThreatCategory};

/// Get the default set of detection rules
pub fn get_default_rules() -> Vec<DetectionRule> {
    let mut rules = Vec::new();

    // Prompt Injection Rules
    rules.extend(prompt_injection_rules());

    // Dangerous Command Rules
    rules.extend(dangerous_command_rules());

    // Unauthorized File Access Rules
    rules.extend(file_access_rules());

    // Data Exfiltration Rules
    rules.extend(data_exfiltration_rules());

    // Privilege Escalation Rules
    rules.extend(privilege_escalation_rules());

    rules
}

/// Prompt injection detection rules
fn prompt_injection_rules() -> Vec<DetectionRule> {
    vec![
        DetectionRule {
            id: "PI001".to_string(),
            name: "System Prompt Override Attempt".to_string(),
            description: "Detects attempts to override or ignore system prompts".to_string(),
            category: ThreatCategory::PromptInjection,
            severity: Severity::Critical,
            enabled: true,
            patterns: vec![PatternSpec {
                field: "content".to_string(),
                pattern: r"(?i)(ignore|disregard|forget|override|bypass).{0,30}(previous|prior|above|system|initial|original).{0,30}(instructions?|prompts?|rules?|commands?|guidelines?)".to_string(),
                negate: false,
            }],
        },
        DetectionRule {
            id: "PI002".to_string(),
            name: "Role Hijacking Attempt".to_string(),
            description: "Detects attempts to make the agent assume a different role".to_string(),
            category: ThreatCategory::PromptInjection,
            severity: Severity::High,
            enabled: true,
            patterns: vec![PatternSpec {
                field: "content".to_string(),
                pattern: r"(?i)(you are now|act as|pretend to be|assume the role|roleplay as|from now on you('re| are)).{0,50}(admin|root|developer|system|god|unrestricted|jailbroken|DAN|evil)".to_string(),
                negate: false,
            }],
        },
        DetectionRule {
            id: "PI003".to_string(),
            name: "Instruction Injection in Tool Output".to_string(),
            description: "Detects instruction-like patterns in tool results that may be injection attempts".to_string(),
            category: ThreatCategory::PromptInjection,
            severity: Severity::High,
            enabled: true,
            patterns: vec![PatternSpec {
                field: "content".to_string(),
                pattern: r"(?i)(IMPORTANT|URGENT|CRITICAL|OVERRIDE|NEW INSTRUCTION|SYSTEM MESSAGE|ADMIN NOTE|PRIORITY):\s*[A-Z]".to_string(),
                negate: false,
            }],
        },
        DetectionRule {
            id: "PI004".to_string(),
            name: "Jailbreak Keywords".to_string(),
            description: "Detects common jailbreak attempt keywords".to_string(),
            category: ThreatCategory::PromptInjection,
            severity: Severity::High,
            enabled: true,
            patterns: vec![PatternSpec {
                field: "content".to_string(),
                pattern: r"(?i)(jailbreak|DAN mode|developer mode|god mode|unrestricted mode|sudo mode|no restrictions|bypass safety|ignore safety|disable filters)".to_string(),
                negate: false,
            }],
        },
        DetectionRule {
            id: "PI005".to_string(),
            name: "Delimiter Escape Attempt".to_string(),
            description: "Detects attempts to escape prompt delimiters".to_string(),
            category: ThreatCategory::PromptInjection,
            severity: Severity::Medium,
            enabled: true,
            patterns: vec![PatternSpec {
                field: "content".to_string(),
                pattern: r"(```\s*system|<\|im_start\|>|<\|endoftext\|>|\[INST\]|\[/INST\]|<<SYS>>|<</SYS>>|Human:|Assistant:)".to_string(),
                negate: false,
            }],
        },
        DetectionRule {
            id: "PI006".to_string(),
            name: "Base64 Encoded Payload".to_string(),
            description: "Detects large base64 encoded content that may hide malicious instructions".to_string(),
            category: ThreatCategory::PromptInjection,
            severity: Severity::Medium,
            enabled: true,
            patterns: vec![PatternSpec {
                field: "content".to_string(),
                // Matches base64 strings of at least 100 characters
                pattern: r"(?:[A-Za-z0-9+/]{4}){25,}(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?".to_string(),
                negate: false,
            }],
        },
    ]
}

/// Dangerous command detection rules
fn dangerous_command_rules() -> Vec<DetectionRule> {
    vec![
        DetectionRule {
            id: "DC001".to_string(),
            name: "Destructive Command".to_string(),
            description: "Detects potentially destructive shell commands".to_string(),
            category: ThreatCategory::DangerousCommand,
            severity: Severity::Critical,
            enabled: true,
            patterns: vec![PatternSpec {
                field: "command".to_string(),
                pattern: r"(?i)(rm\s+-[rf]{1,2}f?\s+/(?!\w)|mkfs\s|dd\s+if=.+of=/dev/|wipefs|shred\s|format\s+[cC]:)".to_string(),
                negate: false,
            }],
        },
        DetectionRule {
            id: "DC002".to_string(),
            name: "Reverse Shell Pattern".to_string(),
            description: "Detects reverse shell establishment attempts".to_string(),
            category: ThreatCategory::DangerousCommand,
            severity: Severity::Critical,
            enabled: true,
            patterns: vec![PatternSpec {
                field: "command".to_string(),
                pattern: r"(?i)(bash\s+-i.*>&.*(/dev/tcp|/dev/udp)|nc\s+(-e|--exec)\s*/bin/|ncat.*(-e|--exec)|python.*socket.*connect.*exec|perl.*socket.*INET|ruby.*TCPSocket|php.*fsockopen.*exec)".to_string(),
                negate: false,
            }],
        },
        DetectionRule {
            id: "DC003".to_string(),
            name: "Cron Job Manipulation".to_string(),
            description: "Detects attempts to modify cron jobs".to_string(),
            category: ThreatCategory::DangerousCommand,
            severity: Severity::High,
            enabled: true,
            patterns: vec![PatternSpec {
                field: "command".to_string(),
                pattern: r"(?i)(crontab\s+-[elr]|echo.+>>\s*/etc/cron|/var/spool/cron)".to_string(),
                negate: false,
            }],
        },
        DetectionRule {
            id: "DC004".to_string(),
            name: "Service Manipulation".to_string(),
            description: "Detects attempts to stop or disable system services".to_string(),
            category: ThreatCategory::DangerousCommand,
            severity: Severity::High,
            enabled: true,
            patterns: vec![PatternSpec {
                field: "command".to_string(),
                pattern: r"(?i)(systemctl\s+(stop|disable|mask)|service\s+\w+\s+stop|killall\s+-9|pkill\s+-9)".to_string(),
                negate: false,
            }],
        },
        DetectionRule {
            id: "DC005".to_string(),
            name: "History Tampering".to_string(),
            description: "Detects attempts to clear or modify command history".to_string(),
            category: ThreatCategory::DangerousCommand,
            severity: Severity::Medium,
            enabled: true,
            patterns: vec![PatternSpec {
                field: "command".to_string(),
                pattern: r"(?i)(history\s+-[cd]|rm\s+.*\.bash_history|unset\s+HISTFILE|export\s+HISTSIZE=0)".to_string(),
                negate: false,
            }],
        },
    ]
}

/// Unauthorized file access detection rules
fn file_access_rules() -> Vec<DetectionRule> {
    vec![
        DetectionRule {
            id: "FA001".to_string(),
            name: "Sensitive System File Access".to_string(),
            description: "Detects access to sensitive system files".to_string(),
            category: ThreatCategory::UnauthorizedFileAccess,
            severity: Severity::Critical,
            enabled: true,
            patterns: vec![PatternSpec {
                field: "path".to_string(),
                pattern: r"(?i)(/etc/(passwd|shadow|sudoers|gshadow)|/root/|/home/\w+/\.ssh/)".to_string(),
                negate: false,
            }],
        },
        DetectionRule {
            id: "FA002".to_string(),
            name: "Cloud Credentials Access".to_string(),
            description: "Detects access to cloud provider credential files".to_string(),
            category: ThreatCategory::UnauthorizedFileAccess,
            severity: Severity::Critical,
            enabled: true,
            patterns: vec![PatternSpec {
                field: "path".to_string(),
                pattern: r"(?i)(\.aws/(credentials|config)|\.azure/|\.gcloud/|\.kube/config|\.docker/config\.json)".to_string(),
                negate: false,
            }],
        },
        DetectionRule {
            id: "FA003".to_string(),
            name: "Environment File Access".to_string(),
            description: "Detects access to environment files that may contain secrets".to_string(),
            category: ThreatCategory::UnauthorizedFileAccess,
            severity: Severity::High,
            enabled: true,
            patterns: vec![PatternSpec {
                field: "path".to_string(),
                pattern: r"(?i)(\.env$|\.env\.(local|development|production|staging)|secrets?\.(ya?ml|json)|credentials?\.(ya?ml|json))".to_string(),
                negate: false,
            }],
        },
        DetectionRule {
            id: "FA004".to_string(),
            name: "Private Key Access".to_string(),
            description: "Detects access to private key files".to_string(),
            category: ThreatCategory::UnauthorizedFileAccess,
            severity: Severity::Critical,
            enabled: true,
            patterns: vec![PatternSpec {
                field: "path".to_string(),
                pattern: r"(?i)(\.(pem|key|p12|pfx|ppk)$|id_(rsa|dsa|ecdsa|ed25519)$|\.keystore$)".to_string(),
                negate: false,
            }],
        },
        DetectionRule {
            id: "FA005".to_string(),
            name: "System Directory Write".to_string(),
            description: "Detects writes to system directories".to_string(),
            category: ThreatCategory::SystemTampering,
            severity: Severity::Critical,
            enabled: true,
            patterns: vec![PatternSpec {
                field: "path".to_string(),
                pattern: r"^(/usr/(bin|sbin|lib)|/bin|/sbin|/lib|/etc|/boot|/sys|/proc)/".to_string(),
                negate: false,
            }],
        },
    ]
}

/// Data exfiltration detection rules
fn data_exfiltration_rules() -> Vec<DetectionRule> {
    vec![
        DetectionRule {
            id: "DE001".to_string(),
            name: "Network Data Exfiltration Command".to_string(),
            description: "Detects commands that may exfiltrate data over the network".to_string(),
            category: ThreatCategory::DataExfiltration,
            severity: Severity::High,
            enabled: true,
            patterns: vec![PatternSpec {
                field: "command".to_string(),
                pattern: r"(?i)(curl|wget|nc|netcat|ncat)\s+.*(--data|-d\s+@|--upload-file|-T\s+|POST\s+--data|<\s*/etc/)".to_string(),
                negate: false,
            }],
        },
        DetectionRule {
            id: "DE002".to_string(),
            name: "DNS Exfiltration Pattern".to_string(),
            description: "Detects potential DNS-based data exfiltration".to_string(),
            category: ThreatCategory::DataExfiltration,
            severity: Severity::High,
            enabled: true,
            patterns: vec![PatternSpec {
                field: "command".to_string(),
                pattern: r"(?i)(nslookup|dig|host)\s+.{50,}\.".to_string(),
                negate: false,
            }],
        },
        DetectionRule {
            id: "DE003".to_string(),
            name: "Archive and Upload Pattern".to_string(),
            description: "Detects archiving files and potentially uploading them".to_string(),
            category: ThreatCategory::DataExfiltration,
            severity: Severity::Medium,
            enabled: true,
            patterns: vec![PatternSpec {
                field: "command".to_string(),
                pattern: r"(?i)(tar\s+[cz].*&&.*(curl|wget|scp|rsync)|zip.*&&.*(curl|wget|scp|rsync))".to_string(),
                negate: false,
            }],
        },
        DetectionRule {
            id: "DE004".to_string(),
            name: "Large Output to Network".to_string(),
            description: "Detects piping large data to network commands".to_string(),
            category: ThreatCategory::DataExfiltration,
            severity: Severity::Medium,
            enabled: true,
            patterns: vec![PatternSpec {
                field: "command".to_string(),
                pattern: r"(?i)(cat|head|tail|less|more|xxd|base64).+\|\s*(curl|wget|nc|netcat)".to_string(),
                negate: false,
            }],
        },
    ]
}

/// Privilege escalation detection rules
fn privilege_escalation_rules() -> Vec<DetectionRule> {
    vec![
        DetectionRule {
            id: "PE001".to_string(),
            name: "Sudo Command".to_string(),
            description: "Detects use of sudo for privilege escalation".to_string(),
            category: ThreatCategory::PrivilegeEscalation,
            severity: Severity::High,
            enabled: true,
            patterns: vec![PatternSpec {
                field: "command".to_string(),
                pattern: r"(?i)^sudo\s+".to_string(),
                negate: false,
            }],
        },
        DetectionRule {
            id: "PE002".to_string(),
            name: "SUID/SGID Bit Manipulation".to_string(),
            description: "Detects attempts to set SUID/SGID bits".to_string(),
            category: ThreatCategory::PrivilegeEscalation,
            severity: Severity::Critical,
            enabled: true,
            patterns: vec![PatternSpec {
                field: "command".to_string(),
                pattern: r"(?i)chmod\s+[0-7]*[4567][0-7]*\s|chmod\s+[ugo]*\+s".to_string(),
                negate: false,
            }],
        },
        DetectionRule {
            id: "PE003".to_string(),
            name: "User/Group Manipulation".to_string(),
            description: "Detects attempts to manipulate users or groups".to_string(),
            category: ThreatCategory::PrivilegeEscalation,
            severity: Severity::High,
            enabled: true,
            patterns: vec![PatternSpec {
                field: "command".to_string(),
                pattern: r"(?i)(useradd|usermod|groupadd|adduser|passwd\s+root|chpasswd)".to_string(),
                negate: false,
            }],
        },
        DetectionRule {
            id: "PE004".to_string(),
            name: "Sudoers Modification".to_string(),
            description: "Detects attempts to modify sudoers file".to_string(),
            category: ThreatCategory::PrivilegeEscalation,
            severity: Severity::Critical,
            enabled: true,
            patterns: vec![PatternSpec {
                field: "command".to_string(),
                pattern: r"(?i)(visudo|echo.*>>?\s*/etc/sudoers|tee.*(/etc/sudoers|/etc/sudoers\.d/))".to_string(),
                negate: false,
            }],
        },
        DetectionRule {
            id: "PE005".to_string(),
            name: "Capability Manipulation".to_string(),
            description: "Detects attempts to manipulate Linux capabilities".to_string(),
            category: ThreatCategory::PrivilegeEscalation,
            severity: Severity::High,
            enabled: true,
            patterns: vec![PatternSpec {
                field: "command".to_string(),
                pattern: r"(?i)(setcap|getcap.*\+ep|capsh)".to_string(),
                negate: false,
            }],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_rules_load() {
        let rules = get_default_rules();
        assert!(!rules.is_empty());

        // Check we have rules for each category
        let categories: std::collections::HashSet<_> = rules.iter().map(|r| &r.category).collect();
        assert!(categories.contains(&ThreatCategory::PromptInjection));
        assert!(categories.contains(&ThreatCategory::DangerousCommand));
        assert!(categories.contains(&ThreatCategory::UnauthorizedFileAccess));
        assert!(categories.contains(&ThreatCategory::DataExfiltration));
        assert!(categories.contains(&ThreatCategory::PrivilegeEscalation));
    }

    #[test]
    fn test_all_rules_have_valid_regex() {
        let rules = get_default_rules();

        for rule in rules {
            for pattern in &rule.patterns {
                let result = regex::Regex::new(&pattern.pattern);
                assert!(
                    result.is_ok(),
                    "Rule {} has invalid regex: {}",
                    rule.id,
                    pattern.pattern
                );
            }
        }
    }

    #[test]
    fn test_all_rules_have_required_fields() {
        let rules = get_default_rules();

        for rule in rules {
            assert!(!rule.id.is_empty(), "Rule has empty id");
            assert!(!rule.name.is_empty(), "Rule {} has empty name", rule.id);
            assert!(!rule.description.is_empty(), "Rule {} has empty description", rule.id);
            assert!(!rule.patterns.is_empty(), "Rule {} has no patterns", rule.id);
        }
    }
}
