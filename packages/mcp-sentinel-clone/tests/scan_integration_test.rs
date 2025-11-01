use mcp_sentinel_clone::engine::scanner::Scanner;
use mcp_sentinel_clone::models::config::ScanConfig;
use anyhow::Result;

#[tokio::test]
async fn test_scan_fixtures() -> Result<()> {
    let config = ScanConfig::default();
    let scanner = Scanner::new(config);
    let result = scanner.scan_directory("tests/fixtures").await?;

    // Check that we found vulnerabilities of different types
    assert!(result.vulnerabilities.iter().any(|v| v.vulnerability_type == mcp_sentinel_clone::models::vulnerability::VulnerabilityType::SecretsLeakage));
    assert!(result.vulnerabilities.iter().any(|v| v.vulnerability_type == mcp_sentinel_clone::models::vulnerability::VulnerabilityType::CommandInjection));
    assert!(result.vulnerabilities.iter().any(|v| v.vulnerability_type == mcp_sentinel_clone::models::vulnerability::VulnerabilityType::SensitiveFileAccess));
    assert!(result.vulnerabilities.iter().any(|v| v.vulnerability_type == mcp_sentinel_clone::models::vulnerability::VulnerabilityType::PromptInjection));
    assert!(result.vulnerabilities.iter().any(|v| v.vulnerability_type == mcp_sentinel_clone::models::vulnerability::VulnerabilityType::ToolPoisoning));

    Ok(())
}

#[tokio::test]
async fn test_scan_empty_file() -> Result<()> {
    let config = ScanConfig::default();
    let scanner = Scanner::new(config);
    let result = scanner.scan_directory("tests/fixtures/empty.txt").await?;
    assert_eq!(result.vulnerabilities.len(), 0);
    Ok(())
}
