use mcp_sentinel_clone::{
    config::load_config,
    engine::scanner::Scanner,
    proxy::start_proxy,
    output::{terminal, json, sarif},
    cli::{Cli, Commands, OutputFormat},
};
use std::process::ExitCode;
use tracing_subscriber::{FmtSubscriber, EnvFilter};
use clap::Parser;

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
