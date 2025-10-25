//! Cryptographic utilities for content hashing.
//!
//! Provides SHA-256 hashing functions for content deduplication and integrity verification.
//!
//! # Use Cases
//!
//! - **Content deduplication**: Identify identical vulnerabilities across scans
//! - **Integrity verification**: Verify file contents haven't changed
//! - **Evidence tracking**: Generate unique identifiers for detected patterns
//! - **Cache keys**: Create deterministic keys for caching scan results
//!
//! # Examples
//!
//! ## Basic Content Hashing
//!
//! ```
//! use mcp_sentinel::utils::hash_content;
//!
//! let code = "console.log('test')";
//! let hash = hash_content(code);
//! println!("SHA-256: {}", hash);
//! ```
//!
//! ## Verifying Content Integrity
//!
//! ```
//! use mcp_sentinel::utils::hash_content;
//!
//! let original = "original content";
//! let hash1 = hash_content(original);
//!
//! // Later, verify content hasn't changed
//! let current = "original content";
//! let hash2 = hash_content(current);
//!
//! assert_eq!(hash1, hash2, "Content has been modified!");
//! ```

use sha2::{Sha256, Digest};

/// Compute SHA-256 hash of string content.
///
/// Returns the hash as a lowercase hexadecimal string (64 characters).
/// The hash is deterministic - identical content always produces the same hash.
///
/// # Arguments
///
/// * `content` - String content to hash
///
/// # Returns
///
/// A 64-character lowercase hexadecimal string representing the SHA-256 hash.
///
/// # Examples
///
/// ```
/// use mcp_sentinel::utils::crypto::hash_content;
///
/// let content = "hello world";
/// let hash = hash_content(content);
///
/// assert_eq!(
///     hash,
///     "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
/// );
/// ```
///
/// # Use Cases
///
/// ```
/// use mcp_sentinel::utils::crypto::hash_content;
///
/// // Detect duplicate vulnerabilities
/// let pattern1 = "eval(user_input)";
/// let pattern2 = "eval(user_input)";
/// assert_eq!(hash_content(pattern1), hash_content(pattern2));
///
/// // Create cache keys
/// let file_content = "function foo() { ... }";
/// let cache_key = format!("scan:{}", hash_content(file_content));
/// ```
pub fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Compute SHA-256 hash of raw bytes.
///
/// Like [`hash_content`], but operates on raw byte slices instead of strings.
/// Useful for hashing binary data or when you already have bytes.
///
/// # Arguments
///
/// * `bytes` - Byte slice to hash
///
/// # Returns
///
/// A 64-character lowercase hexadecimal string representing the SHA-256 hash.
///
/// # Examples
///
/// ```
/// use mcp_sentinel::utils::crypto::hash_bytes;
///
/// let data = b"hello world";
/// let hash = hash_bytes(data);
///
/// assert_eq!(
///     hash,
///     "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
/// );
/// ```
///
/// # Use Cases
///
/// ```no_run
/// use mcp_sentinel::utils::crypto::hash_bytes;
/// use std::fs;
///
/// // Hash file contents
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let file_bytes = fs::read("malicious.bin")?;
/// let file_hash = hash_bytes(&file_bytes);
/// println!("File hash: {}", file_hash);
/// # Ok(())
/// # }
/// ```
pub fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_content() {
        let content = "hello world";
        let hash = hash_content(content);

        // SHA-256 of "hello world"
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_hash_deterministic() {
        let content = "test content";
        let hash1 = hash_content(content);
        let hash2 = hash_content(content);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_different_content() {
        let hash1 = hash_content("content1");
        let hash2 = hash_content("content2");

        assert_ne!(hash1, hash2);
    }
}
