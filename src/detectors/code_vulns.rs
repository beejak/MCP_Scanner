use crate::detectors::Detector;
use crate::models::vulnerability::{Vulnerability, VulnerabilityType, Severity, Evidence};
use anyhow::Result;
use regex::Regex;
use once_cell::sync::Lazy;

/// Detector for code-level vulnerabilities
pub struct CodeVulnsDetector {
    patterns: Vec<VulnPattern>,
}

struct VulnPattern {
    name: &'static str,
    regex: Regex,
    severity: Severity,
    vuln_type: VulnerabilityType,
    cwe: Option<u32>,
}

static CODE_VULN_PATTERNS: Lazy<Vec<(&'static str, &'static str, Severity, VulnerabilityType, Option<u32>)>> = Lazy::new(|| {
    vec![
        // Command Injection
        (
            r"(?i)(os\.system|subprocess\.call|exec|eval|__import__)\s*\(",
            "Command Injection Risk",
            Severity::Critical,
            VulnerabilityType::CommandInjection,
            Some(78),
        ),
        (
            r"(?i)subprocess\.(run|Popen|call).*shell\s*=\s*True",
            "Shell Injection with shell=True",
            Severity::Critical,
            VulnerabilityType::CommandInjection,
            Some(78),
        ),
        // Path Traversal
        (
            r"(?i)(open|read|write|file)\s*\([^)]*\+[^)]*\)",
            "Path Traversal Risk",
            Severity::High,
            VulnerabilityType::PathTraversal,
            Some(22),
        ),
        // SQL Injection
        (
            r#"(?i)(execute|cursor\.execute|db\.query)\s*\([^)]*\+[^)]*\)"#,
            "SQL Injection Risk",
            Severity::Critical,
            VulnerabilityType::SqlInjection,
            Some(89),
        ),
        (
            r#"(?i)(execute|query).*[f"'].*\{.*\}.*["']"#,
            "SQL Injection via f-string",
            Severity::Critical,
            VulnerabilityType::SqlInjection,
            Some(89),
        ),
        // Unsafe Deserialization
        (
            r"(?i)(pickle\.loads|yaml\.load(?!_safe)|marshal\.loads)",
            "Unsafe Deserialization",
            Severity::High,
            VulnerabilityType::UnsafeDeserialization,
            Some(502),
        ),
    ]
});

impl CodeVulnsDetector {
    pub fn new() -> Self {
        let patterns = CODE_VULN_PATTERNS
            .iter()
            .map(|(pattern, name, severity, vuln_type, cwe)| VulnPattern {
                name,
                regex: Regex::new(pattern).unwrap(),
                severity: *severity,
                vuln_type: vuln_type.clone(),
                cwe: *cwe,
            })
            .collect();

        Self { patterns }
    }
}

impl Default for CodeVulnsDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for CodeVulnsDetector {
    fn name(&self) -> &'static str {
        "CodeVulnsDetector"
    }

    fn scan(&self, content: &str, file_path: Option<&str>) -> Result<Vec<Vulnerability>> {
        let mut vulnerabilities = Vec::new();
        let mut vuln_counter = 1;

        for pattern in &self.patterns {
            for (line_num, line) in content.lines().enumerate() {
                if let Some(captures) = pattern.regex.captures(line) {
                    let matched_text = captures.get(0).map(|m| m.as_str()).unwrap_or("");

                    let id = format!("CV-{:03}", vuln_counter);
                    vuln_counter += 1;

                    let mut vuln = Vulnerability::new(
                        id,
                        pattern.vuln_type.clone(),
                        pattern.severity,
                        pattern.name.to_string(),
                        format!(
                            "{} detected at line {}",
                            pattern.name,
                            line_num + 1
                        ),
                    )
                    .with_impact("May allow attackers to execute arbitrary code or access sensitive data")
                    .with_remediation("Use safe alternatives and proper input validation")
                    .with_evidence(Evidence {
                        snippet: Some(line.trim().to_string()),
                        context: serde_json::json!({
                            "line_number": line_num + 1,
                            "matched_text": matched_text,
                        }),
                    });

                    if let Some(cwe) = pattern.cwe {
                        vuln = vuln.with_cwe(cwe);
                    }

                    if let Some(path) = file_path {
                        vuln = vuln.with_location(path.to_string(), Some(line_num + 1), None);
                    }

                    vulnerabilities.push(vuln);
                }
            }
        }

        Ok(vulnerabilities)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_command_injection() {
        let detector = CodeVulnsDetector::new();

        let content = "os.system(f'ls {user_input}')";
        let vulns = detector.scan(content, None).unwrap();

        assert!(!vulns.is_empty());
        assert_eq!(vulns[0].severity, Severity::Critical);
    }

    #[test]
    fn test_detect_unsafe_deserialization() {
        let detector = CodeVulnsDetector::new();

        let content = "data = pickle.loads(user_data)";
        let vulns = detector.scan(content, None).unwrap();

        assert!(!vulns.is_empty());
        assert_eq!(vulns[0].severity, Severity::High);
    }
}
