use crate::models::scan_result::ScanResult;
use crate::output::OutputFormatter;
use anyhow::Result;

/// PDF output formatter
pub struct PdfFormatter {}

impl PdfFormatter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for PdfFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter for PdfFormatter {
    fn output(&self, _result: &ScanResult) -> Result<String> {
        // TODO: Implement PDF report generation
        anyhow::bail!("PDF output format not yet implemented")
    }
}
