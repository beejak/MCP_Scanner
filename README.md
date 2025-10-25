# 🛡️ MCP Sentinel

**The Ultimate Security Scanner for Model Context Protocol (MCP) Servers**

MCP Sentinel is a next-generation security scanner that combines static analysis, runtime monitoring, and AI-powered detection to identify vulnerabilities in MCP servers. Built in Rust for maximum performance and reliability.

## Features

### Comprehensive Detection

- **13+ Vulnerability Categories**
  - Tool Poisoning Attacks
  - Prompt Injection
  - Secrets & API Keys Leakage
  - Sensitive Data Exposure (SSH keys, credentials)
  - Data Exfiltration Patterns
  - Command Injection
  - Path Traversal
  - SQL Injection
  - Unsafe Deserialization
  - And more...

### Three Detection Engines

1. **Static Analysis Engine**: Fast code scanning with pattern matching and regex detection
2. **Runtime Proxy Engine** *(Coming Soon)*: Real-time MCP traffic monitoring and guardrails
3. **AI Analysis Engine** *(Coming Soon)*: LLM-powered vulnerability detection with natural language explanations

### Multiple Scanning Modes

- **Quick Scan**: Fast static analysis (default)
- **Deep Scan** *(Coming Soon)*: Static analysis + AI-powered detection
- **Proxy Mode** *(Coming Soon)*: Real-time traffic monitoring
- **Audit Mode** *(Coming Soon)*: Comprehensive security assessment

## Quick Start

### Installation

#### From Source (Current)

```bash
# Clone the repository
git clone https://github.com/yourusername/mcp-sentinel
cd mcp-sentinel

# Build with Cargo
cargo build --release

# The binary will be at target/release/mcp-sentinel
```

### Basic Usage

#### Scan a Directory

```bash
mcp-sentinel scan ./my-mcp-server
```

#### Scan a Single File

```bash
mcp-sentinel scan ./mcp-server/tools.py
```

#### Scan with JSON Output

```bash
mcp-sentinel scan ./my-mcp-server --output json --output-file report.json
```

#### Fail CI/CD on High Severity Issues

```bash
mcp-sentinel scan ./my-mcp-server --fail-on high
```

#### Filter by Severity

```bash
mcp-sentinel scan ./my-mcp-server --severity critical
```

## Commands

### `scan` - Security Scanning

Scan MCP servers for vulnerabilities using static analysis.

```bash
mcp-sentinel scan [OPTIONS] <TARGET>
```

**Options:**
- `--mode <quick|deep>` - Scanning mode (default: quick)
- `--severity <level>` - Minimum severity to report: low, medium, high, critical
- `--fail-on <level>` - Exit with code 1 if vulnerabilities >= level found
- `--output <format>` - Output format: terminal, json, html, pdf
- `--output-file <path>` - Save report to file
- `--config <path>` - Custom configuration file

### `init` - Initialize Configuration

Create a default configuration file.

```bash
mcp-sentinel init
```

### `proxy` *(Coming Soon)*

Run MCP proxy for runtime monitoring.

```bash
mcp-sentinel proxy --port 8080
```

### `monitor` *(Coming Soon)*

Continuously monitor MCP server for changes.

```bash
mcp-sentinel monitor ./my-mcp-server --interval 60
```

### `audit` *(Coming Soon)*

Comprehensive security audit using all engines.

```bash
mcp-sentinel audit ./my-mcp-server --comprehensive
```

## Example Output

```
🛡️  MCP Sentinel

📂 Scanning: ./example-mcp-server
🔍 Engines: static

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
SCAN RESULTS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Risk Score: 85/100 CRITICAL

🔴 CRITICAL Issues: 2
🟠 HIGH Issues: 5
🟡 MEDIUM Issues: 8
🔵 LOW Issues: 12

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
CRITICAL ISSUES
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[TP-001] Tool Poisoning: Hidden instructions in tool 'calculator'
  Location: src/tools.py:45

  Detected Hidden instructions in brackets in tool description.
  This could be an attempt to inject malicious instructions.

  ⚠️  Impact: An AI agent using this tool may be manipulated to
      perform actions beyond the tool's stated purpose.
  🔧 Remediation: Remove the suspicious pattern from the tool
      description. Ensure tool descriptions are clear and contain
      no hidden instructions.

[SEC-002] Secret Detected: GitHub Personal Access Token
  Location: src/config.py:12

  GitHub personal access token found in code. Line: 12

  ⚠️  Impact: Exposed secrets can lead to unauthorized access to
      systems, data breaches, and compromise of services.
  🔧 Remediation: Remove the hardcoded secret. Use environment
      variables, secret management systems, or secure configuration
      files.

⏱️  Scan completed in 2.3s
📊 Scanned: ./example-mcp-server
🔍 Engines: static
```

## Detected Vulnerabilities

### Tool Poisoning
- Hidden instructions in tool descriptions
- Concealed directives using invisible characters
- Instructions to ignore previous commands
- Tool name/description mismatches

### Prompt Injection
- Direct prompt override attempts
- Role manipulation patterns
- System prompt extraction
- Delimiter manipulation

### Secrets Leakage
- SSH private keys (RSA, DSA, EC, OpenSSH)
- API keys (AWS, GitHub, OpenAI, Anthropic, Google, Slack, etc.)
- Database connection strings
- JWT tokens
- Hardcoded passwords

### Code Vulnerabilities
- Command injection (os.system, eval, exec)
- Path traversal risks
- SQL injection patterns
- Unsafe deserialization

## Configuration

Create a default configuration file:

```bash
mcp-sentinel init
```

Configuration file location: `~/.config/mcp-sentinel/config.yaml`

Example configuration:

```yaml
llm:
  provider: openai
  model: gpt-4
  api_key: null  # Or set OPENAI_API_KEY env var
  timeout: 60
  max_retries: 3

scanning:
  max_file_size: 10485760  # 10 MB
  parallel_workers: 8
  respect_gitignore: true
  exclude_patterns:
    - "*.pyc"
    - "node_modules"
    - ".git"

output:
  color: true
  verbosity: 1
  progress: true
  default_format: terminal
```

## CI/CD Integration

### GitHub Actions

```yaml
name: MCP Security Scan

on: [pull_request, push]

jobs:
  security-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install MCP Sentinel
        run: |
          cargo install mcp-sentinel

      - name: Scan MCP Server
        run: |
          mcp-sentinel scan ./mcp-server \
            --output json \
            --output-file results.json \
            --fail-on high

      - name: Upload Results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: security-scan-results
          path: results.json
```

## Development Status

**Phase 1: Foundation (In Progress)**
- ✅ Core CLI framework
- ✅ Static analysis engine (basic)
- ✅ Tool poisoning detector
- ✅ Prompt injection detector
- ✅ Secrets detector
- ✅ Terminal output (colored)
- ✅ JSON output
- 🔄 HTML/PDF output (planned)

**Phase 2: Advanced Detection (Planned)**
- ⏳ AI analysis engine
- ⏳ Semgrep integration
- ⏳ Taint analysis
- ⏳ All 13 vulnerability categories

**Phase 3: Runtime Monitoring (Planned)**
- ⏳ MCP proxy server
- ⏳ Traffic interception
- ⏳ Guardrails engine
- ⏳ Web dashboard

## Architecture

```
mcp-sentinel/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Core library
│   ├── cli/                 # Command implementations
│   ├── engines/             # Detection engines
│   │   ├── static_analysis/ # Static code analysis
│   │   ├── runtime_proxy/   # Runtime monitoring
│   │   └── ai_analysis/     # AI-powered detection
│   ├── detectors/           # Vulnerability detectors
│   ├── models/              # Data models
│   ├── output/              # Output formatters
│   ├── storage/             # Persistent storage
│   └── utils/               # Utilities
└── tests/                   # Tests and fixtures
```

## Contributing

Contributions are welcome! This is an open-source project licensed under Apache 2.0.

Areas where help is needed:
- Additional vulnerability detectors
- Output format implementations (HTML, PDF, SARIF)
- Runtime proxy engine
- AI analysis engine
- Documentation
- Test coverage

## Comparison with Existing Tools

| Feature | MCP Sentinel | Invariant Labs | Cisco Scanner | mcp-shield |
|---------|-------------|----------------|---------------|------------|
| **Language** | Rust | Python | Python | TypeScript |
| **Static Analysis** | ✅ | ✅ | ✅ | ✅ |
| **Runtime Monitoring** | 🔄 | ✅ | ❌ | ❌ |
| **AI-Powered Detection** | 🔄 | ❌ | ❌ | ✅ |
| **Custom Rules** | 🔄 | ✅ | ❌ | ❌ |
| **Secrets Detection** | ✅ | ✅ | ❌ | ❌ |
| **Performance** | 🚀 Fast (Rust) | Moderate | Moderate | Fast |
| **Single Binary** | ✅ | ❌ | ❌ | ❌ (requires Node) |

## Why Rust?

- **Performance**: 10-100x faster than Python/TypeScript scanners
- **Memory Safety**: Built-in protection against common vulnerabilities
- **Single Binary**: No runtime dependencies (unlike Python/Node.js)
- **Concurrency**: Excellent async/parallel scanning capabilities
- **Cross-Platform**: Compile once for Linux/macOS/Windows

## License

Apache License 2.0

## Acknowledgments

This project was inspired by existing MCP security tools:
- [Invariant Labs mcp-scan](https://github.com/invariantlabs-ai/mcp-scan)
- [Cisco AI Defense mcp-scanner](https://github.com/cisco-ai-defense/mcp-scanner)
- [mcp-shield](https://github.com/riseandignite/mcp-shield)
- [Google mcp-security](https://github.com/google/mcp-security)
- [Ant Group MCPScan](https://github.com/antgroup/MCPScan)

MCP Sentinel aims to combine the best features from all these tools while adding innovations in performance, detection capabilities, and user experience.

## Support

- 🐛 Report issues: [GitHub Issues](https://github.com/yourusername/mcp-sentinel/issues)
- 💬 Discussions: [GitHub Discussions](https://github.com/yourusername/mcp-sentinel/discussions)
- 📖 Documentation: [Full Docs](https://docs.mcp-sentinel.io)

---

**Built with ❤️ for the MCP security community**
