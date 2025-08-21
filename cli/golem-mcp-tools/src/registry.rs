//! Tool registry for MCP tool discovery and management
//!
//! Provides a centralized registry for all available MCP tools, with automatic
//! tool discovery and schema validation.

use golem_mcp_types::{
    error::{McpError, McpResult},
    protocol::Tool,
};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, warn};

/// Registry for managing available MCP tools
#[derive(Debug, Clone)]
pub struct ToolRegistry {
    /// Map of tool name to tool definition
    tools: HashMap<String, Tool>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    /// Create a new tool registry
    #[must_use]
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };
        
        // Register default tools
        registry.register_default_tools();
        registry
    }

    /// Register a new tool
    ///
    /// # Errors
    ///
    /// Returns `McpError::InvalidRequest` if the tool definition is invalid.
    pub fn register_tool(&mut self, tool: Tool) -> McpResult<()> {
        // Validate tool definition
        self.validate_tool(&tool)?;
        
        debug!("Registering tool: {}", tool.name);
        self.tools.insert(tool.name.clone(), tool);
        Ok(())
    }

    /// Unregister a tool by name
    pub fn unregister_tool(&mut self, name: &str) -> Option<Tool> {
        debug!("Unregistering tool: {}", name);
        self.tools.remove(name)
    }

    /// Get a tool by name
    #[must_use]
    pub fn get_tool(&self, name: &str) -> Option<&Tool> {
        self.tools.get(name)
    }

    /// List all registered tools
    #[must_use]
    pub fn list_tools(&self) -> Vec<&Tool> {
        self.tools.values().collect()
    }

    /// Get tools by category
    #[must_use]
    pub fn get_tools_by_category(&self, category: &str) -> Vec<&Tool> {
        self.tools
            .values()
            .filter(|tool| {
                tool.metadata
                    .get("category")
                    .and_then(|v| v.as_str())
                    .map_or(false, |c| c == category)
            })
            .collect()
    }

    /// Search tools by name pattern
    #[must_use]
    pub fn search_tools(&self, pattern: &str) -> Vec<&Tool> {
        self.tools
            .values()
            .filter(|tool| {
                tool.name.contains(pattern) || tool.description.contains(pattern)
            })
            .collect()
    }

    /// Validate a tool definition
    fn validate_tool(&self, tool: &Tool) -> McpResult<()> {
        if tool.name.is_empty() {
            return Err(McpError::invalid_request(
                "Tool name cannot be empty".to_owned(),
            ));
        }

        if tool.description.is_empty() {
            return Err(McpError::invalid_request(
                "Tool description cannot be empty".to_owned(),
            ));
        }

        // Validate input schema is a valid JSON schema object
        if !tool.input_schema.is_object() {
            return Err(McpError::invalid_request(
                "Tool input schema must be a JSON object".to_owned(),
            ));
        }

        // Check for required schema fields
        let schema = tool.input_schema.as_object().unwrap();
        if !schema.contains_key("type") {
            return Err(McpError::invalid_request(
                "Tool input schema must have a 'type' field".to_owned(),
            ));
        }

        Ok(())
    }

    /// Register default Golem CLI tools
    fn register_default_tools(&mut self) {
        let default_tools = vec![
            self.create_component_list_tool(),
            self.create_component_add_tool(),
            self.create_worker_list_tool(),
            self.create_worker_invoke_tool(),
            self.create_template_list_tool(),
        ];

        for tool in default_tools {
            if let Err(e) = self.register_tool(tool) {
                warn!("Failed to register default tool: {}", e);
            }
        }
    }

    /// Create component list tool definition
    fn create_component_list_tool(&self) -> Tool {
        Tool {
            name: "golem_component_list".to_owned(),
            description: "List all components in the current project".to_owned(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "string",
                        "description": "Optional project ID to filter components"
                    }
                },
                "additionalProperties": false
            }),
            metadata: {
                let mut map = HashMap::new();
                map.insert("category".to_owned(), Value::String("component".to_owned()));
                map.insert("golem_command".to_owned(), Value::String("component list".to_owned()));
                map
            },
        }
    }

    /// Create component add tool definition
    fn create_component_add_tool(&self) -> Tool {
        Tool {
            name: "golem_component_add".to_owned(),
            description: "Add a new component to the project".to_owned(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "component_name": {
                        "type": "string",
                        "description": "Name of the component to add"
                    },
                    "wasm_file": {
                        "type": "string",
                        "description": "Path to the WASM file"
                    },
                    "project_id": {
                        "type": "string",
                        "description": "Optional project ID"
                    }
                },
                "required": ["component_name", "wasm_file"],
                "additionalProperties": false
            }),
            metadata: {
                let mut map = HashMap::new();
                map.insert("category".to_owned(), Value::String("component".to_owned()));
                map.insert("golem_command".to_owned(), Value::String("component add".to_owned()));
                map
            },
        }
    }

    /// Create worker list tool definition
    fn create_worker_list_tool(&self) -> Tool {
        Tool {
            name: "golem_worker_list".to_owned(),
            description: "List all workers for a component".to_owned(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "component_id": {
                        "type": "string",
                        "description": "Component ID to list workers for"
                    }
                },
                "required": ["component_id"],
                "additionalProperties": false
            }),
            metadata: {
                let mut map = HashMap::new();
                map.insert("category".to_owned(), Value::String("worker".to_owned()));
                map.insert("golem_command".to_owned(), Value::String("worker list".to_owned()));
                map
            },
        }
    }

    /// Create worker invoke tool definition
    fn create_worker_invoke_tool(&self) -> Tool {
        Tool {
            name: "golem_worker_invoke".to_owned(),
            description: "Invoke a function on a worker".to_owned(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "component_id": {
                        "type": "string",
                        "description": "Component ID"
                    },
                    "worker_name": {
                        "type": "string",
                        "description": "Worker name"
                    },
                    "function_name": {
                        "type": "string",
                        "description": "Function to invoke"
                    },
                    "function_parameters": {
                        "type": "array",
                        "description": "Function parameters",
                        "items": {
                            "type": "string"
                        }
                    }
                },
                "required": ["component_id", "worker_name", "function_name"],
                "additionalProperties": false
            }),
            metadata: {
                let mut map = HashMap::new();
                map.insert("category".to_owned(), Value::String("worker".to_owned()));
                map.insert("golem_command".to_owned(), Value::String("worker invoke-and-await".to_owned()));
                map
            },
        }
    }

    /// Create template list tool definition
    fn create_template_list_tool(&self) -> Tool {
        Tool {
            name: "golem_template_list".to_owned(),
            description: "List available Golem templates".to_owned(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "language": {
                        "type": "string",
                        "enum": ["rust", "python", "typescript", "javascript", "go"],
                        "description": "Filter templates by programming language"
                    }
                },
                "additionalProperties": false
            }),
            metadata: {
                let mut map = HashMap::new();
                map.insert("category".to_owned(), Value::String("template".to_owned()));
                map.insert("golem_command".to_owned(), Value::String("list-examples".to_owned()));
                map
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry_creation() {
        let registry = ToolRegistry::new();
        assert!(!registry.tools.is_empty());
    }

    #[test]
    fn test_tool_registration() {
        let mut registry = ToolRegistry::new();
        let tool = Tool {
            name: "test_tool".to_owned(),
            description: "A test tool".to_owned(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
            metadata: HashMap::new(),
        };

        assert!(registry.register_tool(tool.clone()).is_ok());
        assert_eq!(registry.get_tool("test_tool"), Some(&tool));
    }

    #[test]
    fn test_invalid_tool_registration() {
        let mut registry = ToolRegistry::new();
        let invalid_tool = Tool {
            name: "".to_owned(), // Invalid empty name
            description: "A test tool".to_owned(),
            input_schema: serde_json::json!({
                "type": "object"
            }),
            metadata: HashMap::new(),
        };

        assert!(registry.register_tool(invalid_tool).is_err());
    }

    #[test]
    fn test_tool_search() {
        let registry = ToolRegistry::new();
        let results = registry.search_tools("component");
        assert!(!results.is_empty());
        assert!(results.iter().any(|tool| tool.name.contains("component")));
    }

    #[test]
    fn test_tools_by_category() {
        let registry = ToolRegistry::new();
        let component_tools = registry.get_tools_by_category("component");
        assert!(!component_tools.is_empty());
        
        for tool in component_tools {
            assert_eq!(
                tool.metadata.get("category").and_then(|v| v.as_str()),
                Some("component")
            );
        }
    }
}