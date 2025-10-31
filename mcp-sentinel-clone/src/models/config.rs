use serde::{Serialize, Deserialize};

// The final, validated configuration struct used by the scanner.
#[derive(Debug, Serialize, Clone)]
pub struct ScanConfig {
    pub exclude_patterns: Vec<String>,
}

// A struct that represents the raw, partial configuration loaded from a YAML file.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RawScanConfig {
    pub exclude_patterns: Option<Vec<String>>,
}

impl ScanConfig {
    // Merges the raw, partial config into a complete ScanConfig.
    pub fn merge(mut self, raw: RawScanConfig) -> Self {
        if let Some(exclude_patterns) = raw.exclude_patterns {
            self.exclude_patterns = exclude_patterns;
        }
        self
    }
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            exclude_patterns: vec![
                "node_modules/".to_string(),
                ".git/".to_string(),
                "target/".to_string(),
                "dist/".to_string(),
            ],
        }
    }
}
