use anyhow::Result;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// Represents a discovered file for scanning
#[derive(Debug, Clone)]
pub struct DiscoveredFile {
    pub path: PathBuf,
    pub size: u64,
    pub is_binary: bool,
}

/// File scanner for discovering files to scan
pub struct FileScanner {
    max_file_size: u64,
    respect_gitignore: bool,
    follow_symlinks: bool,
    max_depth: Option<usize>,
}

impl FileScanner {
    /// Create a new file scanner with default settings
    pub fn new() -> Self {
        Self {
            max_file_size: 10_485_760, // 10 MB
            respect_gitignore: true,
            follow_symlinks: false,
            max_depth: None,
        }
    }

    /// Set maximum file size to scan
    pub fn max_file_size(mut self, size: u64) -> Self {
        self.max_file_size = size;
        self
    }

    /// Set whether to respect .gitignore files
    pub fn respect_gitignore(mut self, respect: bool) -> Self {
        self.respect_gitignore = respect;
        self
    }

    /// Set whether to follow symbolic links
    pub fn follow_symlinks(mut self, follow: bool) -> Self {
        self.follow_symlinks = follow;
        self
    }

    /// Set maximum directory depth
    pub fn max_depth(mut self, depth: Option<usize>) -> Self {
        self.max_depth = depth;
        self
    }

    /// Discover files in a directory
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

    /// Discover files matching specific patterns
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

/// Check if a file is binary by reading first few bytes
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

/// Read file contents as string
pub fn read_file_contents(path: &Path) -> Result<String> {
    Ok(std::fs::read_to_string(path)?)
}

/// Get file extension
pub fn get_file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
}

/// Check if file is likely source code
pub fn is_source_file(path: &Path) -> bool {
    const SOURCE_EXTENSIONS: &[&str] = &[
        "py", "js", "ts", "tsx", "jsx", "rs", "go", "java", "c", "cpp", "h", "hpp",
        "cs", "rb", "php", "swift", "kt", "scala", "clj", "sh", "bash", "zsh",
    ];

    get_file_extension(path)
        .map(|ext| SOURCE_EXTENSIONS.contains(&ext.as_str()))
        .unwrap_or(false)
}

/// Check if file is likely a config file
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
