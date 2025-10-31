use crate::models::scan_result::ScanResult;
use crate::models::vulnerability::Vulnerability;
use serde_sarif::sarif::{Sarif, Run, Tool, ToolComponent, ReportingDescriptor, Result as SarifResult, Location, PhysicalLocation, ArtifactLocation, Message};

pub fn display_scan_result(result: &ScanResult) {
    let tool = Tool::builder()
        .driver(ToolComponent::builder()
            .name("MCP Sentinel Clone")
            .build())
        .build();

    let results: Vec<SarifResult> = result.vulnerabilities.iter().map(convert_vuln_to_sarif_result).collect();

    let run = Run::builder()
        .tool(tool)
        .results(results)
        .build();

    let sarif = Sarif::builder()
        .version("2.1.0")
        .runs(vec![run])
        .build();

    match serde_json::to_string_pretty(&sarif) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error serializing to SARIF: {}", e),
    }
}

fn convert_vuln_to_sarif_result(vuln: &Vulnerability) -> SarifResult {
    SarifResult::builder()
        .rule_id(vuln.id.clone())
        .message(Message::builder().text(vuln.description.clone()).build())
        .locations(vec![Location::builder()
            .physical_location(PhysicalLocation::builder()
                .artifact_location(ArtifactLocation::builder()
                    .uri(vuln.location.file_path.clone())
                    .build())
                .build())
            .build()])
        .build()
}
