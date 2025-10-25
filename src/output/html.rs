use crate::models::scan_result::ScanResult;
use crate::output::OutputFormatter;
use anyhow::Result;

/// HTML output formatter
pub struct HtmlFormatter {}

impl HtmlFormatter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for HtmlFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter for HtmlFormatter {
    fn output(&self, _result: &ScanResult) -> Result<String> {
        // TODO: Implement HTML report generation with handlebars
        Ok("<html><body><h1>MCP Sentinel Report</h1><p>HTML report generation not yet implemented</p></body></html>".to_string())
    }
}
