//! File discovery and classification utilities.
//!
//! This module provides tools for discovering, filtering, and classifying files
//! for security scanning. It handles .gitignore rules, binary detection, file
//! size limits, and pattern matching.
//!
//! # Features
//!
//! - **Smart discovery**: Respects .gitignore, skips binary files
//! - **Size limits**: Configurable max file size to avoid scanning huge files
//! - **Pattern matching**: Filter files by glob patterns or extensions
//! - **Classification**: Identify source files vs config files
//! - **Symlink handling**: Configurable symlink following
//!
//! # Examples
//!
//! ## Basic File Discovery
//!
//! ```no_run
//! use mcp_sentinel::utils::FileScanner;
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let scanner = FileScanner::new();
//! let files = scanner.discover_files(Path::new("./mcp-server"))?;
//!
//! for file in files {
//!     println!("{}: {} bytes", file.path.display(), file.size);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Custom Configuration
//!
//! ```no_run
//! use mcp_sentinel::utils::FileScanner;
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let scanner = FileScanner::new()
//!     .max_file_size(5_242_880)      // 5 MB limit
//!     .respect_gitignore(true)        // Honor .gitignore
//!     .follow_symlinks(false)         // Don't follow symlinks
//!     .max_depth(Some(10));          // Limit directory depth
//!
//! let files = scanner.discover_files(Path::new("."))?;
//! println!("Found {} files", files.len());
//! # Ok(())
//! # }
//! ```
//!
//! ## Pattern Filtering
//!
//! ```no_run
//! use mcp_sentinel::utils::FileScanner;
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let scanner = FileScanner::new();
//! let patterns = vec!["*.py".to_string(), "*.js".to_string()];
//!
//! let files = scanner.discover_files_with_patterns(
//!     Path::new("./src"),
//!     &patterns
//! )?;
//!
//! println!("Found {} Python/JavaScript files", files.len());
//! # Ok(())
//! # }
//! ```

use anyhow::Result;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// Represents a discovered file for scanning.
///
/// Contains metadata about a file that has been discovered during
/// filesystem traversal and is ready for security scanning.
///
/// # Fields
///
/// - `path`: Full path to the file
/// - `size`: File size in bytes
/// - `is_binary`: Whether the file appears to be binary (not text)
#[derive(Debug, Clone)]
pub struct DiscoveredFile {
    /// Full path to the discovered file
    pub path: PathBuf,
    /// File size in bytes
    pub size: u64,
    /// Whether the file is detected as binary
    pub is_binary: bool,
}

/// File scanner for discovering files to scan.
///
/// Provides flexible file discovery with configurable filtering, size limits,
/// and .gitignore support. Uses the `ignore` crate for efficient traversal.
///
/// # Builder Pattern
///
/// The scanner uses a builder pattern for configuration:
///
/// ```
/// use mcp_sentinel::utils::FileScanner;
///
/// let scanner = FileScanner::new()
///     .max_file_size(10_485_760)
///     .respect_gitignore(true)
///     .follow_symlinks(false);
/// ```
///
/// # Default Configuration
///
/// - Max file size: 10 MB
/// - Respect .gitignore: Yes
/// - Follow symlinks: No
/// - Max depth: Unlimited
/// - Binary files: Skipped automatically
pub struct FileScanner {
    max_file_size: u64,
    respect_gitignore: bool,
    follow_symlinks: bool,
    max_depth: Option<usize>,
}

impl FileScanner {
    /// Create a new file scanner with default settings.
    ///
    /// Defaults:
    /// - Max file size: 10 MB (10,485,760 bytes)
    /// - Respect .gitignore: true
    /// - Follow symlinks: false
    /// - Max depth: None (unlimited)
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp_sentinel::utils::FileScanner;
    ///
    /// let scanner = FileScanner::new();
    /// ```
    pub fn new() -> Self {
        Self {
            max_file_size: 10_485_760, // 10 MB
            respect_gitignore: true,
            follow_symlinks: false,
            max_depth: None,
        }
    }

    /// Set maximum file size to scan (in bytes).
    ///
    /// Files larger than this size will be skipped during discovery.
    /// This helps avoid performance issues with very large files.
    ///
    /// # Arguments
    ///
    /// * `size` - Maximum file size in bytes
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp_sentinel::utils::FileScanner;
    ///
    /// // Limit to 5 MB
    /// let scanner = FileScanner::new().max_file_size(5_242_880);
    ///
    /// // No practical limit
    /// let large_scanner = FileScanner::new().max_file_size(u64::MAX);
    /// ```
    pub fn max_file_size(mut self, size: u64) -> Self {
        self.max_file_size = size;
        self
    }

    /// Set whether to respect .gitignore files.
    ///
    /// When enabled, files and directories listed in .gitignore will be
    /// excluded from discovery. This is usually what you want for scanning
    /// source repositories.
    ///
    /// # Arguments
    ///
    /// * `respect` - true to honor .gitignore, false to scan all files
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp_sentinel::utils::FileScanner;
    ///
    /// // Respect .gitignore (default)
    /// let scanner = FileScanner::new().respect_gitignore(true);
    ///
    /// // Scan everything, including ignored files
    /// let full_scanner = FileScanner::new().respect_gitignore(false);
    /// ```
    pub fn respect_gitignore(mut self, respect: bool) -> Self {
        self.respect_gitignore = respect;
        self
    }

    /// Set whether to follow symbolic links.
    ///
    /// When enabled, the scanner will follow symlinks and scan their targets.
    /// When disabled (default), symlinks are skipped to avoid loops and
    /// unexpected file system traversal.
    ///
    /// # Arguments
    ///
    /// * `follow` - true to follow symlinks, false to skip them
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp_sentinel::utils::FileScanner;
    ///
    /// // Follow symlinks (be careful with loops!)
    /// let scanner = FileScanner::new().follow_symlinks(true);
    ///
    /// // Skip symlinks (default, safer)
    /// let safe_scanner = FileScanner::new().follow_symlinks(false);
    /// ```
    pub fn follow_symlinks(mut self, follow: bool) -> Self {
        self.follow_symlinks = follow;
        self
    }

    /// Set maximum directory traversal depth.
    ///
    /// Limits how deep into the directory tree the scanner will go.
    /// None means unlimited depth (default).
    ///
    /// # Arguments
    ///
    /// * `depth` - Maximum depth (None for unlimited)
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp_sentinel::utils::FileScanner;
    ///
    /// // Scan only 3 levels deep
    /// let shallow = FileScanner::new().max_depth(Some(3));
    ///
    /// // Unlimited depth (default)
    /// let deep = FileScanner::new().max_depth(None);
    /// ```
    pub fn max_depth(mut self, depth: Option<usize>) -> Self {
        self.max_depth = depth;
        self
    }

    /// Discover files in a directory.
    ///
    /// Recursively walks the directory tree and returns all files that
    /// match the scanner's configuration (size limits, binary detection, etc.).
    ///
    /// # Arguments
    ///
    /// * `path` - Root directory to scan
    ///
    /// # Returns
    ///
    /// A vector of discovered files with metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path doesn't exist
    /// - Permission denied
    /// - I/O errors during traversal
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mcp_sentinel::utils::FileScanner;
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let scanner = FileScanner::new();
    /// let files = scanner.discover_files(Path::new("./src"))?;
    ///
    /// println!("Found {} files", files.len());
    /// for file in files {
    ///     println!("  {}", file.path.display());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn discover_files(&self, path: &Path) -> Result<Vec<DiscoveredFile>> {
        let mut files = Vec::new();

        let mut builder = WalkBuilder::new(path);
        builder
            .git_ignore(self.respect_gitignore)
            .follow_links(self.follow_symlinks);

        if let Some(depth) = self.max_depth {
            builder.max_depth(Some(depth));
        }

        for result in builder.build() {
            match result {
                Ok(entry) => {
                    let path = entry.path();

                    // Skip directories
                    if !path.is_file() {
                        continue;
                    }

                    // Get file size
                    let metadata = match std::fs::metadata(path) {
                        Ok(m) => m,
                        Err(e) => {
                            warn!("Failed to get metadata for {}: {}", path.display(), e);
                            continue;
                        }
                    };

                    let size = metadata.len();

                    // Skip files larger than max size
                    if size > self.max_file_size {
                        debug!("Skipping large file: {} ({} bytes)", path.display(), size);
                        continue;
                    }

                    // Check if file is binary
                    let is_binary = is_binary_file(path);

                    // Skip binary files
                    if is_binary {
                        debug!("Skipping binary file: {}", path.display());
                        continue;
                    }

                    files.push(DiscoveredFile {
                        path: path.to_path_buf(),
                        size,
                        is_binary,
                    });
                }
                Err(e) => {
                    warn!("Error walking directory: {}", e);
                }
            }
        }

        debug!("Discovered {} files for scanning", files.len());
        Ok(files)
    }

    /// Discover files matching specific patterns.
    ///
    /// Like `discover_files()`, but filters results to only files matching
    /// the provided glob patterns.
    ///
    /// # Arguments
    ///
    /// * `path` - Root directory to scan
    /// * `patterns` - Glob patterns to match (e.g., "*.py", "config.*")
    ///
    /// # Pattern Syntax
    ///
    /// - `*.py` - All Python files
    /// - `test_*.rs` - Rust test files
    /// - `*.json` - All JSON files
    /// - `config.*` - Files named "config" with any extension
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mcp_sentinel::utils::FileScanner;
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let scanner = FileScanner::new();
    /// let patterns = vec!["*.py".to_string(), "*.js".to_string()];
    ///
    /// let files = scanner.discover_files_with_patterns(
    ///     Path::new("./src"),
    ///     &patterns
    /// )?;
    ///
    /// println!("Found {} matching files", files.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn discover_files_with_patterns(
        &self,
        path: &Path,
        patterns: &[String],
    ) -> Result<Vec<DiscoveredFile>> {
        let all_files = self.discover_files(path)?;

        // Filter by patterns if provided
        if patterns.is_empty() {
            return Ok(all_files);
        }

        let filtered: Vec<_> = all_files
            .into_iter()
            .filter(|f| {
                let path_str = f.path.to_string_lossy();
                patterns.iter().any(|pattern| {
                    // Simple glob matching
                    if pattern.contains('*') {
                        let regex_pattern = pattern
                            .replace('.', r"\.")
                            .replace('*', ".*");
                        regex::Regex::new(&regex_pattern)
                            .map(|re| re.is_match(&path_str))
                            .unwrap_or(false)
                    } else {
                        path_str.ends_with(pattern)
                    }
                })
            })
            .collect();

        debug!("Filtered to {} files matching patterns", filtered.len());
        Ok(filtered)
    }
}

impl Default for FileScanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a file is binary by reading first few bytes.
///
/// Reads the first 8KB of the file and checks for null bytes.
/// If more than 1% of bytes are null, the file is considered binary.
///
/// # Algorithm
///
/// 1. Read first 8192 bytes
/// 2. Count null bytes (0x00)
/// 3. If >1% are null, classify as binary
///
/// This heuristic works well for most text vs binary detection.
fn is_binary_file(path: &Path) -> bool {
    use std::fs::File;
    use std::io::Read;

    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };

    let mut buffer = [0u8; 8192];
    let bytes_read = match file.read(&mut buffer) {
        Ok(n) => n,
        Err(_) => return false,
    };

    if bytes_read == 0 {
        return false;
    }

    // Check for null bytes (common in binary files)
    let null_count = buffer[..bytes_read].iter().filter(|&&b| b == 0).count();
    let null_percentage = (null_count as f64) / (bytes_read as f64);

    // If more than 1% null bytes, likely binary
    null_percentage > 0.01
}

/// Read file contents as a UTF-8 string.
///
/// # Arguments
///
/// * `path` - Path to the file to read
///
/// # Errors
///
/// Returns an error if:
/// - File doesn't exist
/// - Permission denied
/// - File contains invalid UTF-8
///
/// # Examples
///
/// ```no_run
/// use mcp_sentinel::utils::file::read_file_contents;
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let content = read_file_contents(Path::new("config.json"))?;
/// println!("File size: {} bytes", content.len());
/// # Ok(())
/// # }
/// ```
pub fn read_file_contents(path: &Path) -> Result<String> {
    Ok(std::fs::read_to_string(path)?)
}

/// Get file extension in lowercase.
///
/// Returns the file extension (without the dot) in lowercase,
/// or None if the file has no extension.
///
/// # Examples
///
/// ```
/// use mcp_sentinel::utils::file::get_file_extension;
/// use std::path::Path;
///
/// assert_eq!(get_file_extension(Path::new("file.py")), Some("py".to_string()));
/// assert_eq!(get_file_extension(Path::new("FILE.RS")), Some("rs".to_string()));
/// assert_eq!(get_file_extension(Path::new("noext")), None);
/// ```
pub fn get_file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
}

/// Check if file is likely source code.
///
/// Returns true if the file extension matches known source code extensions.
///
/// Recognized extensions: py, js, ts, tsx, jsx, rs, go, java, c, cpp, h, hpp,
/// cs, rb, php, swift, kt, scala, clj, sh, bash, zsh
///
/// # Examples
///
/// ```
/// use mcp_sentinel::utils::file::is_source_file;
/// use std::path::Path;
///
/// assert!(is_source_file(Path::new("main.py")));
/// assert!(is_source_file(Path::new("app.rs")));
/// assert!(!is_source_file(Path::new("README.md")));
/// assert!(!is_source_file(Path::new("data.json")));
/// ```
pub fn is_source_file(path: &Path) -> bool {
    const SOURCE_EXTENSIONS: &[&str] = &[
        "py", "js", "ts", "tsx", "jsx", "rs", "go", "java", "c", "cpp", "h", "hpp",
        "cs", "rb", "php", "swift", "kt", "scala", "clj", "sh", "bash", "zsh",
    ];

    get_file_extension(path)
        .map(|ext| SOURCE_EXTENSIONS.contains(&ext.as_str()))
        .unwrap_or(false)
}

/// Check if file is likely a configuration file.
///
/// Returns true if the file extension or name matches known config file patterns.
///
/// Recognized extensions: json, yaml, yml, toml, ini, conf, cfg
/// Recognized names: .env, dockerfile, makefile, rakefile
///
/// # Examples
///
/// ```
/// use mcp_sentinel::utils::file::is_config_file;
/// use std::path::Path;
///
/// assert!(is_config_file(Path::new("config.json")));
/// assert!(is_config_file(Path::new("settings.yaml")));
/// assert!(is_config_file(Path::new(".env")));
/// assert!(is_config_file(Path::new("Dockerfile")));
/// assert!(!is_config_file(Path::new("main.py")));
/// ```
pub fn is_config_file(path: &Path) -> bool {
    const CONFIG_EXTENSIONS: &[&str] = &["json", "yaml", "yml", "toml", "ini", "conf", "cfg"];
    const CONFIG_NAMES: &[&str] = &[".env", "dockerfile", "makefile", "rakefile"];

    let file_name = path.file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase());

    if let Some(name) = file_name {
        if CONFIG_NAMES.iter().any(|&cn| name.contains(cn)) {
            return true;
        }
    }

    get_file_extension(path)
        .map(|ext| CONFIG_EXTENSIONS.contains(&ext.as_str()))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_file_scanner_discover() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create test files
        let test_file = temp_path.join("test.py");
        let mut file = std::fs::File::create(&test_file).unwrap();
        writeln!(file, "print('hello')").unwrap();

        let scanner = FileScanner::new();
        let files = scanner.discover_files(temp_path).unwrap();

        assert!(files.iter().any(|f| f.path == test_file));
    }

    #[test]
    fn test_is_source_file() {
        assert!(is_source_file(Path::new("test.py")));
        assert!(is_source_file(Path::new("test.rs")));
        assert!(!is_source_file(Path::new("test.txt")));
        assert!(!is_source_file(Path::new("test.pdf")));
    }

    #[test]
    fn test_is_config_file() {
        assert!(is_config_file(Path::new("config.json")));
        assert!(is_config_file(Path::new("settings.yaml")));
        assert!(is_config_file(Path::new(".env")));
        assert!(!is_config_file(Path::new("test.py")));
    }

    #[test]
    fn test_get_file_extension() {
        assert_eq!(get_file_extension(Path::new("test.py")), Some("py".to_string()));
        assert_eq!(get_file_extension(Path::new("TEST.RS")), Some("rs".to_string()));
        assert_eq!(get_file_extension(Path::new("noext")), None);
    }
}
