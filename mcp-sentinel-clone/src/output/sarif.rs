use crate::models::scan_result::ScanResult;
use crate::models::vulnerability::Vulnerability;
use serde_sarif::sarif::{Sarif, Run, Tool, ToolComponent, Result as SarifResult, Location, PhysicalLocation, ArtifactLocation, Message};
use serde_json::Value;

pub fn display_scan_result(result: &ScanResult) {
    let tool = Tool {
        driver: ToolComponent {
            name: "MCP Sentinel Clone".to_string(),
            guid: None,
            release_date_utc: None,
            organization: None,
            product: None,
            product_suite: None,
            short_description: None,
            full_description: None,
            full_name: None,
            version: None,
            semantic_version: None,
            dotted_quad_file_version: None,
            download_uri: None,
            information_uri: None,
            global_message_strings: None,
            notifications: None,
            rules: None,
            taxa: None,
            locations: None,
            language: None,
            contents: None,
            is_comprehensive: None,
            localized_data_semantic_version: None,
            minimum_required_localized_data_semantic_version: None,
            associated_component: None,
            translation_metadata: None,
            supported_taxonomies: None,
            properties: None,
        },
        extensions: None,
        properties: None,
    };

    let results: Vec<SarifResult> = result.vulnerabilities.iter().map(convert_vuln_to_sarif_result).collect();

    let run = Run {
        tool,
        results: Some(results),
        artifacts: None,
        default_source_language: None,
        run_aggregates: None,
        automation_details: None,
        baseline_guid: None,
        default_encoding: None,
        invocations: None,
        conversion: None,
        language: None,
        version_control_provenance: None,
        original_uri_base_ids: None,
        redaction_tokens: None,
        newline_sequences: None,
        column_kind: None,
        external_property_file_references: None,
        thread_flow_locations: None,
        taxonomies: None,
        addresses: None,
        translations: None,
        policies: None,
        web_requests: None,
        web_responses: None,
        special_locations: None,
        logical_locations: None,
        graphs: None,
        properties: None,
    };

    let sarif = Sarif {
        version: Value::String("2.1.0".to_string()),
        schema: Some("https://schemastore.azurewebsites.net/schemas/json/sarif-2.1.0-rtm.5.json".to_string()),
        runs: vec![run],
        inline_external_properties: None,
        properties: None,
    };

    match serde_json::to_string_pretty(&sarif) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error serializing to SARIF: {}", e),
    }
}

fn convert_vuln_to_sarif_result(vuln: &Vulnerability) -> SarifResult {
    SarifResult {
        rule_id: Some(vuln.id.clone()),
        message: Message {
            text: Some(vuln.description.clone()),
            id: None,
            markdown: None,
            arguments: None,
            properties: None,
        },
        locations: Some(vec![Location {
            physical_location: Some(PhysicalLocation {
                artifact_location: Some(ArtifactLocation {
                    uri: Some(vuln.location.file_path.clone()),
                    uri_base_id: None,
                    index: None,
                    description: None,
                    properties: None,
                }),
                region: None,
                context_region: None,
                address: None,
                properties: None,
            }),
            logical_locations: None,
            message: None,
            annotations: None,
            relationships: None,
            id: None,
            properties: None,
        }]),
        attachments: None,
        guid: None,
        occurrence_count: None,
        rank: None,
        rule: None,
        rule_index: None,
        kind: None,
        level: None,
        analysis_target: None,
        web_request: None,
        web_response: None,
        fingerprints: None,
        partial_fingerprints: None,
        code_flows: None,
        graphs: None,
        graph_traversals: None,
        stacks: None,
        related_locations: None,
        suppressions: None,
        baseline_state: None,
        correlation_guid: None,
        provenance: None,
        fixes: None,
        taxa: None,
        hosted_viewer_uri: None,
        work_item_uris: None,
        properties: None,
    }
}
