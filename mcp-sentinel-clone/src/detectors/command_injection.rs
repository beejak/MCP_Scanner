use anyhow::Result;
use regex::Regex;
use crate::models::vulnerability::{Vulnerability, VulnerabilityType, Severity, Location};

struct CommandInjectionPattern {
    name: &'static str,
    regex: Regex,
    language: &'static str,
}

lazy_static::lazy_static! {
    static ref COMMAND_INJECTION_PATTERNS: Vec<CommandInjectionPattern> = vec![
        // Python
        CommandInjectionPattern {
            name: "Python OS System",
            regex: Regex::new(r#"os\.system\s*\("#).unwrap(),
            language: "Python",
        },
        CommandInjectionPattern {
            name: "Python Subprocess Shell",
            regex: Regex::new(r#"subprocess\..*shell\s*=\s*True"#).unwrap(),
            language: "Python",
        },
        // JavaScript
        CommandInjectionPattern {
            name: "JavaScript Child Process Exec",
            regex: Regex::new(r#"child_process\.exec\s*\("#).unwrap(),
            language: "JavaScript",
        },
    ];
}

pub fn detect(content: &str, file_path: &str) -> Result<Vec<Vulnerability>> {
    let mut vulnerabilities = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        for pattern in COMMAND_INJECTION_PATTERNS.iter() {
            if pattern.regex.is_match(line) {
                let vuln = Vulnerability {
                    id: "CI-001".to_string(),
                    vulnerability_type: VulnerabilityType::CommandInjection,
                    severity: Severity::High,
                    title: format!("{} Command Injection", pattern.language),
                    description: format!("Potential command injection using `{}`.", pattern.name),
                    location: Location {
                        file_path: file_path.to_string(),
                        line: line_num + 1,
                        column: 1,
                    },
                    impact: "Allows attackers to execute arbitrary commands on the server.".to_string(),
                    remediation: "Avoid using shell execution. Sanitize user input.".to_string(),
                    code_snippet: line.to_string(),
                    confidence: 0.8,
                };
                vulnerabilities.push(vuln);
            }
        }
    }

    Ok(vulnerabilities)
}
