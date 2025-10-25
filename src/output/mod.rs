pub mod terminal;
pub mod json;
pub mod html;
pub mod pdf;
pub mod sarif;

use crate::models::scan_result::ScanResult;
use anyhow::Result;
use std::path::Path;

/// Output format types
#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Terminal,
    Json,
    Html,
    Pdf,
    Sarif,
}

impl OutputFormat {
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

/// Trait for outputting scan results
pub trait OutputFormatter {
    /// Format and output scan results
    fn output(&self, result: &ScanResult) -> Result<String>;

    /// Save output to file
    fn save_to_file(&self, result: &ScanResult, path: &Path) -> Result<()> {
        let output = self.output(result)?;
        std::fs::write(path, output)?;
        Ok(())
    }
}
