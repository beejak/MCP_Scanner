//! Git repository utilities for MCP scanner.
//!
//! Provides functions for detecting and working with Git repositories.
//! This is useful for scanning remote MCP servers by cloning their repositories.
//!
//! # Current Status
//!
//! - ✅ **Repository detection**: Check if a directory is a git repo
//! - 🚧 **Remote URL extraction**: Get git remote URL *(Coming Soon)*
//! - 🚧 **Repository cloning**: Clone repos to temporary locations *(Coming Soon)*
//!
//! # Planned Features (Phase 2)
//!
//! - Clone GitHub repositories for scanning
//! - Extract remote URLs from .git/config
//! - Detect repository metadata (commits, authors, etc.)
//! - Support for private repositories with credentials
//!
//! # Examples
//!
//! ## Check if Directory is a Git Repo
//!
//! ```no_run
//! use mcp_sentinel::utils::git::is_git_repo;
//! use std::path::Path;
//!
//! if is_git_repo(Path::new("./my-mcp-server")) {
//!     println!("This is a git repository!");
//! } else {
//!     println!("Not a git repository");
//! }
//! ```
//!
//! ## Planned: Clone and Scan Remote Repo
//!
//! ```no_run
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Coming in Phase 2
//! // let temp_dir = clone_repo("https://github.com/user/mcp-server").await?;
//! // let scanner = FileScanner::new();
//! // let files = scanner.discover_files(temp_dir.path())?;
//! # Ok(())
//! # }
//! ```

use anyhow::Result;
use std::path::Path;

/// Check if a directory is a git repository.
///
/// Simply checks for the existence of a `.git` directory. This is a fast
/// heuristic that works for most cases but may not detect all git repositories
/// (e.g., bare repos, worktrees).
///
/// # Arguments
///
/// * `path` - Directory path to check
///
/// # Returns
///
/// `true` if the directory contains a `.git` subdirectory, `false` otherwise.
///
/// # Examples
///
/// ```no_run
/// use mcp_sentinel::utils::git::is_git_repo;
/// use std::path::Path;
///
/// let is_repo = is_git_repo(Path::new("."));
/// if is_repo {
///     println!("Current directory is a git repository");
/// }
/// ```
///
/// # Use Cases
///
/// ```no_run
/// use mcp_sentinel::utils::git::is_git_repo;
/// use std::path::Path;
///
/// // Skip .git directory during scanning
/// let path = Path::new("./some-dir");
/// if is_git_repo(path) {
///     println!("Skipping .git metadata...");
/// }
/// ```
pub fn is_git_repo(path: &Path) -> bool {
    path.join(".git").exists()
}

/// Get git remote URL if available.
///
/// **Status**: Not yet implemented (returns None).
///
/// # Planned Implementation
///
/// Will read `.git/config` or use `git2` crate to extract the remote URL
/// of the repository. Useful for:
/// - Identifying the source of an MCP server
/// - Linking scan results to source repositories
/// - Verifying repository ownership
///
/// # Arguments
///
/// * `path` - Path to git repository
///
/// # Returns
///
/// - `Ok(Some(url))` - Remote URL found
/// - `Ok(None)` - Not a git repo or no remote configured
/// - `Err(_)` - Error reading git config
///
/// # Examples
///
/// ```no_run
/// use mcp_sentinel::utils::git::get_remote_url;
/// use std::path::Path;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// if let Some(url) = get_remote_url(Path::new("./mcp-server"))? {
///     println!("Remote: {}", url);
/// }
/// # Ok(())
/// # }
/// ```
pub fn get_remote_url(path: &Path) -> Result<Option<String>> {
    if !is_git_repo(path) {
        return Ok(None);
    }

    // TODO: Implement git remote URL detection
    // This would use git2 crate or shell out to git command
    Ok(None)
}

/// Clone a git repository to a temporary location.
///
/// **Status**: Not yet implemented.
///
/// # Planned Implementation
///
/// Will clone a git repository using either:
/// - `git2` crate for pure Rust implementation
/// - Shell out to `git clone` command
///
/// The repository will be cloned to a temporary directory that is
/// automatically cleaned up when the `TempDir` is dropped.
///
/// # Arguments
///
/// * `url` - Git repository URL (https or ssh)
///
/// # Returns
///
/// A temporary directory containing the cloned repository.
/// The directory is automatically deleted when dropped.
///
/// # Errors
///
/// Returns an error if:
/// - URL is invalid
/// - Network error during cloning
/// - Authentication required but not provided
/// - Insufficient disk space
///
/// # Examples
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// use mcp_sentinel::utils::git::clone_repo;
///
/// // Clone public repo
/// let temp_dir = clone_repo("https://github.com/user/mcp-server").await?;
/// println!("Cloned to: {:?}", temp_dir.path());
///
/// // Temp directory is automatically cleaned up when temp_dir is dropped
/// # Ok(())
/// # }
/// ```
///
/// # Planned Features
///
/// - Support for private repositories with credentials
/// - Shallow clones for faster performance
/// - Clone specific branches or tags
/// - Progress reporting for large repositories
pub async fn clone_repo(_url: &str) -> Result<tempfile::TempDir> {
    // TODO: Implement git cloning
    // This would use git2 crate or shell out to git command
    anyhow::bail!("Git cloning not yet implemented")
}
