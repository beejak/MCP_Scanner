use anyhow::Result;
use regex::Regex;
use crate::models::vulnerability::{Vulnerability, VulnerabilityType, Severity, Location};

struct ToolPoisoningPattern {
    name: &'static str,
    regex: Regex,
    description: &'static str,
}

lazy_static::lazy_static! {
    static ref TOOL_POISONING_PATTERNS: Vec<ToolPoisoningPattern> = vec![
        ToolPoisoningPattern {
            name: "Invisible Unicode Characters",
            // This regex looks for non-printable, spacing, or control characters that aren't standard whitespace.
            regex: Regex::new(r#"[\u200B-\u200D\uFEFF\u00A0\p{C}]"#).unwrap(),
            description: "Invisible or non-standard Unicode characters were found, which can be used to obscure malicious code from view.",
        },
        ToolPoisoningPattern {
            name: "Model Instruction Manipulation",
            regex: Regex::new(r#"(?i)(ignore|disregard|override|forget)\s+(the\s+)?(above|previous)\s+(instructions?|prompt|context)"#).unwrap(),
            description: "Keywords detected that may be attempting to manipulate or poison the model's instructions.",
        },
        ToolPoisoningPattern {
            name: "Hidden Markers",
            regex: Regex::new(r#"(\[HIDDEN:\]|\[SECRET:\])"#).unwrap(),
            description: "Hidden markers found, which could be an attempt to embed secret instructions or data.",
        },
    ];
}

pub fn detect(content: &str, file_path: &str) -> Result<Vec<Vulnerability>> {
    let mut vulnerabilities = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        for pattern in TOOL_POISONING_PATTERNS.iter() {
            if pattern.regex.is_match(line) {
                let vuln = Vulnerability {
                    id: "TP-001".to_string(),
                    vulnerability_type: VulnerabilityType::ToolPoisoning,
                    severity: Severity::Medium,
                    title: format!("Potential Tool Poisoning: {}", pattern.name),
                    description: pattern.description.to_string(),
                    location: Location {
                        file_path: file_path.to_string(),
                        line: line_num + 1,
                        column: 1,
                    },
                    impact: "Tool poisoning can cause the model to behave in unintended ways, bypass security controls, or execute malicious instructions.".to_string(),
                    remediation: "Review the suspicious code to ensure it is not attempting to manipulate the language model. Remove any unnecessary or malicious patterns.".to_string(),
                    code_snippet: line.trim().to_string(),
                    confidence: 0.6,
                };
                vulnerabilities.push(vuln);
            }
        }
    }

    Ok(vulnerabilities)
}
