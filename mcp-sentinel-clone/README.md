# MCP Sentinel Clone

This project is a functional clone of the `mcp-sentinel` security scanner, built from scratch in Rust. It is a command-line tool that scans a given directory for a variety of security vulnerabilities.

## ⚡ Features

- **5 Detection Categories:**
  - 🔐 **Secrets Detection:** Finds hardcoded secrets like API keys and private keys.
  - 💉 **Command Injection:** Detects the use of dangerous functions that can lead to command injection in Python and JavaScript.
  - 📁 **Sensitive File Access:** Identifies code that accesses sensitive system files.
  - 🎣 **Tool Poisoning:** Looks for patterns that could be used to manipulate or poison the behavior of a language model.
  - 🔓 **Prompt Injection:** Detects common prompt injection techniques.

- **Multiple Output Formats:**
  - **Terminal:** A colorized, human-readable output.
  - **JSON:** A machine-readable format for easy integration with other tools.
  - **SARIF 2.1.0:** A standardized format for sharing static analysis results, compatible with GitHub Code Scanning.

- **High Performance:**
  - Built with Rust and `tokio` for concurrent, high-speed scanning.

- **Advanced Configuration:**
  - **YAML Configuration:** Configure the scanner using a `.mcp-sentinel-clone.yaml` file.
  - **Severity Filtering:** Filter results by severity level (`--severity`).
  - **CI/CD Integration:** Use the `--fail-on` flag to control exit codes based on vulnerability severity.

- **Semantic Analysis:**
  - Uses `tree-sitter` for AST-based analysis, providing more accurate results than traditional regex-based scanning.

## 🚀 Quick Start

### Basic Usage

To scan a directory, run the following command:

```bash
mcp-sentinel-clone scan /path/to/your/project
```

### Options

- `--output <format>`: Specify the output format (`terminal`, `json`, or `sarif`).
- `--config <path>`: Provide a path to a custom configuration file.
- `--severity <level>`: Set the minimum severity level to report (`low`, `medium`, `high`, `critical`).
- `--fail-on <level>`: Exit with a non-zero status code if vulnerabilities are found at or above this severity level.
