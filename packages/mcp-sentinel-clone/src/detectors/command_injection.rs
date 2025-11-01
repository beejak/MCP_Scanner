use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;
use crate::models::vulnerability::{Vulnerability, VulnerabilityType, Severity, Location};

lazy_static! {
    static ref OS_SYSTEM_RE: Regex = Regex::new(r#"os\.system\s*\("#).unwrap();
    static ref SUBPROCESS_SHELL_TRUE_RE: Regex = Regex::new(r#"subprocess\.(call|run|check_call|check_output)\s*\(.*shell\s*=\s*True.*\)"#).unwrap();
}

pub fn detect(content: &str, file_path: &str) -> Result<Vec<Vulnerability>> {
    let mut vulnerabilities = vec![];

    if !file_path.ends_with(".py") {
        return Ok(vulnerabilities);
    }

    for (line_num, line) in content.lines().enumerate() {
        if OS_SYSTEM_RE.is_match(line) || SUBPROCESS_SHELL_TRUE_RE.is_match(line) {
            vulnerabilities.push(Vulnerability {
                id: "CI-001".to_string(),
                vulnerability_type: VulnerabilityType::CommandInjection,
                severity: Severity::High,
                title: "Potential Command Injection".to_string(),
                description: "A function call that could be vulnerable to command injection was found.".to_string(),
                location: Location {
                    file_path: file_path.to_string(),
                    line: line_num + 1,
                    column: 1, // Simple for now
                },
                impact: "Allows attackers to execute arbitrary commands on the server.".to_string(),
                remediation: "Avoid using shell execution. Sanitize user input.".to_string(),
                code_snippet: line.to_string(),
                confidence: 0.7, // Lower confidence than AST
            });
        }
    }

    Ok(vulnerabilities)
}
