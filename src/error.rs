//! Error types for MCP Sentinel.
//!
//! This module provides structured error types for different failure modes
//! in the scanner. Using specific error types allows library users to handle
//! errors programmatically and provides better error messages to end users.

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias using `ScanError` as the error type.
pub type Result<T> = std::result::Result<T, ScanError>;

/// Main error type for scanning operations.
///
/// This enum represents all possible errors that can occur during scanning.
/// Each variant provides specific context about what went wrong.
///
/// # Examples
///
/// ```
/// use mcp_sentinel::error::ScanError;
/// use std::path::PathBuf;
///
/// let error = ScanError::TargetNotFound {
///     path: PathBuf::from("/nonexistent"),
/// };
/// assert!(error.to_string().contains("not found"));
/// ```
#[derive(Debug, Error)]
pub enum ScanError {
    /// Target file or directory does not exist.
    #[error("Target not found: {path}")]
    TargetNotFound {
        /// Path that was not found
        path: PathBuf,
    },

    /// Failed to read a file.
    #[error("Failed to read {path}: {source}")]
    FileReadError {
        /// Path to the file that couldn't be read
        path: PathBuf,
        /// Underlying I/O error
        #[source]
        source: std::io::Error,
    },

    /// Failed to write to a file.
    #[error("Failed to write to {path}: {source}")]
    FileWriteError {
        /// Path where write failed
        path: PathBuf,
        /// Underlying I/O error
        #[source]
        source: std::io::Error,
    },

    /// Invalid UTF-8 encoding in file.
    #[error("Invalid UTF-8 in {path} at byte offset {byte_offset}")]
    InvalidEncoding {
        /// Path to file with encoding issue
        path: PathBuf,
        /// Byte offset where invalid UTF-8 was found
        byte_offset: usize,
    },

    /// Invalid configuration.
    #[error("Invalid configuration: {message}")]
    ConfigError {
        /// Description of the configuration problem
        message: String,
    },

    /// Configuration file format error.
    #[error("Failed to parse config file {path}: {source}")]
    ConfigParseError {
        /// Path to config file
        path: PathBuf,
        /// Underlying parse error
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// A detector failed during scanning.
    #[error("Detector '{detector}' failed: {reason}")]
    DetectorError {
        /// Name of the detector that failed
        detector: String,
        /// Reason for failure
        reason: String,
    },

    /// Invalid JSON or MCP protocol format.
    #[error("Invalid MCP format in {location}: {reason}")]
    InvalidMcpFormat {
        /// Location (file path or description)
        location: String,
        /// Why the format is invalid
        reason: String,
    },

    /// Regex pattern compilation failed.
    #[error("Invalid regex pattern '{pattern}': {source}")]
    RegexError {
        /// The pattern that failed to compile
        pattern: String,
        /// Underlying regex error
        #[source]
        source: regex::Error,
    },

    /// Output formatting failed.
    #[error("Failed to format output as {format}: {reason}")]
    OutputError {
        /// The output format that failed
        format: String,
        /// Reason for failure
        reason: String,
    },

    /// File is too large to scan.
    #[error("File {path} is too large ({size} bytes, max {max_size} bytes)")]
    FileTooLarge {
        /// Path to the large file
        path: PathBuf,
        /// Actual file size
        size: u64,
        /// Maximum allowed size
        max_size: u64,
    },

    /// Generic I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// YAML serialization/deserialization error.
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// TOML serialization/deserialization error.
    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    /// TOML deserialization error.
    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),
}

impl ScanError {
    /// Create a `TargetNotFound` error.
    pub fn target_not_found(path: impl Into<PathBuf>) -> Self {
        ScanError::TargetNotFound { path: path.into() }
    }

    /// Create a `FileReadError` with context.
    pub fn file_read_error(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        ScanError::FileReadError {
            path: path.into(),
            source,
        }
    }

    /// Create a `FileWriteError` with context.
    pub fn file_write_error(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        ScanError::FileWriteError {
            path: path.into(),
            source,
        }
    }

    /// Create a `ConfigError`.
    pub fn config_error(message: impl Into<String>) -> Self {
        ScanError::ConfigError {
            message: message.into(),
        }
    }

    /// Create a `DetectorError`.
    pub fn detector_error(detector: impl Into<String>, reason: impl Into<String>) -> Self {
        ScanError::DetectorError {
            detector: detector.into(),
            reason: reason.into(),
        }
    }

    /// Create an `InvalidMcpFormat` error.
    pub fn invalid_mcp_format(location: impl Into<String>, reason: impl Into<String>) -> Self {
        ScanError::InvalidMcpFormat {
            location: location.into(),
            reason: reason.into(),
        }
    }

    /// Create an `OutputError`.
    pub fn output_error(format: impl Into<String>, reason: impl Into<String>) -> Self {
        ScanError::OutputError {
            format: format.into(),
            reason: reason.into(),
        }
    }

    /// Create a `FileTooLarge` error.
    pub fn file_too_large(path: impl Into<PathBuf>, size: u64, max_size: u64) -> Self {
        ScanError::FileTooLarge {
            path: path.into(),
            size,
            max_size,
        }
    }

    /// Check if this error is recoverable.
    ///
    /// Recoverable errors are those where scanning can continue with other files.
    /// Non-recoverable errors indicate fundamental problems that should stop scanning.
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            ScanError::FileReadError { .. }
                | ScanError::InvalidEncoding { .. }
                | ScanError::DetectorError { .. }
                | ScanError::InvalidMcpFormat { .. }
                | ScanError::FileTooLarge { .. }
        )
    }

    /// Get a user-friendly hint for fixing this error.
    pub fn hint(&self) -> Option<&'static str> {
        match self {
            ScanError::TargetNotFound { .. } => {
                Some("Check that the path is correct and the file/directory exists")
            }
            ScanError::InvalidEncoding { .. } => {
                Some("Ensure the file is saved with UTF-8 encoding")
            }
            ScanError::ConfigError { .. } => {
                Some("Run 'mcp-sentinel init' to create a default configuration")
            }
            ScanError::FileTooLarge { .. } => {
                Some("Increase max_file_size in configuration or exclude this file")
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_not_found_error() {
        let error = ScanError::target_not_found("/nonexistent");
        assert!(error.to_string().contains("not found"));
        assert!(error.to_string().contains("/nonexistent"));
    }

    #[test]
    fn test_config_error() {
        let error = ScanError::config_error("Missing API key");
        assert!(error.to_string().contains("Invalid configuration"));
        assert!(error.to_string().contains("Missing API key"));
    }

    #[test]
    fn test_detector_error() {
        let error = ScanError::detector_error("ToolPoisoning", "Regex failed");
        assert!(error.to_string().contains("ToolPoisoning"));
        assert!(error.to_string().contains("Regex failed"));
    }

    #[test]
    fn test_is_recoverable() {
        let recoverable = ScanError::detector_error("test", "test");
        assert!(recoverable.is_recoverable());

        let non_recoverable = ScanError::target_not_found("/test");
        assert!(!non_recoverable.is_recoverable());
    }

    #[test]
    fn test_hint() {
        let error = ScanError::target_not_found("/test");
        assert!(error.hint().is_some());
        assert!(error.hint().unwrap().contains("path is correct"));
    }
}
