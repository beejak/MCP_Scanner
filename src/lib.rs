//! MCP Sentinel - Security Scanner for Model Context Protocol Servers
//!
//! This library provides comprehensive security scanning for MCP servers through
//! three detection engines:
//! - **Static Analysis**: Code scanning with Semgrep integration and pattern matching
//! - **Runtime Proxy**: Real-time traffic monitoring and guardrails enforcement
//! - **AI Analysis**: LLM-powered vulnerability detection with risk explanations
//!
//! # Example
//!
//! ```no_run
//! use mcp_sentinel::engines::static_analysis::StaticAnalyzer;
//! use mcp_sentinel::models::config::Config;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = Config::default();
//!     let analyzer = StaticAnalyzer::new(config);
//!     let vulnerabilities = analyzer.scan_directory("./my-mcp-server").await?;
//!
//!     println!("Found {} vulnerabilities", vulnerabilities.len());
//!     Ok(())
//! }
//! ```

pub mod cli;
pub mod engines;
pub mod detectors;
pub mod models;
pub mod output;
pub mod storage;
pub mod utils;

// Re-export commonly used types
pub use models::vulnerability::{Vulnerability, Severity, VulnerabilityType};
pub use models::scan_result::ScanResult;
pub use models::config::Config;
