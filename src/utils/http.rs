//! HTTP and URL utilities for MCP scanner.
//!
//! Provides URL parsing and validation, with special support for GitHub URLs.
//! Used for scanning remote MCP servers by URL and integrating with GitHub.
//!
//! # Features
//!
//! - **URL validation**: Parse and validate HTTP/HTTPS URLs
//! - **GitHub detection**: Identify and parse GitHub repository URLs
//! - **Repository extraction**: Extract owner and repo name from URLs
//!
//! # Use Cases
//!
//! - Scanning MCP servers hosted on GitHub
//! - Validating user-provided URLs before fetching
//! - Extracting repository metadata from URLs
//! - CLI commands like `mcp-sentinel scan https://github.com/user/repo`
//!
//! # Examples
//!
//! ## Parse and Validate URL
//!
//! ```
//! use mcp_sentinel::utils::http::parse_url;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let url = parse_url("https://github.com/user/repo")?;
//! println!("Scheme: {}", url.scheme());
//! println!("Host: {}", url.host_str().unwrap());
//! # Ok(())
//! # }
//! ```
//!
//! ## Check if URL is GitHub
//!
//! ```
//! use mcp_sentinel::utils::http::is_github_url;
//!
//! assert!(is_github_url("https://github.com/user/repo"));
//! assert!(!is_github_url("https://gitlab.com/user/repo"));
//! ```
//!
//! ## Extract GitHub Repository Info
//!
//! ```
//! use mcp_sentinel::utils::http::parse_github_url;
//!
//! let (owner, repo) = parse_github_url("https://github.com/cisco-ai-defense/mcp-scanner")
//!     .expect("Valid GitHub URL");
//!
//! assert_eq!(owner, "cisco-ai-defense");
//! assert_eq!(repo, "mcp-scanner");
//! ```

use anyhow::Result;
use url::Url;

/// Parse and validate a URL.
///
/// Parses a URL string and validates that it uses HTTP or HTTPS scheme.
/// Other schemes (ftp, file, etc.) are rejected.
///
/// # Arguments
///
/// * `url_str` - URL string to parse
///
/// # Returns
///
/// A parsed [`Url`] object if valid.
///
/// # Errors
///
/// Returns an error if:
/// - URL syntax is invalid
/// - Scheme is not http or https
///
/// # Examples
///
/// ```
/// use mcp_sentinel::utils::http::parse_url;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Valid URLs
/// assert!(parse_url("https://example.com").is_ok());
/// assert!(parse_url("http://localhost:8080").is_ok());
///
/// // Invalid URLs
/// assert!(parse_url("ftp://example.com").is_err());
/// assert!(parse_url("not a url").is_err());
/// # Ok(())
/// # }
/// ```
pub fn parse_url(url_str: &str) -> Result<Url> {
    let url = Url::parse(url_str)?;

    // Validate scheme
    match url.scheme() {
        "http" | "https" => Ok(url),
        scheme => anyhow::bail!("Unsupported URL scheme: {}", scheme),
    }
}

/// Check if a string looks like a GitHub URL.
///
/// Simple check that looks for "github.com" in the URL string.
/// This is a fast heuristic but not a complete validation.
///
/// For full parsing and validation, use [`parse_github_url`].
///
/// # Arguments
///
/// * `url_str` - URL string to check
///
/// # Returns
///
/// `true` if the URL contains "github.com", `false` otherwise.
///
/// # Examples
///
/// ```
/// use mcp_sentinel::utils::http::is_github_url;
///
/// assert!(is_github_url("https://github.com/user/repo"));
/// assert!(is_github_url("http://github.com/user/repo"));
/// assert!(is_github_url("https://api.github.com/repos/user/repo"));
///
/// assert!(!is_github_url("https://gitlab.com/user/repo"));
/// assert!(!is_github_url("https://example.com"));
/// ```
pub fn is_github_url(url_str: &str) -> bool {
    url_str.contains("github.com")
}

/// Extract GitHub repository information from a URL.
///
/// Parses a GitHub URL and extracts the repository owner and name.
/// Handles both regular GitHub URLs and URLs ending in .git.
///
/// # Arguments
///
/// * `url_str` - GitHub URL to parse
///
/// # Returns
///
/// - `Some((owner, repo))` - Successfully parsed owner and repository name
/// - `None` - Not a valid GitHub URL or couldn't extract info
///
/// # Supported URL Formats
///
/// - `https://github.com/owner/repo`
/// - `https://github.com/owner/repo.git`
/// - `http://github.com/owner/repo`
///
/// # Examples
///
/// ```
/// use mcp_sentinel::utils::http::parse_github_url;
///
/// // Standard URL
/// let result = parse_github_url("https://github.com/cisco-ai-defense/mcp-scanner");
/// assert_eq!(result, Some(("cisco-ai-defense".to_string(), "mcp-scanner".to_string())));
///
/// // URL with .git extension
/// let result = parse_github_url("https://github.com/user/repo.git");
/// assert_eq!(result, Some(("user".to_string(), "repo".to_string())));
///
/// // Non-GitHub URL
/// let result = parse_github_url("https://gitlab.com/user/repo");
/// assert_eq!(result, None);
///
/// // Invalid URL
/// let result = parse_github_url("not a url");
/// assert_eq!(result, None);
/// ```
///
/// # Use Cases
///
/// ```no_run
/// use mcp_sentinel::utils::http::parse_github_url;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let url = "https://github.com/user/mcp-server";
///
/// if let Some((owner, repo)) = parse_github_url(url) {
///     println!("Scanning {}/{}", owner, repo);
///     // Clone and scan the repository
/// } else {
///     println!("Not a valid GitHub URL");
/// }
/// # Ok(())
/// # }
/// ```
pub fn parse_github_url(url_str: &str) -> Option<(String, String)> {
    let url = Url::parse(url_str).ok()?;

    if url.host_str()? != "github.com" {
        return None;
    }

    let path_segments: Vec<&str> = url.path_segments()?.collect();

    if path_segments.len() >= 2 {
        let owner = path_segments[0].to_string();
        let repo = path_segments[1].trim_end_matches(".git").to_string();
        Some((owner, repo))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url() {
        assert!(parse_url("https://example.com").is_ok());
        assert!(parse_url("http://example.com").is_ok());
        assert!(parse_url("ftp://example.com").is_err());
        assert!(parse_url("not a url").is_err());
    }

    #[test]
    fn test_is_github_url() {
        assert!(is_github_url("https://github.com/user/repo"));
        assert!(!is_github_url("https://gitlab.com/user/repo"));
    }

    #[test]
    fn test_parse_github_url() {
        let result = parse_github_url("https://github.com/cisco-ai-defense/mcp-scanner");
        assert_eq!(result, Some(("cisco-ai-defense".to_string(), "mcp-scanner".to_string())));

        let result2 = parse_github_url("https://github.com/user/repo.git");
        assert_eq!(result2, Some(("user".to_string(), "repo".to_string())));

        let result3 = parse_github_url("https://gitlab.com/user/repo");
        assert_eq!(result3, None);
    }
}
