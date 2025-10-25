use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::models::vulnerability::{Vulnerability, Severity};

/// Summary statistics for scan results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub total_issues: usize,
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub risk_score: u32,
}

impl Summary {
    /// Calculate summary from vulnerabilities
    pub fn from_vulnerabilities(vulnerabilities: &[Vulnerability]) -> Self {
        let total_issues = vulnerabilities.len();
        let critical = vulnerabilities.iter().filter(|v| v.severity == Severity::Critical).count();
        let high = vulnerabilities.iter().filter(|v| v.severity == Severity::High).count();
        let medium = vulnerabilities.iter().filter(|v| v.severity == Severity::Medium).count();
        let low = vulnerabilities.iter().filter(|v| v.severity == Severity::Low).count();

        // Calculate risk score (0-100)
        // Formula: (critical * 40 + high * 20 + medium * 10 + low * 5) capped at 100
        let risk_score = std::cmp::min(
            100,
            (critical * 40 + high * 20 + medium * 10 + low * 5) as u32,
        );

        Self {
            total_issues,
            critical,
            high,
            medium,
            low,
            risk_score,
        }
    }

    /// Get risk level description
    pub fn risk_level(&self) -> &'static str {
        match self.risk_score {
            0..=20 => "Low",
            21..=40 => "Medium",
            41..=70 => "High",
            _ => "Critical",
        }
    }

    /// Check if there are any issues at or above given severity
    pub fn has_issues_at_or_above(&self, severity: Severity) -> bool {
        match severity {
            Severity::Critical => self.critical > 0,
            Severity::High => self.critical > 0 || self.high > 0,
            Severity::Medium => self.critical > 0 || self.high > 0 || self.medium > 0,
            Severity::Low => self.total_issues > 0,
        }
    }
}

/// Metadata about the scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub scan_duration_ms: u64,
    pub engines_used: Vec<String>,
    pub llm_provider: Option<String>,
    pub llm_model: Option<String>,
}

/// Complete scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// Version of mcp-sentinel
    pub version: String,

    /// Unique scan ID
    pub scan_id: String,

    /// Timestamp when scan started
    pub timestamp: DateTime<Utc>,

    /// Target that was scanned
    pub target: String,

    /// Engines used for scanning
    pub engines: Vec<String>,

    /// Summary statistics
    pub summary: Summary,

    /// List of vulnerabilities found
    pub vulnerabilities: Vec<Vulnerability>,

    /// Scan metadata
    pub metadata: Metadata,
}

impl ScanResult {
    /// Create a new scan result
    pub fn new(
        target: String,
        engines: Vec<String>,
        vulnerabilities: Vec<Vulnerability>,
        metadata: Metadata,
    ) -> Self {
        let summary = Summary::from_vulnerabilities(&vulnerabilities);

        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            scan_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            target,
            engines,
            summary,
            vulnerabilities,
            metadata,
        }
    }

    /// Filter vulnerabilities by minimum severity
    pub fn filter_by_severity(mut self, min_severity: Severity) -> Self {
        self.vulnerabilities.retain(|v| v.severity >= min_severity);
        self.summary = Summary::from_vulnerabilities(&self.vulnerabilities);
        self
    }

    /// Get highest severity found
    pub fn highest_severity(&self) -> Option<Severity> {
        self.vulnerabilities
            .iter()
            .map(|v| v.severity)
            .max()
    }

    /// Group vulnerabilities by type
    pub fn group_by_type(&self) -> std::collections::HashMap<String, Vec<&Vulnerability>> {
        let mut groups: std::collections::HashMap<String, Vec<&Vulnerability>> = std::collections::HashMap::new();

        for vuln in &self.vulnerabilities {
            groups
                .entry(vuln.vuln_type.to_string())
                .or_default()
                .push(vuln);
        }

        groups
    }

    /// Group vulnerabilities by severity
    pub fn group_by_severity(&self) -> std::collections::HashMap<Severity, Vec<&Vulnerability>> {
        let mut groups: std::collections::HashMap<Severity, Vec<&Vulnerability>> = std::collections::HashMap::new();

        for vuln in &self.vulnerabilities {
            groups
                .entry(vuln.severity)
                .or_default()
                .push(vuln);
        }

        groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::vulnerability::VulnerabilityType;

    fn create_test_vulnerability(severity: Severity) -> Vulnerability {
        Vulnerability::new(
            "TEST-001".to_string(),
            VulnerabilityType::ToolPoisoning,
            severity,
            "Test".to_string(),
            "Test description".to_string(),
        )
    }

    #[test]
    fn test_summary_calculation() {
        let vulns = vec![
            create_test_vulnerability(Severity::Critical),
            create_test_vulnerability(Severity::Critical),
            create_test_vulnerability(Severity::High),
            create_test_vulnerability(Severity::Medium),
            create_test_vulnerability(Severity::Low),
        ];

        let summary = Summary::from_vulnerabilities(&vulns);
        assert_eq!(summary.total_issues, 5);
        assert_eq!(summary.critical, 2);
        assert_eq!(summary.high, 1);
        assert_eq!(summary.medium, 1);
        assert_eq!(summary.low, 1);
        assert_eq!(summary.risk_score, 100); // 2*40 + 1*20 + 1*10 + 1*5 = 115, capped at 100
    }

    #[test]
    fn test_risk_level() {
        let summary = Summary {
            total_issues: 1,
            critical: 0,
            high: 0,
            medium: 0,
            low: 1,
            risk_score: 15,
        };
        assert_eq!(summary.risk_level(), "Low");

        let summary2 = Summary {
            total_issues: 2,
            critical: 2,
            high: 0,
            medium: 0,
            low: 0,
            risk_score: 80,
        };
        assert_eq!(summary2.risk_level(), "Critical");
    }

    #[test]
    fn test_has_issues_at_or_above() {
        let summary = Summary {
            total_issues: 3,
            critical: 0,
            high: 1,
            medium: 1,
            low: 1,
            risk_score: 35,
        };

        assert!(summary.has_issues_at_or_above(Severity::Low));
        assert!(summary.has_issues_at_or_above(Severity::Medium));
        assert!(summary.has_issues_at_or_above(Severity::High));
        assert!(!summary.has_issues_at_or_above(Severity::Critical));
    }
}
