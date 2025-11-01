use anyhow::Result;
use regex::Regex;
use crate::models::vulnerability::{Vulnerability, VulnerabilityType, Severity, Location};

struct PromptInjectionPattern {
    name: &'static str,
    regex: Regex,
    description: &'static str,
}

lazy_static::lazy_static! {
    static ref PROMPT_INJECTION_PATTERNS: Vec<PromptInjectionPattern> = vec![
        PromptInjectionPattern {
            name: "Ignore Previous Instructions",
            regex: Regex::new(r#"(?i)(ignore|disregard|forget|disregard|pay no attention to|now forget)\s+(the|all)\s+(above|previous|prior)\s+(instruction|instructions|context|prompts?)"#).unwrap(),
            description: "A classic prompt injection technique attempting to make the model ignore its initial system prompt.",
        },
        PromptInjectionPattern {
            name: "System Prompt Evasion",
            regex: Regex::new(r#"(?i)what are your instructions\?|what is your system prompt\?|reveal your instructions|output your system prompt"#).unwrap(),
            description: "An attempt to make the model reveal its own system prompt or confidential instructions.",
        },
        PromptInjectionPattern {
            name: "Role Playing Attack",
            regex: Regex::new(r#"(?i)you are now\s+(a|an)\s+.*(developer|hacker|unrestricted AI)|act as|roleplay as"#).unwrap(),
            description: "An attempt to make the model adopt a different persona, potentially one with fewer safety restrictions.",
        },
        PromptInjectionPattern {
            name: "Confidential Information Leak",
            regex: Regex::new(r#"(?i)(what is the content of|output the content of)\s+(the|file)\s+.*(password|secret|key)"#).unwrap(),
            description: "A pattern that may be trying to trick the model into leaking sensitive information from its context or tools.",
        },
    ];
}

pub fn detect(content: &str, file_path: &str) -> Result<Vec<Vulnerability>> {
    let mut vulnerabilities = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        for pattern in PROMPT_INJECTION_PATTERNS.iter() {
            if pattern.regex.is_match(line) {
                let vuln = Vulnerability {
                    id: "PI-001".to_string(),
                    vulnerability_type: VulnerabilityType::PromptInjection,
                    severity: Severity::High,
                    title: format!("Potential Prompt Injection: {}", pattern.name),
                    description: pattern.description.to_string(),
                    location: Location {
                        file_path: file_path.to_string(),
                        line: line_num + 1,
                        column: 1,
                    },
                    impact: "Prompt injection can lead to data exfiltration, unauthorized actions, and bypassing of the model's safety and moderation layers.".to_string(),
                    remediation: "Carefully validate and sanitize any user-provided input that is passed to a language model. Implement strict input filters and output parsing.".to_string(),
                    code_snippet: line.trim().to_string(),
                    confidence: 0.75,
                };
                vulnerabilities.push(vuln);
            }
        }
    }

    Ok(vulnerabilities)
}
