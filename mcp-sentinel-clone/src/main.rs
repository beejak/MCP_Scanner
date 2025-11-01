use clap::{Parser, ValueEnum};
use mcp_sentinel_clone::{
    config::load_config,
    models::vulnerability::Severity,
    proxy::start_proxy,
    scanner::Scanner,
    output::{terminal, json, sarif},
};
use std::process::ExitCode;
use tracing_subscriber::{FmtSubscriber, EnvFilter};

#[derive(Parser)]
#[command(
    name = "mcp-sentinel-clone",
    version = "0.1.0",
    about = "A clone of the MCP Sentinel security scanner."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
enum Commands {
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
enum OutputFormat {
    Terminal,
    Json,
    Sarif,
}

#[tokio::main]
async fn main() -> ExitCode {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Scan { target, output, config, severity, fail_on } => {
            let config = match load_config(config.as_deref()) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error loading configuration: {}", e);
                    return ExitCode::from(2);
                }
            };

            let scanner = Scanner::new(config);
            match scanner.scan_directory(&target).await {
                Ok(mut result) => {
                    result.vulnerabilities.retain(|v| v.severity >= severity);
                    match output {
                        OutputFormat::Terminal => terminal::display_scan_result(&result),
                        OutputFormat::Json => json::display_scan_result(&result),
                        OutputFormat::Sarif => sarif::display_scan_result(&result),
                    }
                    if result.vulnerabilities.iter().any(|v| v.severity >= fail_on) {
                        ExitCode::from(1)
                    } else {
                        ExitCode::SUCCESS
                    }
                }
                Err(e) => {
                    eprintln!("Error during scan: {}", e);
                    ExitCode::from(2)
                }
            }
        }
        Commands::Proxy { port } => {
            start_proxy(port).await;
            ExitCode::SUCCESS
        }
    };

    result
}
