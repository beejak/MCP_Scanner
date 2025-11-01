use anyhow::{Context, Result};
use crate::models::config::{ScanConfig, RawScanConfig};
use std::fs;
use std::path::Path;

pub fn load_config(path: Option<&str>) -> Result<ScanConfig> {
    let default_config = ScanConfig::default();
    let file_path = path.map(Path::new);

    let config_path = if let Some(p) = file_path {
        if p.exists() {
            Some(p.to_path_buf())
        } else {
            anyhow::bail!("Config file not found at: {}", p.display());
        }
    } else {
        // Search for the default config file in the current directory
        let default_filename = ".mcp-sentinel-clone.yaml";
        let current_dir_path = Path::new(default_filename);
        if current_dir_path.exists() {
            Some(current_dir_path.to_path_buf())
        } else {
            None
        }
    };

    let final_config = if let Some(p) = config_path {
        let yaml_str = fs::read_to_string(p).context("Failed to read config file")?;
        let raw_config: RawScanConfig = serde_yaml::from_str(&yaml_str).context("Failed to parse YAML config")?;
        default_config.merge(raw_config)
    } else {
        default_config
    };

    Ok(final_config)
}
