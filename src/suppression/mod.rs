//! Vulnerability Suppression System
//!
//! Manages suppression of false positives and accepted risks through
//! configuration files and inline annotations.
//!
//! # Features
//!
//! - YAML-based suppression rules
//! - Pattern matching (glob, regex, exact)
//! - Expiration dates for temporary suppressions
//! - Audit logging of suppressed vulnerabilities
//! - Justification requirements
//! - Team-wide suppressions via config files
//!
//! # Suppression File Format
//!
//! `.mcp-sentinel-ignore.yaml`:
//!
//! ```yaml
//! version: "1.0"
//! suppressions:
//!   - id: "SUP-001"
//!     reason: "False positive - this is test data"
//!     author: "john@example.com"
//!     date: "2025-01-15"
//!     expires: "2025-07-15"
//!     patterns:
//!       - type: "glob"
//!         value: "tests/**/*.py"
//!       - type: "vuln_type"
//!         value: "secrets_leakage"
//!
//!   - id: "SUP-002"
//!     reason: "Accepted risk - credentials rotated weekly"
//!     patterns:
//!       - type: "file"
//!         value: "config/legacy_auth.py"
//!       - type: "line"
//!         value: 42
//! ```
//!
//! # Usage
//!
//! ```rust
//! use mcp_sentinel::suppression::SuppressionManager;
//!
//! # fn main() -> anyhow::Result<()> {
//! let manager = SuppressionManager::load(".mcp-sentinel-ignore.yaml")?;
//!
//! // Check if vulnerability should be suppressed
//! if manager.should_suppress(&vulnerability)? {
//!     println!("Vulnerability suppressed");
//! }
//! # Ok(())
//! # }
//! ```

pub mod auditor;
pub mod matcher;
pub mod parser;

use crate::models::Vulnerability;
use anyhow::Result;
use std::path::Path;
use tracing::{debug, info};

pub use auditor::SuppressionAuditor;
pub use matcher::SuppressionMatcher;
pub use parser::{Suppression, SuppressionConfig, SuppressionPattern, PatternType};

/// Main suppression manager
pub struct SuppressionManager {
    /// Loaded suppression configuration
    config: SuppressionConfig,

    /// Pattern matcher
    matcher: SuppressionMatcher,

    /// Audit logger
    auditor: SuppressionAuditor,
}

impl SuppressionManager {
    /// Load suppression configuration from file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to suppression config file
    ///
    /// # Returns
    ///
    /// Initialized manager
    ///
    /// # Errors
    ///
    /// - File not found
    /// - Invalid YAML
    /// - Invalid configuration
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        info!("Loading suppression config from: {}", path.as_ref().display());

        let config = parser::load_suppression_config(path.as_ref())?;
        let matcher = SuppressionMatcher::new();
        let auditor = SuppressionAuditor::new()?;

        debug!("Loaded {} suppression rules", config.suppressions.len());

        Ok(Self {
            config,
            matcher,
            auditor,
        })
    }

    /// Create empty suppression manager (no suppressions)
    pub fn empty() -> Result<Self> {
        Ok(Self {
            config: SuppressionConfig {
                version: "1.0".to_string(),
                suppressions: vec![],
            },
            matcher: SuppressionMatcher::new(),
            auditor: SuppressionAuditor::new()?,
        })
    }

    /// Check if a vulnerability should be suppressed
    ///
    /// # Arguments
    ///
    /// * `vuln` - Vulnerability to check
    ///
    /// # Returns
    ///
    /// true if vulnerability should be suppressed
    pub fn should_suppress(&self, vuln: &Vulnerability) -> Result<bool> {
        for suppression in &self.config.suppressions {
            // Skip expired suppressions
            if suppression.is_expired() {
                continue;
            }

            // Check if this suppression matches the vulnerability
            if self.matcher.matches(suppression, vuln)? {
                // Log the suppression
                self.auditor.log_suppression(suppression, vuln)?;

                debug!(
                    "Vulnerability suppressed: {} (rule: {})",
                    vuln.vuln_type.name(),
                    suppression.id
                );

                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Filter vulnerabilities, removing suppressed ones
    ///
    /// # Arguments
    ///
    /// * `vulnerabilities` - List of vulnerabilities to filter
    ///
    /// # Returns
    ///
    /// Filtered list with suppressions removed
    pub fn filter_suppressions(
        &self,
        vulnerabilities: Vec<Vulnerability>,
    ) -> Result<Vec<Vulnerability>> {
        let original_count = vulnerabilities.len();

        let filtered: Vec<Vulnerability> = vulnerabilities
            .into_iter()
            .filter(|v| !self.should_suppress(v).unwrap_or(false))
            .collect();

        let suppressed_count = original_count - filtered.len();

        if suppressed_count > 0 {
            info!(
                "Suppressed {} vulnerabilities ({} remaining)",
                suppressed_count,
                filtered.len()
            );
        }

        Ok(filtered)
    }

    /// Get list of active suppressions
    pub fn get_active_suppressions(&self) -> Vec<&Suppression> {
        self.config
            .suppressions
            .iter()
            .filter(|s| !s.is_expired())
            .collect()
    }

    /// Get list of expired suppressions
    pub fn get_expired_suppressions(&self) -> Vec<&Suppression> {
        self.config
            .suppressions
            .iter()
            .filter(|s| s.is_expired())
            .collect()
    }

    /// Get suppression statistics
    pub fn get_stats(&self) -> SuppressionStats {
        SuppressionStats {
            total: self.config.suppressions.len(),
            active: self.get_active_suppressions().len(),
            expired: self.get_expired_suppressions().len(),
        }
    }
}

/// Results of filtering with suppression information
#[derive(Debug, Clone)]
pub struct FilteredResults {
    /// Vulnerabilities that passed suppression filters
    pub active_vulnerabilities: Vec<Vulnerability>,

    /// Vulnerabilities that were suppressed
    pub suppressed_vulnerabilities: Vec<VulnerabilityWithReason>,
}

/// Vulnerability with suppression reason attached
#[derive(Debug, Clone)]
pub struct VulnerabilityWithReason {
    /// The vulnerability that was suppressed
    pub vulnerability: Vulnerability,

    /// Reason for suppression
    pub suppression_reason: String,

    /// ID of suppression rule that matched
    pub suppression_id: String,

    /// Author of suppression
    pub suppression_author: Option<String>,
}

/// Suppression statistics
#[derive(Debug, Clone)]
pub struct SuppressionStats {
    /// Total number of suppressions
    pub total: usize,

    /// Number of active suppressions
    pub active: usize,

    /// Number of expired suppressions
    pub expired: usize,
}

impl SuppressionStats {
    /// Format as human-readable string
    pub fn format(&self) -> String {
        format!(
            "Suppressions: {} total ({} active, {} expired)",
            self.total, self.active, self.expired
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_manager() {
        let manager = SuppressionManager::empty().unwrap();
        let stats = manager.get_stats();

        assert_eq!(stats.total, 0);
        assert_eq!(stats.active, 0);
        assert_eq!(stats.expired, 0);
    }

    #[test]
    fn test_stats_format() {
        let stats = SuppressionStats {
            total: 10,
            active: 8,
            expired: 2,
        };

        let formatted = stats.format();
        assert!(formatted.contains("10 total"));
        assert!(formatted.contains("8 active"));
        assert!(formatted.contains("2 expired"));
    }
}
