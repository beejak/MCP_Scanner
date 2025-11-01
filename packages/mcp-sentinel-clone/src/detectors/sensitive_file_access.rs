use anyhow::Result;
use regex::Regex;
use crate::models::vulnerability::{Vulnerability, VulnerabilityType, Severity, Location};

struct SensitiveFilePattern {
    name: &'static str,
    regex: Regex,
    description: &'static str,
}

lazy_static::lazy_static! {
    static ref SENSITIVE_FILE_PATTERNS: Vec<SensitiveFilePattern> = vec![
        SensitiveFilePattern {
            name: "SSH Key Access",
            regex: Regex::new(r#"(id_rsa|id_ed25519)"#).unwrap(),
            description: "Code appears to be accessing a private SSH key.",
        },
        SensitiveFilePattern {
            name: "AWS Credentials File",
            regex: Regex::new(r#"(\.aws/credentials)"#).unwrap(),
            description: "Code is accessing the AWS credentials file, which contains sensitive access keys.",
        },
        SensitiveFilePattern {
            name: "GCP Credentials File",
            regex: Regex::new(r#"(\.config/gcloud/)"#).unwrap(),
            description: "Code is accessing a Google Cloud Platform credentials file.",
        },
        SensitiveFilePattern {
            name: "Shell History File",
            regex: Regex::new(r#"(\.bash_history|\.zsh_history)"#).unwrap(),
            description: "Accessing shell history files can expose sensitive commands and data.",
        },
        SensitiveFilePattern {
            name: "Environment File",
            regex: Regex::new(r#"(\.env)"#).unwrap(),
            description: "Accessing a .env file, which often contains secrets and configuration.",
        },
    ];
}

pub fn detect(content: &str, file_path: &str) -> Result<Vec<Vulnerability>> {
    let mut vulnerabilities = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        for pattern in SENSITIVE_FILE_PATTERNS.iter() {
            if pattern.regex.is_match(line) {
                let vuln = Vulnerability {
                    id: "SFA-001".to_string(),
                    vulnerability_type: VulnerabilityType::SensitiveFileAccess,
                    severity: Severity::High,
                    title: format!("Potential Sensitive File Access: {}", pattern.name),
                    description: pattern.description.to_string(),
                    location: Location {
                        file_path: file_path.to_string(),
                        line: line_num + 1,
                        column: 1,
                    },
                    impact: "Reading sensitive files can lead to the exposure of credentials, private keys, and other secrets.".to_string(),
                    remediation: "Avoid accessing sensitive system or configuration files directly in the code. Use environment variables or a dedicated secrets management service.".to_string(),
                    code_snippet: line.trim().to_string(),
                    confidence: 0.7,
                };
                vulnerabilities.push(vuln);
            }
        }
    }

    Ok(vulnerabilities)
}
