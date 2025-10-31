use anyhow::Result;
use std::fs;
use std::path::Path;
use std::time::Instant;
use walkdir::WalkDir;

use crate::{
    detectors,
    models::{
        config::ScanConfig,
        scan_result::ScanResult,
        vulnerability::Vulnerability,
    },
};

pub struct Scanner {
    config: ScanConfig,
}

impl Scanner {
    pub fn new(config: ScanConfig) -> Self {
        Self { config }
    }

    pub fn scan_directory(&self, path: &str) -> Result<ScanResult> {
        let start_time = Instant::now();
        let mut result = ScanResult {
            target: path.to_string(),
            engines: vec!["static".to_string()],
            vulnerabilities: vec![],
            scan_duration_ms: 0,
        };

        let walker = WalkDir::new(path).into_iter();
        for entry in walker.filter_entry(|e| !self.is_excluded(e.path())) {
            let entry = entry?;
            if entry.file_type().is_file() {
                if let Ok(vulns) = self.scan_file(entry.path()) {
                    result.vulnerabilities.extend(vulns);
                }
            }
        }

        result.scan_duration_ms = start_time.elapsed().as_millis() as u64;
        Ok(result)
    }

    fn scan_file(&self, path: &Path) -> Result<Vec<Vulnerability>> {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Ok(Vec::new()), // Skip files that can't be read as string
        };
        let file_path = path.to_str().unwrap_or_default();

        let mut vulnerabilities = Vec::new();

        vulnerabilities.extend(detectors::secrets::detect(&content, file_path)?);
        vulnerabilities.extend(detectors::command_injection::detect(&content, file_path)?);
        vulnerabilities.extend(detectors::sensitive_file_access::detect(&content, file_path)?);
        vulnerabilities.extend(detectors::tool_poisoning::detect(&content, file_path)?);
        vulnerabilities.extend(detectors::prompt_injection::detect(&content, file_path)?);

        Ok(vulnerabilities)
    }

    fn is_excluded(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        for pattern in &self.config.exclude_patterns {
            if path_str.contains(pattern) {
                return true;
            }
        }
        false
    }
}
