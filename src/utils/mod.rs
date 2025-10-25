//! Utility functions and helpers for MCP Sentinel.
//!
//! This module provides common utility functions used throughout the scanner:
//!
//! - **File operations**: Discovery, traversal, and classification (see [`file`])
//! - **Cryptography**: Hashing for content deduplication (see [`crypto`])
//! - **Git operations**: Repository detection and cloning *(Coming Soon)* (see [`git`])
//! - **HTTP utilities**: URL parsing and GitHub integration (see [`http`])
//!
//! # Examples
//!
//! ## File Discovery
//!
//! ```no_run
//! use mcp_sentinel::utils::FileScanner;
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let scanner = FileScanner::new()
//!     .max_file_size(10_485_760)  // 10 MB
//!     .respect_gitignore(true);
//!
//! let files = scanner.discover_files(Path::new("./mcp-server"))?;
//! println!("Found {} files to scan", files.len());
//! # Ok(())
//! # }
//! ```
//!
//! ## Content Hashing
//!
//! ```
//! use mcp_sentinel::utils::hash_content;
//!
//! let content = "suspicious code";
//! let hash = hash_content(content);
//! println!("SHA-256: {}", hash);
//! ```

pub mod file;
pub mod crypto;
pub mod git;
pub mod http;

pub use file::{FileScanner, DiscoveredFile};
pub use crypto::hash_content;