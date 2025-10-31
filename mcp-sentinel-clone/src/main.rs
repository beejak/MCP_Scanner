use clap::{Parser, ValueEnum};
use mcp_sentinel_clone::{
    models::config::ScanConfig,
    scanner::Scanner,
    output::{terminal, json},
};

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
    },
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum OutputFormat {
    Terminal,
    Json,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { target, output } => {
            let config = ScanConfig::default();
            let scanner = Scanner::new(config);
            match scanner.scan_directory(&target) {
                Ok(result) => {
                    match output {
                        OutputFormat::Terminal => terminal::display_scan_result(&result),
                        OutputFormat::Json => json::display_scan_result(&result),
                    }
                }
                Err(e) => {
                    eprintln!("Error during scan: {}", e);
                }
            }
        }
    }
}
