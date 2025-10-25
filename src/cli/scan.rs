use crate::detectors::{Detector, ToolPoisoningDetector, PromptInjectionDetector, SecretsDetector};
use crate::models::config::Config;
use crate::models::scan_result::{ScanResult, Metadata};
use crate::models::vulnerability::{Vulnerability, Severity};
use crate::output::{OutputFormat, OutputFormatter, terminal::TerminalFormatter, json::JsonFormatter};
use crate::utils::file::{FileScanner, read_file_contents};
use crate::error::{Result, ScanError};
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::{info, debug};

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum ScanMode {
    /// Quick scan (static analysis only)
    Quick,
    /// Deep scan (static analysis + AI)
    Deep,
}

pub async fn run(
    target: String,
    mode: ScanMode,
    severity: String,
    fail_on: Option<String>,
    output_format: String,
    output_file: Option<String>,
    _llm_provider: Option<String>,
    _llm_model: Option<String>,
    _llm_api_key: Option<String>,
    _config_file: Option<String>,
    config: Config,
) -> Result<i32> {
    info!("Starting MCP security scan");
    debug!("Target: {}", target);
    debug!("Mode: {:?}", mode);

    let start_time = Instant::now();

    // Determine scan engines based on mode
    let engines = match mode {
        ScanMode::Quick => vec!["static".to_string()],
        ScanMode::Deep => vec!["static".to_string(), "ai".to_string()],
    };

    // Scan the target
    let vulnerabilities = scan_target(&target, &config).await?;

    let duration_ms = start_time.elapsed().as_millis() as u64;

    // Create scan result
    let mut result = ScanResult::new(
        target.clone(),
        engines.clone(),
        vulnerabilities,
        Metadata {
            scan_duration_ms: duration_ms,
            engines_used: engines,
            llm_provider: None, // TODO: Use LLM provider when deep mode is implemented
            llm_model: None,
        },
    );

    // Filter by minimum severity if specified
    if let Some(min_sev) = Severity::from_str(&severity) {
        result = result.filter_by_severity(min_sev);
    }

    // Output results
    output_results(&result, &output_format, output_file.as_deref())?;

    // Determine exit code
    let exit_code = if let Some(fail_level_str) = fail_on {
        if let Some(fail_level) = Severity::from_str(&fail_level_str) {
            if result.summary.has_issues_at_or_above(fail_level) {
                1 // Fail if vulnerabilities at or above threshold
            } else {
                0
            }
        } else {
            0
        }
    } else {
        0
    };

    Ok(exit_code)
}

/// Scan a target (directory or file)
async fn scan_target(target: &str, config: &Config) -> Result<Vec<Vulnerability>> {
    let target_path = Path::new(target);
    let mut all_vulnerabilities = Vec::new();

    // Verify target exists
    if !target_path.exists() {
        return Err(ScanError::TargetNotFound {
            path: target_path.to_path_buf(),
        });
    }

    // Initialize detectors
    let detectors: Vec<Box<dyn Detector>> = vec![
        Box::new(ToolPoisoningDetector::new()),
        Box::new(PromptInjectionDetector::new()),
        Box::new(SecretsDetector::new()),
    ];

    if target_path.is_file() {
        // Scan single file
        info!("Scanning file: {}", target);
        let content = read_file_contents(target_path)
            .map_err(|e| ScanError::FileReadError {
                path: target_path.to_path_buf(),
                source: e,
            })?;
        all_vulnerabilities.extend(scan_content(&content, Some(target), &detectors)?);
    } else if target_path.is_dir() {
        // Scan directory
        info!("Scanning directory: {}", target);

        let scanner = FileScanner::new()
            .max_file_size(config.scanning.max_file_size)
            .respect_gitignore(config.scanning.respect_gitignore)
            .follow_symlinks(config.scanning.follow_symlinks);

        let files = scanner.discover_files(target_path)
            .map_err(|e| ScanError::DirectoryTraversalError {
                path: target_path.to_path_buf(),
                source: e,
            })?;

        info!("Found {} files to scan", files.len());

        for file in files {
            debug!("Scanning: {}", file.path.display());

            match read_file_contents(&file.path) {
                Ok(content) => {
                    let file_path_str = file.path.to_string_lossy().to_string();
                    let vulns = scan_content(&content, Some(&file_path_str), &detectors)?;
                    all_vulnerabilities.extend(vulns);
                }
                Err(e) => {
                    // Log error but continue scanning other files
                    debug!("Failed to read {}: {}", file.path.display(), e);
                }
            }
        }
    } else {
        return Err(ScanError::InvalidTarget {
            path: target_path.to_path_buf(),
            reason: "Target is neither a file nor a directory".to_string(),
        });
    }

    info!("Scan complete. Found {} vulnerabilities", all_vulnerabilities.len());
    Ok(all_vulnerabilities)
}

/// Scan content with all detectors
fn scan_content(
    content: &str,
    file_path: Option<&str>,
    detectors: &[Box<dyn Detector>],
) -> Result<Vec<Vulnerability>> {
    let mut vulnerabilities = Vec::new();

    for detector in detectors {
        match detector.scan(content, file_path) {
            Ok(vulns) => vulnerabilities.extend(vulns),
            Err(e) => {
                // Log detector failures but don't stop scanning
                debug!("Detector {} failed: {}", detector.name(), e);
            }
        }
    }

    Ok(vulnerabilities)
}

/// Output scan results in requested format
fn output_results(
    result: &ScanResult,
    format: &str,
    output_file: Option<&str>,
) -> Result<()> {
    let output_format = OutputFormat::from_str(format)
        .unwrap_or(OutputFormat::Terminal);

    match output_format {
        OutputFormat::Terminal => {
            let formatter = TerminalFormatter::new();
            let output = formatter.output(result)?;
            println!("{}", output);

            if let Some(file_path) = output_file {
                std::fs::write(file_path, output)
                    .map_err(|e| ScanError::FileWriteError {
                        path: PathBuf::from(file_path),
                        source: e,
                    })?;
                println!("Report saved to: {}", file_path);
            }
        }
        OutputFormat::Json => {
            let formatter = JsonFormatter::new();
            let output = formatter.output(result)?;

            if let Some(file_path) = output_file {
                std::fs::write(file_path, &output)
                    .map_err(|e| ScanError::FileWriteError {
                        path: PathBuf::from(file_path),
                        source: e,
                    })?;
                println!("Report saved to: {}", file_path);
            } else {
                println!("{}", output);
            }
        }
        _ => {
            return Err(ScanError::UnsupportedFormat {
                format: format.to_string(),
                supported: vec!["terminal".to_string(), "json".to_string()],
            });
        }
    }

    Ok(())
}
