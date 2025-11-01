use clap::{Parser, ValueEnum};
use crate::models::vulnerability::Severity;

#[derive(Parser)]
#[command(
    name = "mcp-sentinel-clone",
    version = "0.1.0",
    about = "A clone of the MCP Sentinel security scanner."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Parser)]
pub enum Commands {
    /// Scan a directory for vulnerabilities.
    Scan {
        /// The path to the directory to scan.
        #[arg(required = true)]
        target: String,
        /// The output format.
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Terminal)]
        output: OutputFormat,
        /// Path to a custom configuration file.
        #[arg(short, long)]
        config: Option<String>,
        /// Minimum severity to report.
        #[arg(long, value_enum, default_value_t = Severity::Low)]
        severity: Severity,
        /// Exit with code 1 if vulnerabilities at or above this level are found.
        #[arg(long, value_enum, default_value_t = Severity::Low)]
        fail_on: Severity,
    },
    /// Run as a forwarding proxy.
    Proxy {
        /// The port to listen on.
        #[arg(short, long, default_value_t = 8000)]
        port: u16,
    },
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OutputFormat {
    Terminal,
    Json,
    Sarif,
}
