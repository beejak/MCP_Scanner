use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanConfig {
    pub exclude_patterns: Vec<String>,
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
