//! Output formatting for scan results.
//!
//! This module provides various output formats for displaying vulnerability scan results.
//! All formatters implement the [`OutputFormatter`] trait, providing a consistent interface
//! for rendering [`ScanResult`] data.
//!
//! # Available Formats
//!
//! - **Terminal**: Human-readable colored output for CLI usage (see [`terminal::TerminalFormatter`])
//! - **JSON**: Machine-readable JSON for CI/CD integration (see [`json::JsonFormatter`])
//! - **HTML**: Interactive web report *(Coming Soon)*
//! - **PDF**: Printable PDF report *(Coming Soon)*
//! - **SARIF**: GitHub Security integration format *(Coming Soon)*
//!
//! # Examples
//!
//! ## Terminal Output
//!
//! ```no_run
//! use mcp_sentinel::output::{OutputFormatter, terminal::TerminalFormatter};
//! use mcp_sentinel::models::scan_result::ScanResult;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let formatter = TerminalFormatter::new();
//! # let result = todo!();
//! let output = formatter.output(&result)?;
//! println!("{}", output);
//! # Ok(())
//! # }
//! ```
//!
//! ## JSON Output for CI/CD
//!
//! ```no_run
//! use mcp_sentinel::output::{OutputFormatter, json::JsonFormatter};
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let formatter = JsonFormatter::new();
//! # let result = todo!();
//! formatter.save_to_file(&result, Path::new("scan-results.json"))?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Selecting Format at Runtime
//!
//! ```
//! use mcp_sentinel::output::OutputFormat;
//!
//! let format = OutputFormat::from_str("json").unwrap();
//! match format {
//!     OutputFormat::Json => println!("Using JSON output"),
//!     OutputFormat::Terminal => println!("Using Terminal output"),
//!     _ => println!("Other format"),
//! }
//! ```

pub mod terminal;
pub mod json;
pub mod html;
pub mod pdf;
pub mod sarif;

use crate::models::scan_result::ScanResult;
use anyhow::Result;
use std::path::Path;

/// Output format types for scan results.
///
/// Determines how vulnerability scan results are presented to the user or
/// integrated with other tools.
///
/// # Format Selection
///
/// - **Terminal**: Best for interactive CLI usage and human review
/// - **JSON**: Best for automation, CI/CD pipelines, and machine processing
/// - **HTML**: Best for sharing reports with non-technical stakeholders *(Coming Soon)*
/// - **PDF**: Best for compliance documentation and archival *(Coming Soon)*
/// - **SARIF**: Best for GitHub Security tab integration *(Coming Soon)*
#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    /// Colored terminal output with emojis and formatting
    Terminal,
    /// JSON output for machine consumption
    Json,
    /// HTML web report *(Coming Soon)*
    Html,
    /// PDF document *(Coming Soon)*
    Pdf,
    /// SARIF format for GitHub Security *(Coming Soon)*
    Sarif,
}

impl OutputFormat {
    /// Parse an output format from a string.
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp_sentinel::output::OutputFormat;
    ///
    /// assert!(matches!(OutputFormat::from_str("json"), Some(OutputFormat::Json)));
    /// assert!(matches!(OutputFormat::from_str("terminal"), Some(OutputFormat::Terminal)));
    /// assert!(OutputFormat::from_str("invalid").is_none());
    /// ```
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "terminal" => Some(OutputFormat::Terminal),
            "json" => Some(OutputFormat::Json),
            "html" => Some(OutputFormat::Html),
            "pdf" => Some(OutputFormat::Pdf),
            "sarif" => Some(OutputFormat::Sarif),
            _ => None,
        }
    }
}

/// Trait for formatting and outputting scan results.
///
/// All output formatters must implement this trait, providing a consistent
/// interface for rendering [`ScanResult`] data in different formats.
///
/// # Implementation Notes
///
/// - The `output` method returns a String containing the formatted result
/// - The default `save_to_file` implementation handles file writing
/// - Implementations should handle errors gracefully and return Result
///
/// # Examples
///
/// Implementing a custom formatter:
///
/// ```
/// use mcp_sentinel::output::OutputFormatter;
/// use mcp_sentinel::models::scan_result::ScanResult;
/// use anyhow::Result;
///
/// struct CustomFormatter;
///
/// impl OutputFormatter for CustomFormatter {
///     fn output(&self, result: &ScanResult) -> Result<String> {
///         Ok(format!("Found {} issues in {}", 
///             result.summary.total_issues,
///             result.target))
///     }
/// }
/// ```
pub trait OutputFormatter {
    /// Format scan results as a string.
    ///
    /// # Arguments
    ///
    /// * `result` - The scan result to format
    ///
    /// # Returns
    ///
    /// A formatted string representation of the scan results, or an error
    /// if formatting fails.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Serialization fails (for JSON/SARIF formats)
    /// - Template rendering fails (for HTML/PDF formats)
    /// - Any I/O operations fail
    fn output(&self, result: &ScanResult) -> Result<String>;

    /// Save formatted output to a file.
    ///
    /// This default implementation calls `output()` and writes the result
    /// to the specified file path. Formatters can override this if they
    /// need custom file-writing logic.
    ///
    /// # Arguments
    ///
    /// * `result` - The scan result to format and save
    /// * `path` - The file path to write to
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mcp_sentinel::output::{OutputFormatter, json::JsonFormatter};
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let formatter = JsonFormatter::new();
    /// # let result = todo!();
    /// formatter.save_to_file(&result, Path::new("output.json"))?;
    /// # Ok(())
    /// # }
    /// ```
    fn save_to_file(&self, result: &ScanResult, path: &Path) -> Result<()> {
        let output = self.output(result)?;
        std::fs::write(path, output)?;
        Ok(())
    }
}
