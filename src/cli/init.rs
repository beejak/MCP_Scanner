use crate::models::config::Config;
use anyhow::Result;
use std::path::PathBuf;

pub async fn run(_config_path: Option<String>) -> Result<i32> {
    println!("🛡️  MCP Sentinel - Initialization\n");

    // Determine config path
    let config = Config::default();
    let config_path = _config_path
        .map(PathBuf::from)
        .unwrap_or_else(|| config.storage.config_dir.join("config.yaml"));

    // Create config directories if they don't exist
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Create default configuration
    println!("Creating default configuration at: {}", config_path.display());

    config.save_to_file(&config_path)?;

    println!("✅ Configuration file created successfully!");
    println!("\nNext steps:");
    println!("  1. Edit the configuration file to customize settings");
    println!("  2. Run 'mcp-sentinel scan <target>' to start scanning");
    println!("\nExample:");
    println!("  mcp-sentinel scan ./my-mcp-server\n");

    Ok(0)
}
