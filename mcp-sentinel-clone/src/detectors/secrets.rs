use anyhow::Result;
use regex::Regex;
use crate::models::vulnerability::{Vulnerability, VulnerabilityType, Severity, Location};

struct SecretPattern {
    name: &'static str,
    regex: Regex,
}

lazy_static::lazy_static! {
    static ref SECRET_PATTERNS: Vec<SecretPattern> = vec![
        SecretPattern {
            name: "AWS Access Key ID",
            regex: Regex::new(r#"(?i)(AKIA[A-Z0-9]{16})"#).unwrap(),
        },
        SecretPattern {
            name: "OpenAI API Key",
            regex: Regex::new(r#"(sk-[a-zA-Z0-9]{48})"#).unwrap(),
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
