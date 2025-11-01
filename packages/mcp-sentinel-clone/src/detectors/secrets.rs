use anyhow::Result;
use regex::Regex;
use crate::models::vulnerability::{Vulnerability, VulnerabilityType, Severity, Location};

struct SecretPattern {
    name: &'static str,
    regex: Regex,
}

lazy_static::lazy_static! {
    static ref SECRET_PATTERNS: Vec<SecretPattern> = vec![
        // AWS Access Keys
        SecretPattern {
            name: "AWS Access Key ID",
            regex: Regex::new(r#"(?i)(AKIA[A-Z0-9]{16})"#).unwrap(),
        },
        SecretPattern {
            name: "AWS Secret Access Key",
            regex: Regex::new(r#"(?i)(ASIA[A-Z0-9]{16})"#).unwrap(),
        },
        // OpenAI API Keys
        SecretPattern {
            name: "OpenAI API Key",
            regex: Regex::new(r#"(sk-[a-zA-Z0-9]{48})"#).unwrap(),
        },
        // Anthropic API Keys
        SecretPattern {
            name: "Anthropic API Key",
            regex: Regex::new(r#"(sk-ant-[a-zA-Z0-9-]{95})"#).unwrap(),
        },
        // JWT Tokens
        SecretPattern {
            name: "JWT Token",
            regex: Regex::new(r#"(eyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,})"#).unwrap(),
        },
        // Private Keys
        SecretPattern {
            name: "RSA Private Key",
            regex: Regex::new(r#"-----BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY-----"#).unwrap(),
        },
        // Database Connection Strings
        SecretPattern {
            name: "PostgreSQL Connection String",
            regex: Regex::new(r#"postgres://[^:]+:[^@]+@[^/]+/\S+"#).unwrap(),
        },
        SecretPattern {
            name: "MySQL Connection String",
            regex: Regex::new(r#"mysql://[^:]+:[^@]+@[^/]+/\S+"#).unwrap(),
        },
        // GitHub Tokens
        SecretPattern {
            name: "GitHub Token",
            regex: Regex::new(r#"(ghp_[a-zA-Z0-9]{36})"#).unwrap(),
        },
        SecretPattern {
            name: "GitHub OAuth Token",
            regex: Regex::new(r#"(gho_[a-zA-Z0-9]{36})"#).unwrap(),
        },
        // Generic API Keys
        SecretPattern {
            name: "Generic API Key",
            regex: Regex::new(r#"(?i)(api[_-]?key|apikey|api[_-]?secret)['"\s]*[:=]\s*['"]?([a-zA-Z0-9_\-]{32,})['"]?"#).unwrap(),
        },
        // Slack Tokens
        SecretPattern {
            name: "Slack Token",
            regex: Regex::new(r#"(xox[pborsa]-[0-9]{10,13}-[0-9]{10,13}-[0-9]{10,13}-[a-z0-9]{32})"#).unwrap(),
        },
        // Google API Keys
        SecretPattern {
            name: "Google API Key",
            regex: Regex::new(r#"AIza[0-9A-Za-z_-]{35}"#).unwrap(),
        },
        // Hardcoded Passwords
        SecretPattern {
            name: "Hardcoded Password",
            regex: Regex::new(r#"(?i)(password|passwd|pwd)['"\s]*[:=]\s*['"]([^'"\s]{8,})['"]"#).unwrap(),
        },
    ];
}

pub fn detect(content: &str, file_path: &str) -> Result<Vec<Vulnerability>> {
    let mut vulnerabilities = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        for pattern in SECRET_PATTERNS.iter() {
            if let Some(captures) = pattern.regex.captures(line) {
                let secret_text = captures.get(0).map_or("", |m| m.as_str());
                let vuln = Vulnerability {
                    id: format!("SEC-001"),
                    vulnerability_type: VulnerabilityType::SecretsLeakage,
                    severity: Severity::Critical,
                    title: format!("{} Found", pattern.name),
                    description: format!("A secret of type {} was found.", pattern.name),
                    location: Location {
                        file_path: file_path.to_string(),
                        line: line_num + 1,
                        column: line.find(secret_text).unwrap_or(0) + 1,
                    },
                    impact: "Exposure of secrets can lead to unauthorized access.".to_string(),
                    remediation: "Remove secrets from code and use a secret management solution.".to_string(),
                    code_snippet: line.to_string(),
                    confidence: 0.95,
                };
                vulnerabilities.push(vuln);
            }
        }
    }

    Ok(vulnerabilities)
}
