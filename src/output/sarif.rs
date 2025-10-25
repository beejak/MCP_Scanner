use crate::models::scan_result::ScanResult;
use crate::output::OutputFormatter;
use anyhow::Result;

/// SARIF output formatter (Static Analysis Results Interchange Format)
pub struct SarifFormatter {}

impl SarifFormatter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for SarifFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter for SarifFormatter {
    fn output(&self, _result: &ScanResult) -> Result<String> {
        // TODO: Implement SARIF format output for GitHub Security integration
        anyhow::bail!("SARIF output format not yet implemented")
    }
}
