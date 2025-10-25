use crate::models::scan_result::ScanResult;
use crate::output::OutputFormatter;
use anyhow::Result;

/// JSON output formatter
pub struct JsonFormatter {
    pretty: bool,
}

impl JsonFormatter {
    pub fn new() -> Self {
        Self { pretty: true }
    }

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
