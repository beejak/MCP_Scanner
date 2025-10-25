//! MCP Sentinel - Security Scanner for Model Context Protocol Servers
//!
//! This library provides comprehensive security scanning for MCP servers through
//! three detection engines:
//! - **Static Analysis**: Code scanning with pattern matching and regex detection
//! - **Runtime Proxy** *(Coming Soon)*: Real-time traffic monitoring and guardrails enforcement
//! - **AI Analysis** *(Coming Soon)*: LLM-powered vulnerability detection with risk explanations
//!
//! # Quick Start
//!
//! Scan a directory for vulnerabilities:
//!
//! ```no_run
//! use mcp_sentinel::detectors::{ToolPoisoningDetector, Detector};
//! use mcp_sentinel::error::Result;
//!
//! fn main() -> Result<()> {
//!     let detector = ToolPoisoningDetector::new();
//!     let content = std::fs::read_to_string("tools.json")?;
//!     let vulnerabilities = detector.scan(&content, Some("tools.json"))?;
//!     
//!     for vuln in vulnerabilities {
//!         println!("[{}] {}: {}", vuln.severity, vuln.id, vuln.title);
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! # Features
//!
//! ## Detectors
//!
//! Multiple specialized detectors for different vulnerability types:
//!
//! - [`ToolPoisoningDetector`](detectors::ToolPoisoningDetector): Hidden instructions in MCP tool descriptions
//! - [`PromptInjectionDetector`](detectors::PromptInjectionDetector): Prompt injection attempts
//! - [`SecretsDetector`](detectors::SecretsDetector): Leaked credentials and API keys
//! - [`CodeVulnsDetector`](detectors::CodeVulnsDetector): Code-level vulnerabilities
//!
//! ## Output Formats
//!
//! Results can be formatted in multiple ways:
//!
//! - Terminal (colored, human-readable)
//! - JSON (machine-readable)
//! - HTML *(Coming Soon)*
//! - PDF *(Coming Soon)*
//!
//! # Error Handling
//!
//! All fallible operations return [`error::Result<T>`](error::Result) which uses
//! [`ScanError`](error::ScanError) for structured error handling:
//!
//! ```no_run
//! use mcp_sentinel::error::{Result, ScanError};
//!
//! fn scan_file(path: &str) -> Result<()> {
//!     let content = std::fs::read_to_string(path)
//!         .map_err(|e| ScanError::file_read_error(path, e))?;
//!     // ... scan logic
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod cli;
pub mod engines;
pub mod detectors;
pub mod models;
pub mod output;
pub mod storage;
pub mod utils;

// Re-export commonly used types
pub use error::{Result, ScanError};
pub use models::vulnerability::{Vulnerability, Severity, VulnerabilityType};
pub use models::scan_result::ScanResult;
pub use models::config::Config;
