use crate::models::scan_result::ScanResult;
use crate::models::vulnerability::{Severity, Vulnerability};
use crate::output::OutputFormatter;
use anyhow::Result;
use comfy_table::{Table, Cell, Color, Attribute, ContentArrangement};
use crossterm::style::{Stylize, Color as CTColor};
use std::sync::atomic::{AtomicBool, Ordering};

static COLORS_ENABLED: AtomicBool = AtomicBool::new(true);

/// Disable colored output
pub fn disable_colors() {
    COLORS_ENABLED.store(false, Ordering::Relaxed);
}

/// Check if colors are enabled
pub fn colors_enabled() -> bool {
    COLORS_ENABLED.load(Ordering::Relaxed)
}

/// Terminal output formatter with colored output
pub struct TerminalFormatter {
    verbose: bool,
}

impl TerminalFormatter {
    pub fn new() -> Self {
        Self { verbose: false }
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Format header
    fn format_header(&self) -> String {
        let mut output = String::new();

        if colors_enabled() {
            output.push_str(&format!("{}\n\n", "🛡️  MCP Sentinel".bold().with(CTColor::Cyan)));
        } else {
            output.push_str("MCP Sentinel\n\n");
        }

        output
    }

    /// Format summary section
    fn format_summary(&self, result: &ScanResult) -> String {
        let mut output = String::new();

        output.push_str(&self.format_section_title("SCAN RESULTS"));
        output.push('\n');

        // Risk score with color
        let risk_level = result.summary.risk_level();
        let risk_display = if colors_enabled() {
            let color = match result.summary.risk_score {
                0..=20 => CTColor::Blue,
                21..=40 => CTColor::Yellow,
                41..=70 => CTColor::Rgb { r: 255, g: 165, b: 0 }, // Orange
                _ => CTColor::Red,
            };
            format!("{}/100 {}", result.summary.risk_score, risk_level)
                .with(color)
                .bold()
                .to_string()
        } else {
            format!("{}/100 {}", result.summary.risk_score, risk_level)
        };

        output.push_str(&format!("Risk Score: {}\n\n", risk_display));

        // Issue counts
        output.push_str(&self.format_severity_count("CRITICAL", result.summary.critical, Severity::Critical));
        output.push_str(&self.format_severity_count("HIGH", result.summary.high, Severity::High));
        output.push_str(&self.format_severity_count("MEDIUM", result.summary.medium, Severity::Medium));
        output.push_str(&self.format_severity_count("LOW", result.summary.low, Severity::Low));

        output.push('\n');
        output
    }

    /// Format severity count line
    fn format_severity_count(&self, label: &str, count: usize, severity: Severity) -> String {
        if colors_enabled() {
            let color = severity_color(severity);
            format!(
                "{} {} Issues: {}\n",
                severity.badge(),
                label.with(color).bold(),
                count
            )
        } else {
            format!("{} Issues: {}\n", label, count)
        }
    }

    /// Format section title
    fn format_section_title(&self, title: &str) -> String {
        let separator = "━".repeat(60);

        if colors_enabled() {
            format!(
                "{}\n{}\n{}",
                separator.with(CTColor::DarkGrey),
                title.bold(),
                separator.with(CTColor::DarkGrey)
            )
        } else {
            format!("{}\n{}\n{}", separator, title, separator)
        }
    }

    /// Format vulnerabilities grouped by severity
    fn format_vulnerabilities(&self, result: &ScanResult) -> String {
        let mut output = String::new();

        let grouped = result.group_by_severity();

        // Show critical issues first
        if let Some(vulns) = grouped.get(&Severity::Critical) {
            if !vulns.is_empty() {
                output.push_str(&self.format_section_title("CRITICAL ISSUES"));
                output.push('\n');
                for vuln in vulns {
                    output.push_str(&self.format_vulnerability(vuln));
                    output.push('\n');
                }
            }
        }

        // Show high issues
        if let Some(vulns) = grouped.get(&Severity::High) {
            if !vulns.is_empty() {
                output.push_str(&self.format_section_title("HIGH ISSUES"));
                output.push('\n');
                for vuln in vulns {
                    output.push_str(&self.format_vulnerability(vuln));
                    output.push('\n');
                }
            }
        }

        // Only show medium/low if verbose or few issues
        if self.verbose || result.summary.total_issues <= 20 {
            if let Some(vulns) = grouped.get(&Severity::Medium) {
                if !vulns.is_empty() {
                    output.push_str(&self.format_section_title("MEDIUM ISSUES"));
                    output.push('\n');
                    for vuln in vulns {
                        output.push_str(&self.format_vulnerability(vuln));
                        output.push('\n');
                    }
                }
            }

            if let Some(vulns) = grouped.get(&Severity::Low) {
                if !vulns.is_empty() {
                    output.push_str(&self.format_section_title("LOW ISSUES"));
                    output.push('\n');
                    for vuln in vulns {
                        output.push_str(&self.format_vulnerability(vuln));
                        output.push('\n');
                    }
                }
            }
        }

        output
    }

    /// Format a single vulnerability
    fn format_vulnerability(&self, vuln: &Vulnerability) -> String {
        let mut output = String::new();

        // ID and title
        if colors_enabled() {
            let color = severity_color(vuln.severity);
            output.push_str(&format!(
                "[{}] {}\n",
                vuln.id.with(color).bold(),
                vuln.title.bold()
            ));
        } else {
            output.push_str(&format!("[{}] {}\n", vuln.id, vuln.title));
        }

        // Location
        if let Some(ref location) = vuln.location {
            output.push_str(&format!("  Location: {}\n", location));
        }

        // Description
        output.push_str(&format!("  {}\n", vuln.description));

        // Impact (if present and verbose or critical/high)
        if !vuln.impact.is_empty() && (self.verbose || vuln.severity >= Severity::High) {
            if colors_enabled() {
                output.push_str(&format!("  {} {}\n", "⚠️  Impact:".yellow(), vuln.impact));
            } else {
                output.push_str(&format!("  Impact: {}\n", vuln.impact));
            }
        }

        // Remediation (if present and verbose or critical/high)
        if !vuln.remediation.is_empty() && (self.verbose || vuln.severity >= Severity::High) {
            if colors_enabled() {
                output.push_str(&format!("  {} {}\n", "🔧 Remediation:".green(), vuln.remediation));
            } else {
                output.push_str(&format!("  Remediation: {}\n", vuln.remediation));
            }
        }

        // AI Analysis (if present and verbose)
        if let Some(ref ai_analysis) = vuln.ai_analysis {
            if self.verbose {
                output.push_str(&format!("  AI Analysis:\n    {}\n", ai_analysis.explanation));
            }
        }

        output
    }

    /// Format footer
    fn format_footer(&self, result: &ScanResult) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "⏱️  Scan completed in {:.1}s\n",
            result.metadata.scan_duration_ms as f64 / 1000.0
        ));

        output.push_str(&format!("📊 Scanned: {}\n", result.target));
        output.push_str(&format!("🔍 Engines: {}\n", result.engines.join(", ")));

        output
    }
}

impl Default for TerminalFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter for TerminalFormatter {
    fn output(&self, result: &ScanResult) -> Result<String> {
        let mut output = String::new();

        output.push_str(&self.format_header());
        output.push_str(&format!("📂 Scanning: {}\n", result.target));
        output.push_str(&format!("🔍 Engines: {}\n\n", result.engines.join(" | ")));

        output.push_str(&self.format_summary(result));

        if result.summary.total_issues > 0 {
            output.push_str(&self.format_vulnerabilities(result));
        } else {
            output.push_str("✅ No vulnerabilities found!\n\n");
        }

        output.push_str(&self.format_footer(result));

        Ok(output)
    }
}

/// Get color for severity
fn severity_color(severity: Severity) -> CTColor {
    match severity {
        Severity::Critical => CTColor::Red,
        Severity::High => CTColor::Rgb { r: 255, g: 165, b: 0 }, // Orange
        Severity::Medium => CTColor::Yellow,
        Severity::Low => CTColor::Blue,
    }
}

/// Create a summary table
pub fn create_summary_table(result: &ScanResult) -> Table {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);

    // Header
    table.set_header(vec![
        Cell::new("Severity").fg(Color::Cyan).add_attribute(Attribute::Bold),
        Cell::new("Count").fg(Color::Cyan).add_attribute(Attribute::Bold),
    ]);

    // Rows
    table.add_row(vec![
        Cell::new("Critical").fg(Color::Red),
        Cell::new(result.summary.critical),
    ]);
    table.add_row(vec![
        Cell::new("High").fg(Color::Yellow),
        Cell::new(result.summary.high),
    ]);
    table.add_row(vec![
        Cell::new("Medium").fg(Color::Yellow),
        Cell::new(result.summary.medium),
    ]);
    table.add_row(vec![
        Cell::new("Low").fg(Color::Blue),
        Cell::new(result.summary.low),
    ]);

    table
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::vulnerability::VulnerabilityType;
    use crate::models::scan_result::Metadata;
    use chrono::Utc;

    #[test]
    fn test_terminal_formatter() {
        let vuln = Vulnerability::new(
            "TEST-001".to_string(),
            VulnerabilityType::ToolPoisoning,
            Severity::Critical,
            "Test Vulnerability".to_string(),
            "A test vulnerability".to_string(),
        );

        let result = ScanResult::new(
            "./test".to_string(),
            vec!["static".to_string()],
            vec![vuln],
            Metadata {
                scan_duration_ms: 1500,
                engines_used: vec!["static".to_string()],
                llm_provider: None,
                llm_model: None,
            },
        );

        let formatter = TerminalFormatter::new();
        let output = formatter.output(&result).unwrap();

        assert!(output.contains("MCP Sentinel"));
        assert!(output.contains("CRITICAL"));
        assert!(output.contains("TEST-001"));
    }
}
