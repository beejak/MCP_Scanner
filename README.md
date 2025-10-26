# MCP Sentinel

🛡️ Enterprise-Grade Security Scanner for MCP Servers

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Version](https://img.shields.io/badge/version-2.5.0-green.svg)](https://github.com/beejak/MCP_Scanner/releases/tag/v2.5.0)
[![Release](https://img.shields.io/github/v/release/beejak/MCP_Scanner)](https://github.com/beejak/MCP_Scanner/releases/latest)

MCP Sentinel is a next-generation security scanner for Model Context Protocol (MCP) servers that combines **semantic AST analysis**, **Semgrep integration**, **AI-powered detection**, **HTML reporting**, and **GitHub URL scanning** in a single, blazing-fast Rust binary.

## ⚡ Features

### 🚀 Phase 2.5 - Advanced Analysis (NEW!)

- **🌳 Tree-sitter AST Parsing**: Semantic code analysis for Python, JavaScript, TypeScript, Go
  - Dataflow analysis tracking tainted variables from sources to sinks
  - Context-aware vulnerability detection beyond regex patterns
  - Pattern-based detection for command injection, SQL injection, path traversal

- **🔍 Semgrep Integration**: Access 1000+ community SAST rules
  - Security-focused rule filtering
  - External process integration with seamless result mapping
  - Configurable severity thresholds

- **📊 HTML Report Generator**: Enterprise-ready interactive reports
  - Self-contained HTML with inline CSS/JavaScript
  - Risk scoring (0-100) with visual indicators
  - Expandable vulnerability cards with full details
  - Perfect for stakeholder presentations and compliance audits

- **🐙 GitHub URL Scanning**: Frictionless repository audits
  - Direct URL scanning without manual cloning
  - Shallow cloning (--depth=1) for 10-20x faster downloads
  - Parse owner/repo/branch/tag/commit from any GitHub URL

- **🛡️ Tool Description Analysis**: MCP-specific prompt injection detection
  - Detect AI manipulation attempts in tool metadata
  - Identify misleading descriptions and hidden instructions
  - Flag social engineering in tool documentation

### 🔒 Core Detection (Phase 1-2)

- **Secrets Detection**: 15+ patterns including AWS keys, API keys, JWT tokens, private keys
- **Command Injection**: Python, JavaScript/TypeScript dangerous function detection
- **Sensitive File Access**: SSH keys, AWS credentials, browser cookies, shell RC files
- **Tool Poisoning**: Invisible Unicode, malicious keywords, hidden markers
- **Prompt Injection**: Jailbreak patterns, system prompt manipulation, role confusion
- **MCP Config Security**: Insecure HTTP, hardcoded credentials, untrusted executables

### 📤 Output Formats

- **Terminal**: Colored, hierarchical vulnerability display with progress bars
- **JSON**: Structured output for CI/CD integration
- **SARIF 2.1.0**: GitHub Code Scanning, GitLab, SonarQube, VS Code integration
- **HTML**: Interactive dashboards with risk scoring and charts (Phase 2.5)

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
git clone https://github.com/beejak/MCP_Scanner
cd MCP_Scanner
git checkout v2.5.0
cargo build --release
```

### Basic Usage

```bash
# Quick scan (automatic AST analysis for Python/JS/TS/Go)
mcp-sentinel scan ./my-mcp-server

# Scan GitHub repository directly (no manual cloning!)
mcp-sentinel scan https://github.com/owner/mcp-server --fail-on high

# Scan specific branch or commit
mcp-sentinel scan https://github.com/owner/mcp-server/tree/develop
mcp-sentinel scan https://github.com/owner/mcp-server/commit/abc123

# Enable Semgrep integration (requires: pip install semgrep)
mcp-sentinel scan ./my-mcp-server --enable-semgrep

# Generate HTML report for stakeholders
mcp-sentinel scan ./my-mcp-server --output html --output-file report.html

# Comprehensive multi-engine scan
mcp-sentinel scan ./my-mcp-server \
  --mode deep \
  --enable-semgrep \
  --llm-provider openai \
  --output html \
  --output-file audit-report.html

# SARIF output for GitHub Code Scanning
mcp-sentinel scan ./my-mcp-server --output sarif --output-file results.sarif

# Fail CI/CD on high-severity issues
mcp-sentinel scan ./my-mcp-server --fail-on high
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

## 📊 Implementation Status

### ✅ Phase 1.6 Complete (NEW!)

**Production-Ready Features:**
- [x] **SARIF 2.1.0 Output** - Full GitHub Code Scanning, GitLab, SonarQube, VS Code integration
- [x] **Configuration File Support** - YAML configs with multi-level priority (CLI > project > user)
- [x] **MCP Config Scanner** - Detects security issues in Claude Desktop, Cline configs (6 security rules)
- [x] **Progress Indicators** - Real-time progress bars and spinners with smart TTY/CI detection
- [x] **Enhanced Exit Codes** - Standardized codes for CI/CD (0=clean, 1=vulns, 2=error, 3=usage)

**Edge Case Handling:**
- [x] Permission-specific error messages
- [x] Empty/partial config file handling
- [x] YAML syntax errors with line numbers
- [x] Environment variable detection (NO_COLOR, CI, MCP_SENTINEL_NO_PROGRESS)

### ✅ Phase 1 Complete

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

### 🔄 Next Steps

**Phase 2 (Upcoming):**
- [ ] Semgrep integration
- [ ] Tree-sitter code parsing
- [ ] AI analysis engine (OpenAI, Anthropic, Ollama)
- [ ] HTML report generator
- [ ] GitHub repository scanning
- [ ] Additional detectors (PII, toxic flows, anomalies)
- [ ] Baseline scanning & diff-aware scanning
- [ ] Vulnerability suppression (.mcp-sentinel-ignore)

**Phase 3:**
- [ ] Runtime proxy engine
- [ ] Guardrails enforcement
- [ ] Web dashboard
- [ ] Real-time monitoring
- [ ] Rug pull detection

**Phase 4:**
- [ ] PDF report generation
- [ ] Whitelist management
- [ ] Performance optimizations (caching, incremental scans)
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

### MCP Configuration Security (Phase 1.6)
- **Insecure HTTP Servers**: Detects non-HTTPS MCP server URLs (except localhost)
- **Untrusted Domains**: Flags suspicious TLDs, public IPs, unknown domains
- **Overly Permissive Paths**: Detects wildcard or root-level file access permissions
- **Missing SSL Verification**: Warns about missing certificate verification
- **Hardcoded Credentials**: Finds API keys, tokens, passwords in config files
- **Untrusted Executables**: Flags commands from /tmp or relative paths

**Scans these config files:**
- Claude Desktop: `config.json`, `claude_desktop_config.json`
- Cline: `.cline/mcp.json`
- Generic: Any `mcp*.json` or configs in `.claude/`, `.cline/`, `.mcp/` directories

## 🔄 Exit Codes (CI/CD Integration)

MCP Sentinel uses standardized exit codes for reliable CI/CD integration:

| Exit Code | Meaning | When It Happens |
|-----------|---------|----------------|
| **0** | Success | Scan completed with no issues, or all issues below `--fail-on` threshold |
| **1** | Vulnerabilities Found | Scan found vulnerabilities at or above `--fail-on` threshold |
| **2** | Scan Error | Target not found, invalid config, scan failure, or I/O error |
| **3** | Usage Error | Invalid arguments or command syntax (handled by CLI parser) |

### CI/CD Pipeline Example

```bash
# GitHub Actions / GitLab CI / Jenkins
mcp-sentinel scan ./my-server --fail-on high --output sarif --output-file results.sarif
EXIT_CODE=$?

if [ $EXIT_CODE -eq 1 ]; then
  echo "❌ Security vulnerabilities found"
  exit 1
elif [ $EXIT_CODE -eq 2 ]; then
  echo "❌ Scan failed with error"
  exit 2
elif [ $EXIT_CODE -eq 0 ]; then
  echo "✅ Scan passed"
fi
```

### GitHub Actions Integration

```yaml
name: MCP Security Scan
on: [push, pull_request]

jobs:
  security-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Run MCP Sentinel
        run: |
          mcp-sentinel scan . --output sarif --output-file results.sarif --fail-on high

      - name: Upload SARIF to GitHub Code Scanning
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: results.sarif
```

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

## 🎯 CI/CD Best Practices

### Configuration File Strategy
1. **Team Config**: Commit `.mcp-sentinel.yaml` to repo for team standards
2. **Personal Overrides**: Use `~/.mcp-sentinel/config.yaml` for local preferences
3. **CI Overrides**: Use CLI flags in CI for strictest settings

### SARIF Integration
- **GitHub**: Upload SARIF to Code Scanning for PR annotations
- **GitLab**: Use SARIF reports in Security Dashboard
- **VS Code**: Open SARIF files directly in Problems panel
- **SonarQube**: Import SARIF for vulnerability tracking

### Progress Indicators Control
Set environment variables to customize progress display:
- `MCP_SENTINEL_NO_PROGRESS=1` - Disable all progress indicators
- `NO_COLOR=1` - Disable colors (keeps progress structure)
- `CI=true` - Auto-detected in most CI environments

---

**Status**: Phase 1.6 Complete ✅ | Next: Phase 2 (AI Analysis & Advanced Detection)
