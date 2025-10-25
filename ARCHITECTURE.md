# MCP Sentinel Architecture

This document describes the architecture, design decisions, and implementation details of MCP Sentinel.

## Table of Contents

- [Overview](#overview)
- [System Architecture](#system-architecture)
- [Module Structure](#module-structure)
- [Detection Engines](#detection-engines)
- [Data Flow](#data-flow)
- [Design Decisions](#design-decisions)
- [Performance Considerations](#performance-considerations)
- [Future Architecture](#future-architecture)

## Overview

MCP Sentinel is a security scanner designed to detect vulnerabilities in MCP (Model Context Protocol) servers. It's built in Rust for performance, safety, and portability.

### Goals

1. **Performance**: 10-100x faster than Python/TypeScript alternatives
2. **Comprehensive**: Detect 13+ vulnerability categories
3. **Extensible**: Easy to add new detectors and output formats
4. **Portable**: Single binary with no runtime dependencies
5. **Reliable**: Type-safe error handling and comprehensive testing

### Non-Goals

- Real-time traffic analysis (Phase 3 feature)
- Automated remediation
- Integration with specific IDEs (community can build these)

## System Architecture

### High-Level Overview

```
┌─────────────────────────────────────────────────────────────┐
│                         CLI Layer                            │
│  (main.rs, cli/*.rs)                                        │
│  - Argument parsing (clap)                                  │
│  - Command routing                                          │
│  - User interaction                                         │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│                     Scanning Coordinator                     │
│  (cli/scan.rs)                                              │
│  - File discovery                                           │
│  - Detector orchestration                                   │
│  - Result aggregation                                       │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│                      Detection Layer                         │
│  (detectors/*.rs)                                           │
│  ┌───────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │Tool Poisoning │  │    Secrets   │  │Prompt Inject.│   │
│  │   Detector    │  │   Detector   │  │  Detector    │   │
│  └───────────────┘  └──────────────┘  └──────────────┘   │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│                      Models Layer                            │
│  (models/*.rs)                                              │
│  - Vulnerability                                            │
│  - ScanResult                                               │
│  - Config                                                   │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│                      Output Layer                            │
│  (output/*.rs)                                              │
│  ┌─────────┐  ┌──────────┐  ┌──────┐  ┌──────┐           │
│  │Terminal │  │   JSON   │  │ HTML │  │ PDF  │           │
│  │Formatter│  │Formatter │  │Format│  │Format│           │
│  └─────────┘  └──────────┘  └──────┘  └──────┘           │
└─────────────────────────────────────────────────────────────┘
```

### Three-Engine Design (Future)

MCP Sentinel is designed around three detection engines:

1. **Static Analysis Engine** (Phase 1 - Current)
   - Pattern-based detection
   - Regex matching
   - Heuristic analysis

2. **AI Analysis Engine** (Phase 2 - Q2 2025)
   - LLM-powered context analysis
   - Natural language vulnerability explanations
   - Risk scoring

3. **Runtime Proxy Engine** (Phase 3 - Q3 2025)
   - Real-time MCP traffic interception
   - Guardrails enforcement
   - Behavioral anomaly detection

## Module Structure

### Project Layout

```
src/
├── main.rs                    # CLI entry point
├── lib.rs                     # Library exports
├── error.rs                   # Custom error types
│
├── cli/                       # Command-line interface
│   ├── mod.rs                # CLI module exports
│   ├── scan.rs               # Scan command (primary)
│   ├── proxy.rs              # Proxy command (Phase 3)
│   ├── monitor.rs            # Monitor command (Phase 3)
│   ├── audit.rs              # Audit command (Phase 2)
│   ├── init.rs               # Init command
│   ├── whitelist.rs          # Whitelist management
│   └── rules.rs              # Rules management
│
├── detectors/                 # Vulnerability detectors
│   ├── mod.rs                # Detector trait & exports
│   ├── tool_poisoning.rs     # Tool poisoning detection
│   ├── prompt_injection.rs   # Prompt injection detection
│   ├── secrets.rs            # Secrets/credentials detection
│   ├── code_vulns.rs         # Code vulnerability detection
│   ├── pii.rs                # PII detection (stub)
│   ├── toxic_flows.rs        # Toxic flow detection (stub)
│   └── anomalies.rs          # Anomaly detection (stub)
│
├── engines/                   # Detection engines
│   ├── static_analysis/      # Static analysis engine
│   │   ├── mod.rs
│   │   ├── semgrep.rs        # Semgrep integration (Phase 2)
│   │   ├── tree_sitter.rs    # Code parsing (Phase 2)
│   │   ├── taint.rs          # Taint analysis (Phase 2)
│   │   └── patterns.rs       # Pattern library (Phase 2)
│   ├── runtime_proxy/        # Runtime proxy engine (Phase 3)
│   │   ├── mod.rs
│   │   ├── server.rs         # Proxy server
│   │   ├── interceptor.rs    # Request/response interception
│   │   ├── guardrails.rs     # Policy enforcement
│   │   └── monitor.rs        # Real-time monitoring
│   └── ai_analysis/          # AI analysis engine (Phase 2)
│       ├── mod.rs
│       ├── openai.rs         # OpenAI integration
│       ├── anthropic.rs      # Anthropic integration
│       ├── ollama.rs         # Local LLM integration
│       ├── prompts.rs        # Analysis prompts
│       └── scorer.rs         # Risk scoring
│
├── models/                    # Data models
│   ├── mod.rs
│   ├── vulnerability.rs      # Vulnerability data structure
│   ├── scan_result.rs        # Scan result container
│   ├── mcp_protocol.rs       # MCP protocol types
│   └── config.rs             # Configuration
│
├── output/                    # Output formatters
│   ├── mod.rs
│   ├── terminal.rs           # Terminal (colored) output
│   ├── json.rs               # JSON output
│   ├── html.rs               # HTML report (Phase 2)
│   ├── pdf.rs                # PDF report (Phase 2)
│   └── sarif.rs              # SARIF format (Phase 2)
│
├── storage/                   # Persistent storage
│   ├── mod.rs
│   ├── whitelist.rs          # Whitelist database
│   ├── state.rs              # Scanner state
│   └── cache.rs              # Result caching
│
└── utils/                     # Utilities
    ├── mod.rs
    ├── file.rs               # File operations
    ├── crypto.rs             # Hashing/crypto
    ├── git.rs                # Git operations
    └── http.rs               # HTTP utilities
```

### Module Responsibilities

#### CLI Layer (`cli/`)

**Purpose**: Handle user interaction and command routing

**Key Components**:
- `scan.rs`: Coordinates file discovery, detector execution, and result output
- Command parsers for each subcommand
- Error handling and user feedback

**Design Pattern**: Command pattern with async execution

#### Detectors (`detectors/`)

**Purpose**: Identify specific vulnerability types

**Key Components**:
- `Detector` trait: Common interface for all detectors
- Individual detector implementations
- Pattern libraries using `once_cell::Lazy` for regex compilation

**Design Pattern**: Strategy pattern with trait objects

**Example**:
```rust
pub trait Detector: Send + Sync {
    fn name(&self) -> &'static str;
    fn scan(&self, content: &str, file_path: Option<&str>) -> Result<Vec<Vulnerability>>;
}
```

#### Models (`models/`)

**Purpose**: Define core data structures

**Key Types**:
- `Vulnerability`: Represents a single security issue
- `ScanResult`: Container for all vulnerabilities from a scan
- `Config`: Scanner configuration

**Design Pattern**: Builder pattern for complex construction

**Example**:
```rust
let vuln = Vulnerability::new(id, type, severity, title, description)
    .with_location(file, line, col)
    .with_impact(impact)
    .with_remediation(fix);
```

#### Output (`output/`)

**Purpose**: Format and present scan results

**Key Components**:
- `OutputFormatter` trait: Common interface
- Format-specific implementations
- Template-based rendering (handlebars for HTML)

**Design Pattern**: Strategy pattern

#### Utils (`utils/`)

**Purpose**: Shared utility functions

**Key Components**:
- File discovery with `ignore` crate (.gitignore support)
- Cryptographic hashing (SHA-256)
- HTTP/Git helpers

## Detection Engines

### Current Implementation: Pattern-Based Detection

#### Detector Workflow

```
1. File Discovery
   ├─ Walk directory tree (respecting .gitignore)
   ├─ Filter by file size (<10MB default)
   └─ Skip binary files

2. Content Extraction
   ├─ Read file contents (UTF-8)
   ├─ Handle encoding errors gracefully
   └─ Store file path for reporting

3. Pattern Matching
   ├─ Compile regex patterns (once, using Lazy)
   ├─ Scan line-by-line
   └─ Capture matches with context

4. Vulnerability Construction
   ├─ Create Vulnerability struct
   ├─ Add location information
   ├─ Add evidence (code snippet)
   └─ Assign severity & confidence

5. Result Aggregation
   ├─ Collect from all detectors
   ├─ Calculate summary statistics
   └─ Generate ScanResult
```

#### Pattern Compilation

Using `once_cell::Lazy` for one-time regex compilation:

```rust
use once_cell::sync::Lazy;

static PATTERNS: Lazy<Vec<(&'static str, &'static str, Severity)>> = Lazy::new(|| {
    vec![
        (r"pattern1", "Description", Severity::High),
        (r"pattern2", "Description", Severity::Critical),
    ]
});

impl Detector {
    pub fn new() -> Self {
        let patterns = PATTERNS
            .iter()
            .map(|(pat, name, sev)| CompiledPattern {
                regex: Regex::new(pat).unwrap(),
                name,
                severity: *sev,
            })
            .collect();
        Self { patterns }
    }
}
```

**Why this works**:
- Regex compilation happens once per process
- Thread-safe sharing via `Lazy`
- No runtime overhead for pattern compilation

### Detector Implementation Patterns

#### 1. Simple Pattern Matching

For straightforward detections (e.g., finding hardcoded secrets):

```rust
impl Detector for SecretsDetector {
    fn scan(&self, content: &str, file_path: Option<&str>) -> Result<Vec<Vulnerability>> {
        let mut vulnerabilities = Vec::new();

        for pattern in &self.patterns {
            for (line_num, line) in content.lines().enumerate() {
                if let Some(captures) = pattern.regex.captures(line) {
                    // Create vulnerability...
                    vulnerabilities.push(vuln);
                }
            }
        }

        Ok(vulnerabilities)
    }
}
```

#### 2. Heuristic Analysis

For complex detections requiring context (e.g., entropy analysis for secrets):

```rust
fn calculate_entropy(s: &str) -> f64 {
    let mut char_counts = HashMap::new();
    for c in s.chars() {
        *char_counts.entry(c).or_insert(0) += 1;
    }

    let len = s.len() as f64;
    let mut entropy = 0.0;

    for &count in char_counts.values() {
        let probability = count as f64 / len;
        entropy -= probability * probability.log2();
    }

    entropy
}

// High entropy strings (>4.0) are likely secrets
if calculate_entropy(matched_text) > 4.0 {
    vulnerabilities.push(create_vuln());
}
```

#### 3. Semantic Analysis (Future)

For understanding code structure:

```rust
// Phase 2: Using tree-sitter
let mut parser = Parser::new();
parser.set_language(tree_sitter_python::language())?;

let tree = parser.parse(content, None)?;
let root_node = tree.root_node();

// Analyze AST for data flow vulnerabilities
analyze_taint_flow(root_node)?;
```

## Data Flow

### Scan Command Flow

```
User Input (CLI)
    │
    ▼
┌──────────────────┐
│ Parse Arguments  │  (clap)
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Load Config      │  (YAML/TOML)
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Discover Files   │  (walkdir + ignore)
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Initialize       │  (Create detector instances)
│ Detectors        │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Parallel Scan    │  (tokio::spawn for each file)
│   For Each File: │
│   ├─ Read Content│
│   ├─ Run Detector│
│   └─ Collect     │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Aggregate        │  (Combine results)
│ Results          │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Generate         │  (ScanResult with Summary)
│ Summary          │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Format Output    │  (Terminal, JSON, etc.)
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Return Exit Code │  (0=success, 1=vulns found, 2=error)
└──────────────────┘
```

### Parallel Scanning Implementation

```rust
use tokio::task;
use futures::stream::{StreamExt, FuturesUnordered};

pub async fn scan_files_parallel(files: Vec<DiscoveredFile>) -> Result<Vec<Vulnerability>> {
    let mut tasks = FuturesUnordered::new();

    for file in files {
        let task = task::spawn(async move {
            scan_single_file(file).await
        });
        tasks.push(task);
    }

    let mut all_vulnerabilities = Vec::new();
    while let Some(result) = tasks.next().await {
        all_vulnerabilities.extend(result??);
    }

    Ok(all_vulnerabilities)
}
```

## Design Decisions

### 1. Why Rust?

**Decision**: Use Rust as the primary language

**Rationale**:
- **Performance**: 10-100x faster than Python for I/O-bound workloads
- **Safety**: Memory safety without garbage collection
- **Portability**: Single binary, no runtime dependencies
- **Concurrency**: Excellent async/await support with Tokio
- **Type Safety**: Catch errors at compile time

**Trade-offs**:
- Steeper learning curve for contributors
- Longer compile times than interpreted languages
- Smaller ecosystem than Python for security tools

### 2. Pattern-Based Detection (Phase 1)

**Decision**: Start with regex-based pattern matching

**Rationale**:
- Fast implementation and iteration
- Good enough for 80% of vulnerability types
- Deterministic and testable
- No external dependencies (LLM APIs)

**Trade-offs**:
- Limited context understanding
- Potential for false positives
- Can't detect complex logic vulnerabilities

**Mitigation**: Phase 2 adds AI-powered analysis for context

### 3. Async/Parallel Architecture

**Decision**: Use Tokio for async I/O and parallel scanning

**Rationale**:
- I/O-bound workload benefits from async
- Parallel file scanning improves performance
- Non-blocking operations

**Implementation**:
```rust
#[tokio::main]
async fn main() {
    // Async entry point
}

// Spawn tasks for parallel scanning
for file in files {
    tokio::spawn(async move {
        scan_file(file).await
    });
}
```

### 4. Custom Error Types

**Decision**: Use `thiserror` for structured errors, not just `anyhow`

**Rationale**:
- Library users need to handle specific error types
- Better error messages with context
- Distinguishes recoverable from non-recoverable errors

**Before** (Phase 1 initial):
```rust
use anyhow::Result;  // Too generic
```

**After** (Phase 1.5):
```rust
use crate::error::{Result, ScanError};  // Structured errors

pub enum ScanError {
    FileNotFound { path: PathBuf },
    DetectorError { detector: String, reason: String },
    // ... specific error types
}
```

### 5. Builder Pattern for Vulnerabilities

**Decision**: Use builder pattern for `Vulnerability` construction

**Rationale**:
- Many optional fields (CWE, CVSS, evidence, AI analysis)
- Readable construction code
- Fluent API for chaining

**Example**:
```rust
Vulnerability::new(id, type, severity, title, description)
    .with_location(file, line, col)
    .with_impact(impact)
    .with_remediation(fix)
    .with_cwe(798)
```

### 6. Single Binary Distribution

**Decision**: Distribute as a single static binary

**Rationale**:
- Easy installation (no Python/Node.js required)
- Fast startup time
- Consistent behavior across environments

**Implementation**: Use `cargo build --release` with:
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

Result: ~20MB binary with all features

## Performance Considerations

### Optimization Strategies

#### 1. Lazy Regex Compilation

**Problem**: Compiling regexes on every scan is expensive

**Solution**: Use `once_cell::Lazy` for one-time compilation

```rust
static PATTERNS: Lazy<Vec<...>> = Lazy::new(|| { ... });
```

**Impact**: ~50ms saved per scan

#### 2. Parallel File Scanning

**Problem**: Sequential file scanning is slow for large codebases

**Solution**: Use Tokio to spawn tasks for each file

```rust
tokio::spawn(async move {
    scan_file(file).await
})
```

**Impact**: ~10x faster for directories with 100+ files

#### 3. Efficient File Walking

**Problem**: Walking large directories with .gitignore support is complex

**Solution**: Use `ignore` crate (from ripgrep)

```rust
use ignore::WalkBuilder;

let walker = WalkBuilder::new(path)
    .git_ignore(true)
    .build();
```

**Impact**: Automatically skips `node_modules`, `.git`, etc.

#### 4. Bounded Memory Usage

**Problem**: Loading large files into memory can cause OOM

**Solution**: Skip files >10MB, stream large files in future

```rust
if file_size > config.max_file_size {
    warn!("Skipping large file: {}", path);
    continue;
}
```

#### 5. Zero-Copy Where Possible

**Problem**: String allocations are expensive in hot paths

**Solution**: Use `&str` slices instead of `String` when possible

```rust
// Good: Uses slice
fn scan_line(line: &str) -> bool { ... }

// Bad: Allocates new String
fn scan_line(line: String) -> bool { ... }
```

### Performance Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Small repo (<100 files) | <2s | ~1.5s | ✅ |
| Medium repo (100-1000 files) | <10s | ~8s | ✅ |
| Large repo (>1000 files) | <30s | TBD | 🔄 |
| Memory usage | <100MB | ~50MB | ✅ |
| Binary size | <20MB | ~18MB | ✅ |

## Future Architecture

### Phase 2: AI Analysis Engine (Q2 2025)

```
┌────────────────────────────────────┐
│     AI Analysis Engine             │
│                                    │
│  ┌──────────┐   ┌──────────┐     │
│  │ OpenAI   │   │Anthropic │     │
│  │  Client  │   │  Client  │     │
│  └────┬─────┘   └────┬─────┘     │
│       │              │            │
│       └──────┬───────┘            │
│              ▼                    │
│       ┌──────────────┐            │
│       │ Prompt       │            │
│       │ Builder      │            │
│       └──────┬───────┘            │
│              ▼                    │
│       ┌──────────────┐            │
│       │ Risk Scorer  │            │
│       └──────────────┘            │
└────────────────────────────────────┘
```

### Phase 3: Runtime Proxy Engine (Q3 2025)

```
┌─────────────────────────────────────────────┐
│        Runtime Proxy Engine                 │
│                                             │
│  ┌──────────────────────────────────────┐  │
│  │       MCP Proxy Server (Axum)        │  │
│  │  ┌──────────┐     ┌──────────┐      │  │
│  │  │ Request  │────▶│ Response │      │  │
│  │  │Interceptor│◀────│Interceptor│     │  │
│  │  └────┬─────┘     └────┬─────┘      │  │
│  │       │                 │            │  │
│  │       ▼                 ▼            │  │
│  │  ┌──────────────────────────┐       │  │
│  │  │   Guardrails Engine      │       │  │
│  │  │  - PII Detection         │       │  │
│  │  │  - Secrets Blocking      │       │  │
│  │  │  - Toxic Flow Detection  │       │  │
│  │  └──────────┬───────────────┘       │  │
│  │             ▼                        │  │
│  │  ┌──────────────────────────┐       │  │
│  │  │  Real-time Monitor       │       │  │
│  │  │  - Dashboard             │       │  │
│  │  │  - Alerting              │       │  │
│  │  └──────────────────────────┘       │  │
│  └──────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
```

---

## Summary

MCP Sentinel's architecture prioritizes:

1. **Performance**: Rust + async/parallel scanning
2. **Extensibility**: Trait-based detector system
3. **Reliability**: Custom error types + comprehensive testing
4. **Portability**: Single binary distribution
5. **Future-Proof**: Three-engine design for phases 2-3

The modular design allows independent development of detection engines while maintaining a consistent user interface.

For questions or suggestions about the architecture, please open a GitHub Discussion.
