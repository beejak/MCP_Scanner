use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for MCP Sentinel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// LLM configuration for AI analysis
    pub llm: LlmConfig,

    /// Scanning configuration
    pub scanning: ScanningConfig,

    /// Output configuration
    pub output: OutputConfig,

    /// Proxy configuration
    pub proxy: ProxyConfig,

    /// Storage paths
    pub storage: StorageConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            llm: LlmConfig::default(),
            scanning: ScanningConfig::default(),
            output: OutputConfig::default(),
            proxy: ProxyConfig::default(),
            storage: StorageConfig::default(),
        }
    }
}

/// LLM provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Provider: openai, anthropic, or local
    pub provider: String,

    /// Model name (e.g., gpt-4, claude-3-opus-20240229)
    pub model: Option<String>,

    /// API key (can be overridden by env var)
    pub api_key: Option<String>,

    /// Base URL for local LLM (e.g., Ollama)
    pub base_url: Option<String>,

    /// Timeout in seconds
    pub timeout: u64,

    /// Max retries for rate limits
    pub max_retries: u32,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            model: None,
            api_key: None,
            base_url: None,
            timeout: 60,
            max_retries: 3,
        }
    }
}

/// Scanning behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanningConfig {
    /// Maximum file size to scan (bytes)
    pub max_file_size: u64,

    /// Number of parallel workers
    pub parallel_workers: usize,

    /// Respect .gitignore files
    pub respect_gitignore: bool,

    /// File patterns to exclude
    pub exclude_patterns: Vec<String>,

    /// File patterns to include
    pub include_patterns: Vec<String>,

    /// Follow symbolic links
    pub follow_symlinks: bool,

    /// Maximum directory depth
    pub max_depth: Option<usize>,
}

impl Default for ScanningConfig {
    fn default() -> Self {
        Self {
            max_file_size: 10_485_760, // 10 MB
            parallel_workers: num_cpus::get(),
            respect_gitignore: true,
            exclude_patterns: vec![
                "*.pyc".to_string(),
                "*.pyo".to_string(),
                "node_modules".to_string(),
                ".git".to_string(),
                "__pycache__".to_string(),
            ],
            include_patterns: vec![
                "*.py".to_string(),
                "*.ts".to_string(),
                "*.js".to_string(),
                "*.json".to_string(),
            ],
            follow_symlinks: false,
            max_depth: None,
        }
    }
}

/// Output formatting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Enable colored output
    pub color: bool,

    /// Verbosity level (0-3)
    pub verbosity: u8,

    /// Show progress bars
    pub progress: bool,

    /// Default output format
    pub default_format: String,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            color: true,
            verbosity: 1,
            progress: true,
            default_format: "terminal".to_string(),
        }
    }
}

/// Proxy server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Default listen port
    pub port: u16,

    /// Default bind address
    pub bind_address: String,

    /// Enable request logging
    pub log_traffic: bool,

    /// Enable web dashboard
    pub dashboard: bool,

    /// Dashboard port
    pub dashboard_port: u16,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            bind_address: "127.0.0.1".to_string(),
            log_traffic: false,
            dashboard: false,
            dashboard_port: 9090,
        }
    }
}

/// Storage paths configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Config directory path
    pub config_dir: PathBuf,

    /// Cache directory path
    pub cache_dir: PathBuf,

    /// Data directory path
    pub data_dir: PathBuf,

    /// Whitelist database path
    pub whitelist_db: PathBuf,
}

impl Default for StorageConfig {
    fn default() -> Self {
        let dirs = directories::ProjectDirs::from("io", "mcp-sentinel", "mcp-sentinel")
            .expect("Failed to determine project directories");

        let config_dir = dirs.config_dir().to_path_buf();
        let cache_dir = dirs.cache_dir().to_path_buf();
        let data_dir = dirs.data_dir().to_path_buf();
        let whitelist_db = data_dir.join("whitelist.db");

        Self {
            config_dir,
            cache_dir,
            data_dir,
            whitelist_db,
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn from_file(path: &PathBuf) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path)?;

        if path.extension().and_then(|s| s.to_str()) == Some("yaml") ||
           path.extension().and_then(|s| s.to_str()) == Some("yml") {
            Ok(serde_yaml::from_str(&contents)?)
        } else if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            Ok(toml::from_str(&contents)?)
        } else {
            anyhow::bail!("Unsupported config file format. Use .yaml or .toml")
        }
    }

    /// Save configuration to file
    pub fn save_to_file(&self, path: &PathBuf) -> anyhow::Result<()> {
        let contents = if path.extension().and_then(|s| s.to_str()) == Some("yaml") ||
                          path.extension().and_then(|s| s.to_str()) == Some("yml") {
            serde_yaml::to_string(self)?
        } else if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            toml::to_string(self)?
        } else {
            anyhow::bail!("Unsupported config file format. Use .yaml or .toml")
        };

        std::fs::write(path, contents)?;
        Ok(())
    }
}
