use serde::{Deserialize, Serialize};
use serde_json::Value;

/// MCP JSON-RPC request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

/// MCP JSON-RPC response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub result: Option<Value>,
    pub error: Option<McpError>,
}

/// MCP JSON-RPC error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

/// MCP Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// MCP Server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub name: String,
    pub version: String,
    pub capabilities: Option<Value>,
}

/// MCP Tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: Value,
}

/// MCP Tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub content: Vec<ToolResultContent>,
    pub is_error: Option<bool>,
}

/// Content in tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolResultContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { data: String, mime_type: String },
    #[serde(rename = "resource")]
    Resource { uri: String, mime_type: Option<String> },
}

impl McpTool {
    /// Get tool parameters from input schema
    pub fn get_parameters(&self) -> Option<&Value> {
        self.input_schema.get("properties")
    }

    /// Check if tool has a specific parameter
    pub fn has_parameter(&self, name: &str) -> bool {
        self.get_parameters()
            .and_then(|p| p.get(name))
            .is_some()
    }

    /// Get required parameters
    pub fn required_parameters(&self) -> Vec<String> {
        self.input_schema
            .get("required")
            .and_then(|r| r.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_mcp_tool_parameters() {
        let tool = McpTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "param1": {"type": "string"},
                    "param2": {"type": "number"}
                },
                "required": ["param1"]
            }),
        };

        assert!(tool.has_parameter("param1"));
        assert!(tool.has_parameter("param2"));
        assert!(!tool.has_parameter("param3"));

        let required = tool.required_parameters();
        assert_eq!(required, vec!["param1"]);
    }

    #[test]
    fn test_mcp_request_serialization() {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "tools/call".to_string(),
            params: Some(json!({"name": "test", "arguments": {}})),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("tools/call"));
    }
}
