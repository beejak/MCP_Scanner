use crate::detectors::Detector;
use crate::models::vulnerability::{Vulnerability, VulnerabilityType, Severity, Evidence};
use anyhow::Result;
use regex::Regex;
use once_cell::sync::Lazy;

/// Detector for secrets and credentials leakage
pub struct SecretsDetector {
    patterns: Vec<SecretPattern>,
}

struct SecretPattern {
    name: &'static str,
    regex: Regex,
    severity: Severity,
    description: &'static str,
}

static SECRET_PATTERNS: Lazy<Vec<(&'static str, &'static str, Severity, &'static str)>> = Lazy::new(|| {
    vec![
        // SSH Keys
        (
            r"-----BEGIN (RSA|DSA|EC|OPENSSH) PRIVATE KEY-----",
            "SSH Private Key",
            Severity::Critical,
            "SSH private key detected",
        ),
        (
            r"ssh-rsa\s+[A-Za-z0-9+/]{200,}",
            "SSH Public Key (potential leak)",
            Severity::High,
            "SSH public key detected",
        ),
        // AWS Keys
        (
            r"(A3T[A-Z0-9]|AKIA|AGPA|AIDA|AROA|AIPA|ANPA|ANVA|ASIA)[A-Z0-9]{16}",
            "AWS Access Key ID",
            Severity::Critical,
            "AWS access key ID detected",
        ),
        (
            r"(?i)aws.{0,20}?['\"][0-9a-zA-Z/+=]{40}['\"]",
            "AWS Secret Access Key",
            Severity::Critical,
            "AWS secret access key detected",
        ),
        // API Keys (Generic)
        (
            r"(?i)(api[_-]?key|apikey|api[_-]?secret)['\"]?\s*[:=]\s*['\"]?[a-zA-Z0-9_\-]{20,}['\"]?",
            "Generic API Key",
            Severity::High,
            "API key detected",
        ),
        // GitHub Tokens
        (
            r"ghp_[a-zA-Z0-9]{36}",
            "GitHub Personal Access Token",
            Severity::Critical,
            "GitHub personal access token detected",
        ),
        (
            r"gho_[a-zA-Z0-9]{36}",
            "GitHub OAuth Access Token",
            Severity::Critical,
            "GitHub OAuth token detected",
        ),
        (
            r"github_pat_[a-zA-Z0-9]{22}_[a-zA-Z0-9]{59}",
            "GitHub Fine-grained Personal Access Token",
            Severity::Critical,
            "GitHub fine-grained PAT detected",
        ),
        // Anthropic API Keys
        (
            r"sk-ant-api03-[a-zA-Z0-9\-_]{93,}",
            "Anthropic API Key",
            Severity::Critical,
            "Anthropic API key detected",
        ),
        // OpenAI API Keys
        (
            r"sk-[a-zA-Z0-9]{48}",
            "OpenAI API Key",
            Severity::Critical,
            "OpenAI API key detected",
        ),
        // Google API Keys
        (
            r"AIza[0-9A-Za-z\-_]{35}",
            "Google API Key",
            Severity::High,
            "Google API key detected",
        ),
        // Slack Tokens
        (
            r"xox[baprs]-[0-9a-zA-Z]{10,48}",
            "Slack Token",
            Severity::High,
            "Slack token detected",
        ),
        // JWT Tokens
        (
            r"eyJ[a-zA-Z0-9_-]*\.eyJ[a-zA-Z0-9_-]*\.[a-zA-Z0-9_-]*",
            "JWT Token",
            Severity::Medium,
            "JWT token detected",
        ),
        // Private keys in various formats
        (
            r"-----BEGIN PRIVATE KEY-----",
            "Generic Private Key",
            Severity::Critical,
            "Generic private key detected",
        ),
        (
            r"-----BEGIN PGP PRIVATE KEY BLOCK-----",
            "PGP Private Key",
            Severity::Critical,
            "PGP private key detected",
        ),
        // Database Connection Strings
        (
            r"(?i)(postgres|mysql|mongodb)://[a-zA-Z0-9_\-]+:[a-zA-Z0-9_\-]+@",
            "Database Connection String",
            Severity::Critical,
            "Database connection string with credentials",
        ),
        // Password patterns
        (
            r"(?i)(password|passwd|pwd)['\"]?\s*[:=]\s*['\"][^'\"]{8,}['\"]",
            "Hardcoded Password",
            Severity::High,
            "Hardcoded password detected",
        ),
        // Stripe Keys
        (
            r"sk_live_[0-9a-zA-Z]{24}",
            "Stripe Live Secret Key",
            Severity::Critical,
            "Stripe live secret key detected",
        ),
        // Twilio
        (
            r"SK[a-z0-9]{32}",
            "Twilio API Key",
            Severity::High,
            "Twilio API key detected",
        ),
        // Azure
        (
            r"(?i)azure.{0,20}?['\"][0-9a-zA-Z/+=]{40,}['\"]",
            "Azure Secret",
            Severity::Critical,
            "Azure secret detected",
        ),
        // Heroku
        (
            r"[h|H][e|E][r|R][o|O][k|K][u|U].*[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}",
            "Heroku API Key",
            Severity::High,
            "Heroku API key detected",
        ),
        // Generic Base64-encoded secrets (high entropy)
        (
            r"(?i)(secret|token|key)['\"]?\s*[:=]\s*['\"]?[A-Za-z0-9+/]{40,}={0,2}['\"]?",
            "Potential Base64-encoded Secret",
            Severity::Medium,
            "High-entropy base64-like string detected",
        ),
    ]
});

impl SecretsDetector {
    pub fn new() -> Self {
        let patterns = SECRET_PATTERNS
            .iter()
            .map(|(pattern, name, severity, description)| SecretPattern {
                name,
                regex: Regex::new(pattern).unwrap(),
                severity: *severity,
                description,
            })
            .collect();

        Self { patterns }
    }

    /// Calculate entropy of a string (for detecting high-entropy secrets)
    fn calculate_entropy(s: &str) -> f64 {
        use std::collections::HashMap;

        if s.is_empty() {
            return 0.0;
        }

        let mut char_counts: HashMap<char, usize> = HashMap::new();
        for c in s.chars() {
            *char_counts.entry(c).or_insert(0) += 1;
        }

        let len = s.len() as f64;
        let mut entropy = 0.0;

        for &count in char_counts.values() {
            let probability = count as f64 / len;
            entropy -= probability * probability.log2();
        }

        entropy
    }

    /// Check for sensitive file paths in content
    pub fn check_sensitive_paths(&self, content: &str) -> Vec<Vulnerability> {
        let mut vulnerabilities = Vec::new();

        let sensitive_paths = vec![
            (r"~?/\.ssh/(id_rsa|id_ed25519|id_ecdsa)", "SSH private key path"),
            (r"~?/\.aws/credentials", "AWS credentials file path"),
            (r"~?/\.config/gcloud/", "Google Cloud config path"),
            (r"~?/\.kube/config", "Kubernetes config path"),
            (r"~?/\.docker/config\.json", "Docker config path"),
            (r"\.env", "Environment file path"),
            (r"secrets?\.(json|yaml|yml|toml)", "Secrets file path"),
        ];

        for (pattern, description) in sensitive_paths {
            let regex = Regex::new(pattern).unwrap();
            if regex.is_match(content) {
                vulnerabilities.push(
                    Vulnerability::new(
                        format!("SEC-PATH-{}", vulnerabilities.len() + 1),
                        VulnerabilityType::SensitiveDataExposure,
                        Severity::High,
                        format!("Sensitive file path reference: {}", description),
                        format!("Code references sensitive file path: {}", description),
                    )
                    .with_impact("May lead to exposure of credentials or sensitive data")
                    .with_remediation("Avoid hardcoding sensitive file paths. Use configuration or environment variables."),
                );
            }
        }

        vulnerabilities
    }
}

impl Default for SecretsDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for SecretsDetector {
    fn name(&self) -> &'static str {
        "SecretsDetector"
    }

    fn scan(&self, content: &str, file_path: Option<&str>) -> Result<Vec<Vulnerability>> {
        let mut vulnerabilities = Vec::new();
        let mut vuln_counter = 1;

        // Scan for secret patterns
        for pattern in &self.patterns {
            for (line_num, line) in content.lines().enumerate() {
                if let Some(captures) = pattern.regex.captures(line) {
                    let matched_text = captures.get(0).map(|m| m.as_str()).unwrap_or("");

                    // Redact the actual secret in output
                    let redacted = if matched_text.len() > 10 {
                        format!("{}...{}", &matched_text[..5], &matched_text[matched_text.len()-5..])
                    } else {
                        "[REDACTED]".to_string()
                    };

                    let id = format!("SEC-{:03}", vuln_counter);
                    vuln_counter += 1;

                    let mut vuln = Vulnerability::new(
                        id,
                        VulnerabilityType::SecretsLeakage,
                        pattern.severity,
                        format!("Secret Detected: {}", pattern.name),
                        format!(
                            "{} found in code. Line: {}",
                            pattern.description,
                            line_num + 1
                        ),
                    )
                    .with_impact(
                        "Exposed secrets can lead to unauthorized access to systems, data breaches, and compromise of services."
                            .to_string(),
                    )
                    .with_remediation(
                        "Remove the hardcoded secret. Use environment variables, secret management systems (e.g., HashiCorp Vault, AWS Secrets Manager), or secure configuration files.".to_string(),
                    )
                    .with_evidence(Evidence {
                        snippet: Some(format!("[SECRET REDACTED: {}]", redacted)),
                        context: serde_json::json!({
                            "line_number": line_num + 1,
                            "secret_type": pattern.name,
                            "entropy": Self::calculate_entropy(matched_text),
                        }),
                    })
                    .with_cwe(798); // CWE-798: Use of Hard-coded Credentials

                    if let Some(path) = file_path {
                        vuln = vuln.with_location(path.to_string(), Some(line_num + 1), None);
                    }

                    vulnerabilities.push(vuln);
                }
            }
        }

        // Check for sensitive file paths
        vulnerabilities.extend(self.check_sensitive_paths(content));

        Ok(vulnerabilities)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_ssh_private_key() {
        let detector = SecretsDetector::new();

        let content = "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQ...";
        let vulns = detector.scan(content, None).unwrap();

        assert!(!vulns.is_empty());
        assert_eq!(vulns[0].severity, Severity::Critical);
        assert!(vulns[0].description.contains("SSH private key"));
    }

    #[test]
    fn test_detect_aws_key() {
        let detector = SecretsDetector::new();

        let content = "AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE";
        let vulns = detector.scan(content, None).unwrap();

        assert!(!vulns.is_empty());
        assert_eq!(vulns[0].severity, Severity::Critical);
    }

    #[test]
    fn test_detect_github_token() {
        let detector = SecretsDetector::new();

        let content = "token = 'ghp_1234567890123456789012345678901234AB'";
        let vulns = detector.scan(content, None).unwrap();

        assert!(!vulns.is_empty());
        assert_eq!(vulns[0].severity, Severity::Critical);
    }

    #[test]
    fn test_detect_anthropic_key() {
        let detector = SecretsDetector::new();

        let content = "ANTHROPIC_API_KEY=sk-ant-api03-abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789abcdefghijklmnopqrstu";
        let vulns = detector.scan(content, None).unwrap();

        assert!(!vulns.is_empty());
        assert_eq!(vulns[0].severity, Severity::Critical);
    }

    #[test]
    fn test_detect_database_connection_string() {
        let detector = SecretsDetector::new();

        let content = "DATABASE_URL=postgres://user:password@localhost:5432/dbname";
        let vulns = detector.scan(content, None).unwrap();

        assert!(!vulns.is_empty());
        assert_eq!(vulns[0].severity, Severity::Critical);
    }

    #[test]
    fn test_detect_sensitive_path() {
        let detector = SecretsDetector::new();

        let content = "keyfile = '~/.ssh/id_rsa'";
        let vulns = detector.scan(content, None).unwrap();

        assert!(!vulns.is_empty());
        assert!(vulns.iter().any(|v| v.severity == Severity::High));
    }

    #[test]
    fn test_entropy_calculation() {
        // High entropy (random-looking)
        let high_entropy = "aB3xK9mP2qL7nZ4wF";
        assert!(SecretsDetector::calculate_entropy(high_entropy) > 3.0);

        // Low entropy (repetitive)
        let low_entropy = "aaaaaaaaaa";
        assert!(SecretsDetector::calculate_entropy(low_entropy) < 1.0);
    }

    #[test]
    fn test_secret_redaction() {
        let detector = SecretsDetector::new();

        let content = "api_key = 'very_secret_key_12345678'";
        let vulns = detector.scan(content, None).unwrap();

        assert!(!vulns.is_empty());
        // Verify the actual secret is not in the vulnerability description
        let evidence_str = serde_json::to_string(&vulns[0].evidence).unwrap();
        assert!(evidence_str.contains("REDACTED"));
    }
}
