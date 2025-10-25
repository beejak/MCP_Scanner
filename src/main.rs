use clap::{Parser, Subcommand};
use mcp_sentinel::cli;
use mcp_sentinel::output::terminal;
use mcp_sentinel::models::config::Config;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use anyhow::Result;
use std::process;

/// MCP Sentinel - The Ultimate Security Scanner for MCP Servers
#[derive(Parser)]
#[command(
    name = "mcp-sentinel",
    version = env!("CARGO_PKG_VERSION"),
    about = "Security scanner for Model Context Protocol (MCP) servers",
    long_about = "MCP Sentinel combines static analysis, runtime monitoring, and AI-powered \
                  detection to identify security vulnerabilities in MCP servers. \n\n\
                  Detects: tool poisoning, prompt injection, secrets leakage, data exfiltration, \
                  toxic flows, and 13+ vulnerability categories."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan MCP server for vulnerabilities (static + AI analysis)
    Scan {
        /// Target to scan (directory, file, or GitHub URL)
        target: String,

        /// Scanning mode: quick (static only) or deep (static + AI)
        #[arg(long, value_enum, default_value = "quick")]
        mode: cli::scan::ScanMode,

        /// Minimum severity to report: low, medium, high, critical
        #[arg(long, default_value = "low")]
        severity: String,

        /// Exit with code 1 if vulnerabilities >= this level found
        #[arg(long)]
        fail_on: Option<String>,

        /// Output format: terminal, json, html, pdf
        #[arg(long, default_value = "terminal")]
        output: String,

        /// Save report to file
        #[arg(long)]
        output_file: Option<String>,

        /// LLM provider for deep scan: openai, anthropic, local
        #[arg(long)]
        llm_provider: Option<String>,

        /// LLM model (e.g., gpt-4, claude-3-opus)
        #[arg(long)]
        llm_model: Option<String>,

        /// LLM API key (or use env var)
        #[arg(long, env = "MCP_SENTINEL_LLM_API_KEY")]
        llm_api_key: Option<String>,

        /// Custom configuration file
        #[arg(long)]
        config: Option<String>,
    },

    /// Run MCP proxy for runtime monitoring
    Proxy {
        /// MCP configuration file to proxy
        #[arg(long)]
        config: Option<String>,

        /// Proxy listen port
        #[arg(long, default_value = "8080")]
        port: u16,

        /// Custom guardrails rules file
        #[arg(long)]
        guardrails: Option<String>,

        /// Save all MCP traffic to log file
        #[arg(long)]
        log_traffic: bool,

        /// Traffic log destination
        #[arg(long)]
        log_file: Option<String>,

        /// Block requests >= this risk level
        #[arg(long)]
        block_on_risk: Option<String>,

        /// Send alerts to webhook URL
        #[arg(long)]
        alert_webhook: Option<String>,

        /// Launch web dashboard
        #[arg(long)]
        dashboard: bool,
    },

    /// Continuously monitor MCP server for changes
    Monitor {
        /// Target to monitor
        target: String,

        /// Rescan interval in seconds
        #[arg(long, default_value = "300")]
        interval: u64,

        /// Watch for file changes and rescan
        #[arg(long)]
        watch: bool,

        /// Run as background daemon
        #[arg(long)]
        daemon: bool,

        /// Daemon PID file location
        #[arg(long)]
        pid_file: Option<String>,

        /// Alert on vulnerabilities >= this level
        #[arg(long)]
        alert_on: Option<String>,
    },

    /// Comprehensive audit (static + runtime + AI)
    Audit {
        /// Target to audit
        target: String,

        /// Include runtime analysis (launches temporary proxy)
        #[arg(long)]
        include_proxy: bool,

        /// Proxy duration for runtime analysis (seconds)
        #[arg(long, default_value = "300")]
        duration: u64,

        /// Maximum depth analysis (slow)
        #[arg(long)]
        comprehensive: bool,

        /// Output format
        #[arg(long, default_value = "terminal")]
        output: String,

        /// Save report to file
        #[arg(long)]
        output_file: Option<String>,
    },

    /// Initialize configuration
    Init {
        /// Configuration file location
        #[arg(long)]
        config_path: Option<String>,
    },

    /// Manage whitelist of trusted tools/servers
    Whitelist {
        #[command(subcommand)]
        command: WhitelistCommands,
    },

    /// Manage custom guardrails rules
    Rules {
        #[command(subcommand)]
        command: RulesCommands,
    },
}

#[derive(Subcommand)]
enum WhitelistCommands {
    /// Add tool/server to whitelist
    Add {
        /// Type: tool or server
        r#type: String,
        /// Name
        name: String,
        /// SHA-256 hash
        hash: String,
    },

    /// Remove from whitelist
    Remove {
        /// Hash to remove
        hash: String,
    },

    /// List whitelist entries
    List,

    /// Export whitelist to file
    Export {
        /// Output file path
        path: String,
    },

    /// Import whitelist from file
    Import {
        /// Input file path
        path: String,
    },
}

#[derive(Subcommand)]
enum RulesCommands {
    /// Validate guardrails rules syntax
    Validate {
        /// Rules file path
        path: String,
    },

    /// List available rule templates
    List,

    /// Test rules against sample traffic
    Test {
        /// Rules file path
        rules_path: String,
        /// Test traffic JSON file
        traffic_path: String,
    },
}

#[tokio::main]
async fn main() {
    let exit_code = match run().await {
        Ok(code) => code,
        Err(e) => {
            eprintln!("Error: {}", e);
            2 // Error code 2 for scan errors
        }
    };
    process::exit(exit_code);
}

async fn run() -> Result<i32> {
    let cli = Cli::parse();

    // Initialize tracing
    let log_level = if cli.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("mcp_sentinel={}", log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Disable colors if requested
    if cli.no_color {
        terminal::disable_colors();
    }

    // Load configuration
    let config = Config::default();

    // Route to appropriate command handler
    match cli.command {
        Commands::Scan {
            target,
            mode,
            severity,
            fail_on,
            output,
            output_file,
            llm_provider,
            llm_model,
            llm_api_key,
            config: config_file,
        } => {
            cli::scan::run(
                target,
                mode,
                severity,
                fail_on,
                output,
                output_file,
                llm_provider,
                llm_model,
                llm_api_key,
                config_file,
                config,
            )
            .await
        }

        Commands::Proxy {
            config: config_file,
            port,
            guardrails,
            log_traffic,
            log_file,
            block_on_risk,
            alert_webhook,
            dashboard,
        } => {
            cli::proxy::run(
                config_file,
                port,
                guardrails,
                log_traffic,
                log_file,
                block_on_risk,
                alert_webhook,
                dashboard,
                config,
            )
            .await
        }

        Commands::Monitor {
            target,
            interval,
            watch,
            daemon,
            pid_file,
            alert_on,
        } => {
            cli::monitor::run(
                target,
                interval,
                watch,
                daemon,
                pid_file,
                alert_on,
                config,
            )
            .await
        }

        Commands::Audit {
            target,
            include_proxy,
            duration,
            comprehensive,
            output,
            output_file,
        } => {
            cli::audit::run(
                target,
                include_proxy,
                duration,
                comprehensive,
                output,
                output_file,
                config,
            )
            .await
        }

        Commands::Init { config_path } => {
            cli::init::run(config_path).await
        }

        Commands::Whitelist { command } => {
            cli::whitelist::run(command).await
        }

        Commands::Rules { command } => {
            cli::rules::run(command).await
        }
    }
}
