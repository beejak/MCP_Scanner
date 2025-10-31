use anyhow::Result;
use tree_sitter::{Query, QueryCursor};
use crate::models::vulnerability::{Vulnerability, VulnerabilityType, Severity, Location};
use crate::semantic::{a_parser_for, SupportedLanguage};

pub fn detect(content: &str, file_path: &str) -> Result<Vec<Vulnerability>> {
    let language = if file_path.ends_with(".py") {
        SupportedLanguage::Python
    } else if file_path.ends_with(".js") {
        SupportedLanguage::JavaScript
    } else {
        return Ok(vec![]); // Not a supported file type
    };

    let mut parser = a_parser_for(language);
    let tree = parser.parse(content, None).unwrap();
    let root_node = tree.root_node();

    let query_str = match parser.language().unwrap().name() {
        "python" => r#"
            (call
              function: (attribute
                object: (identifier) @object
                attribute: (identifier) @function)
              arguments: (argument_list
                (keyword_argument
                  name: (identifier) @arg_name
                  value: (string) @arg_value)))
            "#,
        "javascript" => r#"
            (call_expression
              function: (member_expression
                object: (identifier) @object
                property: (identifier) @function))
            "#,
        _ => "",
    };

    let query = Query::new(parser.language().unwrap(), query_str)?;
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, root_node, content.as_bytes());

    let mut vulnerabilities = vec![];
    for mat in matches {
        for cap in mat.captures {
            let node = cap.node;
            let start_pos = node.start_position();
            let end_pos = node.end_position();

            let line = content.lines().nth(start_pos.row).unwrap_or("");

            // This is a simplified check. A real implementation would
            // have more sophisticated logic to identify vulnerabilities.
            vulnerabilities.push(Vulnerability {
                id: "CI-001".to_string(),
                vulnerability_type: VulnerabilityType::CommandInjection,
                severity: Severity::High,
                title: "Potential Command Injection".to_string(),
                description: "A function call that could be vulnerable to command injection was found.".to_string(),
                location: Location {
                    file_path: file_path.to_string(),
                    line: start_pos.row + 1,
                    column: start_pos.column + 1,
                },
                impact: "Allows attackers to execute arbitrary commands on the server.".to_string(),
                remediation: "Avoid using shell execution. Sanitize user input.".to_string(),
                code_snippet: line.to_string(),
                confidence: 0.9,
            });
        }
    }

    Ok(vulnerabilities)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_command_injection_python() {
        let content = r#"
import os
os.system("ls")
subprocess.run("ls", shell=True)
"#;
        let file_path = "test.py";
        let vulnerabilities = detect(content, file_path).unwrap();
        assert!(!vulnerabilities.is_empty());
    }
}
