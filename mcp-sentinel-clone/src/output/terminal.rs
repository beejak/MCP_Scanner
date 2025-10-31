use crate::models::scan_result::ScanResult;
use crate::models::vulnerability::Severity;
use colored::*;

pub fn display_scan_result(result: &ScanResult) {
    println!("Scan Results for: {}", result.target.cyan());
    println!("------------------------------------");
    println!(
        "Total Vulnerabilities: {}",
        result.vulnerabilities.len().to_string().yellow()
    );
    println!();

    for vuln in &result.vulnerabilities {
        let severity_str = match vuln.severity {
            Severity::Critical => "CRITICAL".red(),
            Severity::High => "HIGH".magenta(),
            Severity::Medium => "MEDIUM".yellow(),
            Severity::Low => "LOW".blue(),
        };

        println!("[{}] {}", severity_str, vuln.title.bold());
        println!("  Description: {}", vuln.description);
        println!(
            "  Location: {}:{}",
            vuln.location.file_path.underline(),
            vuln.location.line
        );
        println!("  Snippet: {}", vuln.code_snippet.italic());
        println!();
    }
}
