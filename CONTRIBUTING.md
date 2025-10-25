# Contributing to MCP Sentinel

Thank you for considering contributing to MCP Sentinel! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
- [Adding a New Detector](#adding-a-new-detector)
- [Testing](#testing)
- [Documentation](#documentation)
- [Pull Request Process](#pull-request-process)
- [Community](#community)

## Code of Conduct

Be respectful, inclusive, and professional. We're all here to build better security tools for the AI community.

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Git
- Basic understanding of security vulnerability detection

### Development Setup

1. **Fork and clone the repository:**

```bash
git clone https://github.com/YOUR_USERNAME/mcp-sentinel
cd mcp-sentinel
```

2. **Install dependencies:**

```bash
cargo build
```

3. **Run tests:**

```bash
cargo test
```

4. **Run the scanner:**

```bash
cargo run -- scan ./tests/fixtures/vulnerable_servers/
```

## How to Contribute

### Areas for Contribution

We're actively looking for help with:

1. **New Detectors**
   - PII detection
   - Additional code vulnerability patterns
   - Supply chain security checks
   - Custom MCP-specific attacks

2. **Output Formats**
   - HTML report generation
   - PDF report generation
   - SARIF format for GitHub Security integration

3. **Performance Improvements**
   - Parallel scanning optimizations
   - Memory usage reduction
   - Faster regex patterns

4. **Documentation**
   - More examples
   - Tutorials
   - Detection strategy explanations

5. **Testing**
   - Additional test fixtures
   - Integration tests
   - Performance benchmarks

### Good First Issues

Look for issues labeled `good-first-issue` or `help-wanted` in the GitHub issue tracker.

## Adding a New Detector

Follow these steps to add a new vulnerability detector:

### 1. Create the Detector Module

Create a new file in `src/detectors/`:

```rust
// src/detectors/my_new_detector.rs

use crate::detectors::Detector;
use crate::models::vulnerability::{Vulnerability, VulnerabilityType, Severity};
use crate::error::Result;
use regex::Regex;
use once_cell::sync::Lazy;

/// Detector for [description of vulnerability type]
pub struct MyNewDetector {
    patterns: Vec<DetectionPattern>,
}

struct DetectionPattern {
    name: &'static str,
    regex: Regex,
    severity: Severity,
}

static PATTERNS: Lazy<Vec<(&'static str, &'static str, Severity)>> = Lazy::new(|| {
    vec![
        (
            r"pattern_here",
            "Description of what this detects",
            Severity::High,
        ),
        // Add more patterns...
    ]
});

impl MyNewDetector {
    pub fn new() -> Self {
        let patterns = PATTERNS
            .iter()
            .map(|(pattern, name, severity)| DetectionPattern {
                name,
                regex: Regex::new(pattern).unwrap(),
                severity: *severity,
            })
            .collect();

        Self { patterns }
    }
}

impl Default for MyNewDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for MyNewDetector {
    fn name(&self) -> &'static str {
        "MyNewDetector"
    }

    fn scan(&self, content: &str, file_path: Option<&str>) -> Result<Vec<Vulnerability>> {
        let mut vulnerabilities = Vec::new();
        let mut vuln_counter = 1;

        for pattern in &self.patterns {
            for (line_num, line) in content.lines().enumerate() {
                if let Some(captures) = pattern.regex.captures(line) {
                    let matched_text = captures.get(0).map(|m| m.as_str()).unwrap_or("");

                    let id = format!("MND-{:03}", vuln_counter);
                    vuln_counter += 1;

                    let mut vuln = Vulnerability::new(
                        id,
                        VulnerabilityType::YourType,
                        pattern.severity,
                        format!("Title: {}", pattern.name),
                        format!("Description at line {}", line_num + 1),
                    )
                    .with_impact("Explain the impact here".to_string())
                    .with_remediation("How to fix it".to_string());

                    if let Some(path) = file_path {
                        vuln = vuln.with_location(path.to_string(), Some(line_num + 1), None);
                    }

                    vulnerabilities.push(vuln);
                }
            }
        }

        Ok(vulnerabilities)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_vulnerability() {
        let detector = MyNewDetector::new();
        let content = "test content with pattern";
        let vulns = detector.scan(content, None).unwrap();

        assert!(!vulns.is_empty());
        assert_eq!(vulns[0].severity, Severity::High);
    }

    #[test]
    fn test_no_false_positive() {
        let detector = MyNewDetector::new();
        let content = "clean content";
        let vulns = detector.scan(content, None).unwrap();

        assert!(vulns.is_empty());
    }
}
```

### 2. Register the Detector

Add to `src/detectors/mod.rs`:

```rust
pub mod my_new_detector;
pub use my_new_detector::MyNewDetector;
```

### 3. Add to the Scanning Pipeline

Update `src/cli/scan.rs` to include your detector:

```rust
let detectors: Vec<Box<dyn Detector>> = vec![
    Box::new(ToolPoisoningDetector::new()),
    Box::new(PromptInjectionDetector::new()),
    Box::new(SecretsDetector::new()),
    Box::new(MyNewDetector::new()),  // Add this line
];
```

### 4. Add Tests

Create test fixtures in `tests/fixtures/vulnerable_servers/`:

```
tests/fixtures/vulnerable_servers/my-new-vuln/
├── vulnerable_file.py
├── another_file.js
└── README.md
```

Add integration tests in `tests/integration/`:

```rust
#[tokio::test]
async fn test_my_new_detector_integration() {
    // Test the full scanning pipeline with your detector
}
```

### 5. Document Your Detector

Add documentation to your detector explaining:
- What vulnerability it detects
- How it works (regex patterns, heuristics, etc.)
- Examples of vulnerable code
- False positive considerations

### 6. Performance Considerations

- Use `Lazy` statics for regex compilation
- Minimize allocations in hot paths
- Consider using `rayon` for parallel processing if needed
- Benchmark with `cargo bench`

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run tests in specific module
cargo test detectors::
```

### Adding Tests

Every detector should have:

1. **Unit tests** - Test individual patterns
2. **Integration tests** - Test the full detector
3. **False positive tests** - Ensure clean code doesn't trigger alerts
4. **Edge case tests** - Test boundary conditions

Example test structure:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_basic_case() {
        // Test basic vulnerability detection
    }

    #[test]
    fn test_handles_edge_cases() {
        // Test with empty strings, special characters, etc.
    }

    #[test]
    fn test_no_false_positives() {
        // Test legitimate code that shouldn't trigger
    }

    #[test]
    fn test_multiple_vulnerabilities() {
        // Test detecting multiple issues in one file
    }
}
```

### Test Fixtures

Add vulnerable examples to `tests/fixtures/vulnerable_servers/`:

```
vulnerable_servers/
├── tool-poisoning-server/
│   ├── tools.json
│   └── README.md (explains the vulnerability)
├── secrets-server/
│   ├── config.py (contains fake secrets)
│   └── README.md
└── your-new-vuln-server/
    ├── vulnerable_code.xyz
    └── README.md
```

## Documentation

### Code Documentation

- Add rustdoc comments to all public APIs
- Include examples in doc comments
- Document error conditions
- Explain complex algorithms

Example:

```rust
/// Detects XYZ vulnerabilities using pattern matching.
///
/// This detector searches for [explanation] using regex patterns
/// and heuristic analysis.
///
/// # Examples
///
/// ```
/// use mcp_sentinel::detectors::{MyDetector, Detector};
///
/// let detector = MyDetector::new();
/// let vulns = detector.scan(content, Some("file.py"))?;
/// ```
///
/// # Detection Strategy
///
/// [Explain how the detection works]
///
/// # False Positives
///
/// [Explain known false positive scenarios and how to avoid them]
pub struct MyDetector { ... }
```

### Updating README

If your contribution adds significant features:
- Update the feature list
- Add usage examples
- Update the comparison table if applicable

## Pull Request Process

### 1. Before Submitting

- ✅ All tests pass: `cargo test`
- ✅ Code is formatted: `cargo fmt`
- ✅ No warnings: `cargo clippy`
- ✅ Documentation is updated
- ✅ Commit messages are clear

### 2. Commit Message Format

Use conventional commits:

```
feat: Add PII detection for email addresses
fix: Correct regex pattern for AWS keys
docs: Update installation instructions
test: Add integration tests for secrets detector
perf: Optimize parallel file scanning
```

### 3. Pull Request Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Documentation update
- [ ] Performance improvement
- [ ] Other (please describe)

## Testing
How has this been tested?

## Checklist
- [ ] Tests pass locally
- [ ] Code is formatted (cargo fmt)
- [ ] No clippy warnings
- [ ] Documentation updated
- [ ] CHANGELOG.md updated (if applicable)
```

### 4. Review Process

- Maintainers will review within 3-5 days
- Address feedback in additional commits
- Once approved, squash and merge

## Code Style

### Rust Style Guidelines

Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/):

- Use `cargo fmt` for formatting
- Run `cargo clippy` and fix warnings
- Prefer `Result` over `Option` for errors
- Use builder patterns for complex construction
- Keep functions under 50 lines when possible

### Error Handling

```rust
// ✅ Good: Use custom error types
use crate::error::{Result, ScanError};

fn scan_file(path: &Path) -> Result<Vec<Vulnerability>> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| ScanError::file_read_error(path, e))?;
    // ... scan logic
}

// ❌ Bad: Generic errors
fn scan_file(path: &Path) -> Result<Vec<Vulnerability>, Box<dyn Error>> {
    // ...
}
```

### Naming Conventions

- Types: `PascalCase` (e.g., `SecretsDetector`)
- Functions/variables: `snake_case` (e.g., `scan_content`)
- Constants: `SCREAMING_SNAKE_CASE` (e.g., `MAX_FILE_SIZE`)
- Modules: `snake_case` (e.g., `tool_poisoning`)

## Performance Guidelines

### Benchmarking

Add benchmarks for performance-critical code:

```rust
// benches/my_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_detector(c: &mut Criterion) {
    let detector = MyDetector::new();
    let content = "test content";

    c.bench_function("my_detector", |b| {
        b.iter(|| {
            detector.scan(black_box(content), None).unwrap()
        });
    });
}

criterion_group!(benches, benchmark_detector);
criterion_main!(benches);
```

Run with:

```bash
cargo bench
```

### Performance Targets

- Scan time: <5s for 1000 files
- Memory usage: <100MB for typical scans
- Binary size: <30MB

## Community

### Getting Help

- **GitHub Issues**: For bugs and feature requests
- **GitHub Discussions**: For questions and general discussion
- **Discord** (coming soon): For real-time chat

### Recognition

Contributors will be:
- Listed in CONTRIBUTORS.md
- Mentioned in release notes
- Tagged in social media announcements (if desired)

## License

By contributing, you agree that your contributions will be licensed under the Apache License 2.0.

---

Thank you for making MCP Sentinel better! 🛡️
