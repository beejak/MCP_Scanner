use crate::models::scan_result::ScanResult;

pub fn display_scan_result(result: &ScanResult) {
    println!("Scan Results for: {}", result.target);
    println!("------------------------------------");
    println!("Total Vulnerabilities: {}", result.vulnerabilities.len());
    println!();

    for vuln in &result.vulnerabilities {
        println!("[{}] {}", vuln.severity, vuln.title);
        println!("  Description: {}", vuln.description);
        println!("  Location: {}:{}", vuln.location.file_path, vuln.location.line);
        println!("  Snippet: {}", vuln.code_snippet);
        println!();
    }
}
