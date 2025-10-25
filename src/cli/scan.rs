//! Scan command implementation

use anyhow::{Context, Result};
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

use super::types::{LlmProvider, OutputFormat, ScanMode, SeverityLevel};
use crate::models::config::ScanConfig;
use crate::scanner::Scanner;

#[allow(clippy::too_many_arguments)]
pub async fn execute(
    target: String,
    mode: ScanMode,
    _llm_provider: Option<LlmProvider>,
    _llm_model: Option<String>,
    _llm_api_key: Option<String>,
    output: OutputFormat,
    output_file: Option<String>,
    _severity: SeverityLevel,
    fail_on: Option<SeverityLevel>,
    _config: Option<String>,
) -> Result<()> {
    info!("📂 Scanning: {}", target);
    debug!("Mode: {:?}", mode);
    debug!("Output format: {:?}", output);

    // Parse target path
    let target_path = PathBuf::from(&target);

    // Check if target exists
    if !target_path.exists() {
        anyhow::bail!(
            "Target path does not exist: '{}'\nPlease provide a valid directory path.",
            target
        );
    }

    if !target_path.is_dir() {
        anyhow::bail!(
            "Target must be a directory, but '{}' is a file.\nPlease provide a directory to scan.",
            target
        );
    }

    // Create scanner configuration
    let config = ScanConfig::default();
    let scanner = Scanner::new(config);

    // Run scan
    let result = match scanner.scan_directory(&target_path).await {
        Ok(r) => r,
        Err(e) => {
            error!("Scan failed for '{}': {}", target, e);
            return Err(e).context(format!("Failed to scan directory '{}'", target));
        }
    };

    // Output results
    match output {
        OutputFormat::Terminal => {
            if let Err(e) = crate::output::terminal::render(&result) {
                error!("Failed to render terminal output: {}", e);
                return Err(e);
            }
        }
        OutputFormat::Json => {
            let json = match crate::output::json::generate(&result) {
                Ok(j) => j,
                Err(e) => {
                    error!("Failed to generate JSON report: {}", e);
                    return Err(e).context("Failed to generate JSON report");
                }
            };

            if let Some(file_path) = &output_file {
                if let Err(e) = std::fs::write(file_path, &json) {
                    error!("Failed to write report to '{}': {}", file_path, e);
                    return Err(e).context(format!("Failed to write report to '{}'", file_path));
                }
                info!("Report saved to: {}", file_path);
                println!("✅ Report saved to: {}", file_path);
            } else {
                println!("{}", json);
            }
        }
        _ => {
            error!("Output format {:?} not yet implemented", output);
            anyhow::bail!("Output format {:?} not yet implemented", output);
        }
    }

    // Check fail_on threshold
    if let Some(threshold) = fail_on {
        let threshold_severity = match threshold {
            SeverityLevel::Low => crate::models::vulnerability::Severity::Low,
            SeverityLevel::Medium => crate::models::vulnerability::Severity::Medium,
            SeverityLevel::High => crate::models::vulnerability::Severity::High,
            SeverityLevel::Critical => crate::models::vulnerability::Severity::Critical,
        };

        if result.has_issues_at_level(threshold_severity) {
            warn!(
                "Vulnerabilities found at or above {:?} threshold: {} critical, {} high",
                threshold, result.summary.critical, result.summary.high
            );
            anyhow::bail!("Found vulnerabilities at or above {:?} level", threshold);
        }
    }

    Ok(())
}
