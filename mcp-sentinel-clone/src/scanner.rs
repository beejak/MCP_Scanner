use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::fs;
use tokio::task::JoinHandle;
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

    pub async fn scan_directory(&self, path: &str) -> Result<ScanResult> {
        let start_time = Instant::now();
        let mut result = ScanResult {
            target: path.to_string(),
            engines: vec!["static".to_string()],
            vulnerabilities: vec![],
            scan_duration_ms: 0,
        };

        let mut tasks: Vec<JoinHandle<Result<Vec<Vulnerability>>>> = vec![];
        let walker = WalkDir::new(path).into_iter();

        for entry in walker.filter_entry(|e| !self.is_excluded(e.path())) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let path_buf = entry.path().to_path_buf();
                tasks.push(tokio::spawn(async move {
                    scan_file(path_buf).await
                }));
            }
        }

        for task in tasks {
            let res = task.await??;
            result.vulnerabilities.extend(res);
        }

        result.scan_duration_ms = start_time.elapsed().as_millis() as u64;
        Ok(result)
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

async fn scan_file(path: PathBuf) -> Result<Vec<Vulnerability>> {
    let content = match fs::read_to_string(&path).await {
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
