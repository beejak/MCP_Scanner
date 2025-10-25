//! JSON output formatter for machine-readable results.
//!
//! This module provides JSON serialization of scan results, optimized for
//! CI/CD pipelines, automation, and integration with other tools.
//!
//! # Features
//!
//! - **Machine-readable**: Structured JSON output for parsing
//! - **CI/CD friendly**: Exit codes and structured data for automation
//! - **Pretty printing**: Optional formatting for readability
//! - **Complete data**: All vulnerability details preserved
//! - **Schema stable**: Consistent format across versions
//!
//! # Use Cases
//!
//! - **CI/CD Integration**: Fail builds based on vulnerability severity
//! - **Security Dashboards**: Parse and visualize scan results
//! - **Automated Reporting**: Generate reports from JSON data
//! - **Data Analysis**: Process scan data programmatically
//! - **API Integration**: Send results to security platforms
//!
//! # Examples
//!
//! ## Basic JSON Output
//!
//! ```no_run
//! use mcp_sentinel::output::{OutputFormatter, json::JsonFormatter};
//! # use mcp_sentinel::models::scan_result::ScanResult;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let formatter = JsonFormatter::new();
//! # let result: ScanResult = todo!();
//! let json = formatter.output(&result)?;
//! println!("{}", json);
//! # Ok(())
//! # }
//! ```
//!
//! ## Compact JSON (No Pretty Printing)
//!
//! ```no_run
//! use mcp_sentinel::output::{OutputFormatter, json::JsonFormatter};
//! # use mcp_sentinel::models::scan_result::ScanResult;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let formatter = JsonFormatter::new().with_pretty(false);
//! # let result: ScanResult = todo!();
//! let json = formatter.output(&result)?;
//! println!("{}", json);  // Single-line JSON
//! # Ok(())
//! # }
//! ```
//!
//! ## Save to File for CI/CD
//!
//! ```no_run
//! use mcp_sentinel::output::{OutputFormatter, json::JsonFormatter};
//! use std::path::Path;
//! # use mcp_sentinel::models::scan_result::ScanResult;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let formatter = JsonFormatter::new();
//! # let result: ScanResult = todo!();
//! formatter.save_to_file(&result, Path::new("scan-results.json"))?;
//! # Ok(())
//! # }
//! ```
//!
//! ## CI/CD Integration Example
//!
//! ```bash
//! # Scan and save results
//! mcp-sentinel scan ./mcp-server --format json -o results.json
//!
//! # Parse in CI pipeline
//! critical_count=$(jq '.summary.critical' results.json)
//! if [ "$critical_count" -gt 0 ]; then
//!   echo "Critical vulnerabilities found!"
//!   exit 1
//! fi
//! ```

use crate::models::scan_result::ScanResult;
use crate::output::OutputFormatter;
use anyhow::Result;

/// JSON output formatter for machine-readable scan results.
///
/// Serializes scan results to JSON format with optional pretty-printing.
/// All data from the scan is preserved in the JSON output, making it
/// suitable for programmatic consumption and integration.
///
/// # Format Structure
///
/// The JSON output follows this structure:
///
/// ```json
/// {
///   "target": "./mcp-server",
///   "engines": ["static"],
///   "summary": {
///     "total_issues": 5,
///     "critical": 2,
///     "high": 1,
///     "medium": 1,
///     "low": 1,
///     "risk_score": 75
///   },
///   "vulnerabilities": [
///     {
///       "id": "TP-001",
///       "type": "ToolPoisoning",
///       "severity": "Critical",
///       "title": "Hidden Instruction Detected",
///       "description": "...",
///       "location": {
///         "file": "tools.json",
///         "line": 42
///       },
///       "impact": "...",
///       "remediation": "...",
///       "cwe": 78
///     }
///   ],
///   "metadata": {
///     "scan_duration_ms": 1500,
///     "engines_used": ["static"],
///     "llm_provider": null,
///     "llm_model": null
///   }
/// }
/// ```
///
/// # Examples
///
/// ```no_run
/// use mcp_sentinel::output::{OutputFormatter, json::JsonFormatter};
/// # use mcp_sentinel::models::scan_result::ScanResult;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Pretty-printed JSON (default)
/// let formatter = JsonFormatter::new();
/// # let result: ScanResult = todo!();
/// let json = formatter.output(&result)?;
///
/// // Compact JSON
/// let compact_formatter = JsonFormatter::new().with_pretty(false);
/// let compact_json = compact_formatter.output(&result)?;
/// # Ok(())
/// # }
/// ```
pub struct JsonFormatter {
    pretty: bool,
}

impl JsonFormatter {
    /// Create a new JSON formatter with default settings.
    ///
    /// Defaults to pretty-printed JSON for readability.
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp_sentinel::output::json::JsonFormatter;
    ///
    /// let formatter = JsonFormatter::new();
    /// ```
    pub fn new() -> Self {
        Self { pretty: true }
    }

    /// Set pretty-printing mode.
    ///
    /// When enabled (default), JSON output is formatted with indentation
    /// and newlines for human readability. When disabled, output is compact
    /// single-line JSON suitable for streaming or size optimization.
    ///
    /// # Arguments
    ///
    /// * `pretty` - `true` for pretty-printed JSON, `false` for compact
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp_sentinel::output::json::JsonFormatter;
    ///
    /// // Pretty-printed (default)
    /// let pretty = JsonFormatter::new().with_pretty(true);
    ///
    /// // Compact single-line
    /// let compact = JsonFormatter::new().with_pretty(false);
    /// ```
    pub fn with_pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }
}

impl Default for JsonFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter for JsonFormatter {
    /// Format scan results as JSON.
    ///
    /// Serializes the complete [`ScanResult`] to JSON, including all
    /// vulnerabilities, metadata, and summary statistics.
    ///
    /// # Arguments
    ///
    /// * `result` - The scan result to serialize
    ///
    /// # Returns
    ///
    /// A JSON string representation of the scan results.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails (rare, only if data
    /// structures are corrupted or contain invalid UTF-8).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mcp_sentinel::output::{OutputFormatter, json::JsonFormatter};
    /// # use mcp_sentinel::models::scan_result::ScanResult;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let formatter = JsonFormatter::new();
    /// # let result: ScanResult = todo!();
    /// let json = formatter.output(&result)?;
    /// println!("{}", json);
    /// # Ok(())
    /// # }
    /// ```
    fn output(&self, result: &ScanResult) -> Result<String> {
        let json = if self.pretty {
            serde_json::to_string_pretty(result)?
        } else {
            serde_json::to_string(result)?
        };

        Ok(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::vulnerability::{Vulnerability, VulnerabilityType, Severity};
    use crate::models::scan_result::Metadata;

    #[test]
    fn test_json_formatter() {
        let vuln = Vulnerability::new(
            "TEST-001".to_string(),
            VulnerabilityType::ToolPoisoning,
            Severity::Critical,
            "Test".to_string(),
            "Test description".to_string(),
        );

        let result = ScanResult::new(
            "./test".to_string(),
            vec!["static".to_string()],
            vec![vuln],
            Metadata {
                scan_duration_ms: 1500,
                engines_used: vec!["static".to_string()],
                llm_provider: None,
                llm_model: None,
            },
        );

        let formatter = JsonFormatter::new();
        let output = formatter.output(&result).unwrap();

        // Verify it's valid JSON
        assert!(serde_json::from_str::<serde_json::Value>(&output).is_ok());
        assert!(output.contains("TEST-001"));
    }
}
