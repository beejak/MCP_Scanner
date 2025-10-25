# MCP Sentinel

🛡️ The Ultimate Security Scanner for MCP Servers

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)

MCP Sentinel is a next-generation security scanner for Model Context Protocol (MCP) servers that combines static analysis, runtime monitoring, and AI-powered detection in a single, high-performance Rust binary.

## ⚡ Features

- **5 Detection Categories (Phase 1 Complete)**:
  - 🔐 Secrets Detection (15+ patterns including AWS keys, API keys, private keys)
  - 💉 Command Injection (Python, JavaScript/TypeScript patterns)
  - 📁 Sensitive File Access (SSH keys, AWS credentials, browser cookies)
  - 🎣 Tool Poisoning (invisible Unicode, malicious keywords)
  - 🔓 Prompt Injection (jailbreak patterns, system prompt manipulation)

- **Beautiful Terminal Output**:
  - Colored, hierarchical vulnerability display
  - Risk scoring (0-100)
  - Detailed remediation guidance
  - Code snippets with location info

- **Multiple Output Formats**:
  - Terminal (with colors and progress bars)
  - JSON (for CI/CD integration)
  - SARIF 2.1.0 (GitHub Code Scanning, GitLab, SonarQube, VS Code)
  - HTML, PDF (coming in Phase 2-4)

- **High Performance**:
  - Written in Rust for blazing speed
  - Concurrent file scanning
  - Real-time progress indicators
  - Target: <2s for small MCP servers

- **Configuration & CI/CD**:
  - YAML configuration files (~/.mcp-sentinel/config.yaml)
  - Standardized exit codes (0=clean, 1=vulnerabilities, 2=error, 3=usage)
  - Perfect for CI/CD pipelines

- **MCP-Specific Security** (NEW in Phase 1.6):
  - Scans Claude Desktop, Cline, and other MCP client configurations
  - Detects insecure HTTP connections
  - Identifies hardcoded credentials in config files
  - Flags overly permissive tool access

## 🚀 Quick Start

### Installation

```bash
# Using Cargo (when published)
cargo install mcp-sentinel

# Or build from source
git clone https://github.com/yourusername/MCP_Scanner
cd MCP_Scanner
cargo build --release
```

### Basic Usage

```bash
# Scan a local MCP server directory
mcp-sentinel scan ./my-mcp-server

# Scan with JSON output
mcp-sentinel scan ./my-mcp-server --output json

# Generate SARIF output for GitHub Code Scanning
mcp-sentinel scan ./my-mcp-server --output sarif --output-file results.sarif

# Fail CI/CD if high-severity issues found
mcp-sentinel scan ./my-mcp-server --fail-on high

# Use custom configuration file
mcp-sentinel scan ./my-mcp-server --config my-config.yaml

# Scan with minimum severity filter
mcp-sentinel scan ./my-mcp-server --severity medium
```

### Configuration File

Create `~/.mcp-sentinel/config.yaml` or `.mcp-sentinel.yaml` in your project:

```yaml
version: "1.0"
scan:
  mode: quick              # or: deep
  min_severity: low        # low, medium, high, critical
  max_file_size: 10485760  # 10MB in bytes
  parallel_workers: 8
  exclude_patterns:
    - "node_modules/"
    - ".git/"
    - "target/"
    - "dist/"
```

Configuration priority: CLI flags > project config (./.mcp-sentinel.yaml) > user config (~/.mcp-sentinel/config.yaml) > defaults

## 📊 Phase 1 Implementation Status

### ✅ Completed

- [x] Project structure and build configuration
- [x] CLI framework (7 commands: scan, proxy, monitor, audit, init, whitelist, rules)
- [x] Core data models (Vulnerability, ScanResult, Config)
- [x] File discovery and traversal utilities
- [x] Terminal output renderer with colors
- [x] JSON output generator
- [x] 5 vulnerability detectors with comprehensive patterns
- [x] Scanner engine with parallel processing
- [x] Scan command fully functional
- [x] Test fixtures with vulnerable code samples

### 🔄 In Progress / Next Steps

**Phase 2 (Weeks 5-8):**
- [ ] Semgrep integration
- [ ] Tree-sitter code parsing
- [ ] AI analysis engine (OpenAI, Anthropic, Ollama)
- [ ] HTML report generator
- [ ] GitHub repository scanning
- [ ] Configuration file support
- [ ] Additional detectors (PII, toxic flows, anomalies)

**Phase 3 (Weeks 9-12):**
- [ ] Runtime proxy engine
- [ ] Guardrails enforcement
- [ ] Web dashboard
- [ ] Real-time monitoring
- [ ] Rug pull detection

**Phase 4 (Weeks 13-16):**
- [ ] PDF report generation
- [ ] SARIF output format
- [ ] Whitelist management
- [ ] Performance optimizations
- [ ] Comprehensive documentation

## 🛠️ Architecture

```
mcp-sentinel/
├── src/
│   ├── cli/           # Command implementations
│   ├── detectors/     # Vulnerability detectors
│   ├── engines/       # Scanning engines
│   ├── models/        # Data models
│   ├── output/        # Report formatters
│   ├── storage/       # State management
│   ├── utils/         # Utilities
│   └── scanner.rs     # Main scanner API
├── tests/
│   └── fixtures/      # Test vulnerable servers
└── Cargo.toml
```

## 🎯 Detection Capabilities

### Secrets Detection (15+ Patterns)
- AWS Access Keys (AKIA*, ASIA*)
- OpenAI API Keys
- Anthropic API Keys
- JWT Tokens
- Private Keys (RSA, EC, OpenSSH)
- Database Connection Strings
- GitHub Tokens
- Slack Tokens
- Google API Keys
- Hardcoded Passwords

### Command Injection
- Python: `os.system()`, `subprocess` with `shell=True`, `eval()`, `exec()`
- JavaScript: `child_process.exec()`, `eval()`, `Function()` constructor

### Sensitive File Access
- SSH keys (id_rsa, id_ed25519)
- AWS credentials (~/.aws/credentials)
- GCP credentials (~/.config/gcloud/)
- Environment files (.env)
- Browser cookies
- Shell RC files

### Tool Poisoning
- Invisible Unicode characters
- Keywords: "ignore", "disregard", "override", "actually"
- Hidden markers: [HIDDEN:], [SECRET:]

### Prompt Injection
- System prompt manipulation
- Role confusion
- Jailbreak attempts

## 📝 Example Output

```
🛡️  MCP Sentinel v1.0.0

📂 Scanning: ./vulnerable-server
🔍 Engines: Static Analysis ✓

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊 SCAN RESULTS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Risk Score: 85/100 🔴 CRITICAL

🔴 CRITICAL Issues: 4
🟠 HIGH Issues: 2
🟡 MEDIUM Issues: 1
🔵 LOW Issues: 0

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔴 CRITICAL ISSUES
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[SEC-001] AWS Access Key ID Found
  Location: server.py:10

  AWS Access Key ID detected

  ⚠️  Impact: Exposed AWS Access Key ID can be used for unauthorized access
  🔧 Remediation: Remove AWS Access Key ID from source code and use environment variables

⏱️  Scan completed in 1.2s
```

## 🧪 Testing

Test fixtures are available in `tests/fixtures/vulnerable_servers/`:

```bash
# Test the scanner on vulnerable fixtures
mcp-sentinel scan tests/fixtures/vulnerable_servers/test-server/
```

## 📖 Documentation

- [Installation Guide](docs/installation.md) (coming soon)
- [User Guide](docs/user-guide/) (coming soon)
- [API Reference](docs/reference/) (coming soon)
- [Contributing](docs/contributing/) (coming soon)

## 🤝 Contributing

MCP Sentinel is in active development. Phase 1 (foundation) is complete. Contributions welcome!

## 📄 License

Apache 2.0 - See [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

Built with reference to the excellent work by:
- Invariant Labs (mcp-scan)
- Google (mcp-security)
- Antgroup (MCPScan)
- Rise and Ignite (mcp-shield)

---

**Status**: Phase 1 Complete ✅ | Next: Phase 2 (AI Analysis & Advanced Detection)
