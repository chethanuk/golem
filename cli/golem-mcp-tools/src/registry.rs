//! Tool registry for MCP tool schema definitions
//!
//! Following industry patterns (Amazon Q, Codex, rmcp), this registry only defines
//! tool schemas and metadata. Execution happens in the MCP server using rmcp's
//! #[tool] macro. Tools are discovered FROM servers, not created locally.

use golem_mcp_types::{
    error::{McpError, McpResult},
    protocol::Tool,
};
use serde_json::json;
use std::collections::HashMap;
use tracing::debug;

/// Registry for MCP tool schemas and metadata
///
/// This is a simplified registry that only defines tool schemas.
/// The actual tool execution happens in golem-mcp-server using rmcp.
#[derive(Debug)]
pub struct ToolRegistry {
    /// Map of tool name to tool definition (schemas only)
    tools: HashMap<String, Tool>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    /// Create a new tool registry with default domain-based tools
    #[must_use]
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };

        // Register the 4 domain-based tool schemas
        registry.register_domain_tools();
        registry
    }

    /// Get a tool schema by name
    #[must_use]
    pub fn get_tool(&self, name: &str) -> Option<&Tool> {
        self.tools.get(name)
    }

    /// List all tool schemas
    #[must_use]
    pub fn list_tools(&self) -> Vec<&Tool> {
        self.tools.values().collect()
    }

    /// Get the JSON schema for a tool's input
    #[must_use]
    pub fn get_tool_schema(&self, name: &str) -> Option<&serde_json::Value> {
        self.tools.get(name).map(|tool| &tool.input_schema)
    }

    /// Register the 4 domain-based tool schemas
    /// 
    /// Following the industry pattern, we define 4 domain tools instead of 25+ individual ones.
    /// This matches Amazon Q's approach of grouping related operations.
    fn register_domain_tools(&mut self) {
        debug!("Registering domain-based tool schemas");
        
        // Component Management Tool
        self.tools.insert(
            "golem_component_management".to_string(),
            Tool {
                name: "golem_component_management".to_string(),
                description: "Manage Golem components (list, add, update, delete, build, deploy, etc.)".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "operation": {
                            "type": "string",
                            "description": "The operation to perform",
                            "enum": ["list", "add", "update", "delete", "get", "build", "deploy", "clean", "diagnose"]
                        },
                        "component_name": {
                            "type": "string",
                            "description": "Name of the component"
                        },
                        "wasm_file": {
                            "type": "string",
                            "description": "Path to WASM file"
                        },
                        "project_id": {
                            "type": "string",
                            "description": "Project ID"
                        }
                    },
                    "required": ["operation"]
                }),
                metadata: HashMap::from([
                    ("category".to_string(), json!("component")),
                    ("domain".to_string(), json!("component_management"))
                ]),
            },
        );

        // Worker Management Tool
        self.tools.insert(
            "golem_worker_management".to_string(),
            Tool {
                name: "golem_worker_management".to_string(),
                description: "Manage Golem workers (list, invoke, new, delete, stream, update, etc.)".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "operation": {
                            "type": "string",
                            "description": "The operation to perform",
                            "enum": ["list", "invoke", "new", "delete", "get", "stream", "update", "interrupt", "resume", "simulate_crash", "oplog", "revert", "cancel_invocation"]
                        },
                        "component_id": {
                            "type": "string",
                            "description": "Component ID (UUID)"
                        },
                        "worker_name": {
                            "type": "string",
                            "description": "Name of the worker"
                        },
                        "function_name": {
                            "type": "string",
                            "description": "Function to invoke"
                        },
                        "function_parameters": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            },
                            "description": "Function parameters"
                        }
                    },
                    "required": ["operation"]
                }),
                metadata: HashMap::from([
                    ("category".to_string(), json!("worker")),
                    ("domain".to_string(), json!("worker_management"))
                ]),
            },
        );

        // Application Management Tool
        self.tools.insert(
            "golem_application_management".to_string(),
            Tool {
                name: "golem_application_management".to_string(),
                description: "Manage Golem applications (list, deploy, update, etc.)".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "operation": {
                            "type": "string",
                            "description": "The operation to perform",
                            "enum": ["list", "deploy", "update", "delete", "status"]
                        },
                        "app_name": {
                            "type": "string",
                            "description": "Application name"
                        },
                        "manifest_path": {
                            "type": "string",
                            "description": "Path to application manifest"
                        }
                    },
                    "required": ["operation"]
                }),
                metadata: HashMap::from([
                    ("category".to_string(), json!("application")),
                    ("domain".to_string(), json!("application_management"))
                ]),
            },
        );

        // Project Management Tool
        self.tools.insert(
            "golem_project_management".to_string(),
            Tool {
                name: "golem_project_management".to_string(),
                description: "Manage Golem projects and templates (list_templates, new, etc.)".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "operation": {
                            "type": "string",
                            "description": "The operation to perform",
                            "enum": ["list_templates", "new", "init", "validate"]
                        },
                        "project_name": {
                            "type": "string",
                            "description": "Project name"
                        },
                        "template_name": {
                            "type": "string",
                            "description": "Template name"
                        },
                        "language": {
                            "type": "string",
                            "description": "Programming language filter",
                            "enum": ["rust", "python", "typescript", "javascript", "go"]
                        }
                    },
                    "required": ["operation"]
                }),
                metadata: HashMap::from([
                    ("category".to_string(), json!("project")),
                    ("domain".to_string(), json!("project_management"))
                ]),
            },
        );

        debug!("Registered {} domain-based tool schemas", self.tools.len());
    }

    /// Validate that a tool request matches the schema
    ///
    /// # Errors
    ///
    /// Returns an error if the tool doesn't exist or parameters don't match schema
    pub fn validate_tool_request(&self, tool_name: &str, _arguments: &serde_json::Value) -> McpResult<()> {
        if !self.tools.contains_key(tool_name) {
            return Err(McpError::invalid_request(format!(
                "Tool not found: {}",
                tool_name
            )));
        }

        // TODO: Add JSON schema validation of arguments against input_schema
        // For now, just check that the tool exists
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry_creation() {
        let registry = ToolRegistry::new();
        
        // Should have exactly 4 domain-based tools
        assert_eq!(registry.list_tools().len(), 4);
    }

    #[test]
    fn test_get_tool_schema() {
        let registry = ToolRegistry::new();
        
        let schema = registry.get_tool_schema("golem_component_management");
        assert!(schema.is_some());
        
        let schema = schema.unwrap();
        assert!(schema.is_object());
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].is_object());
    }

    #[test]
    fn test_domain_tools_registered() {
        let registry = ToolRegistry::new();
        
        assert!(registry.get_tool("golem_component_management").is_some());
        assert!(registry.get_tool("golem_worker_management").is_some());
        assert!(registry.get_tool("golem_application_management").is_some());
        assert!(registry.get_tool("golem_project_management").is_some());
    }

    #[test]
    fn test_tool_metadata() {
        let registry = ToolRegistry::new();
        
        let component_tool = registry.get_tool("golem_component_management").unwrap();
        assert_eq!(component_tool.metadata.get("domain").unwrap(), "component_management");
        assert_eq!(component_tool.metadata.get("category").unwrap(), "component");
    }

    #[test]
    fn test_validate_existing_tool() {
        let registry = ToolRegistry::new();
        
        let result = registry.validate_tool_request("golem_component_management", &json!({}));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_nonexistent_tool() {
        let registry = ToolRegistry::new();
        
        let result = registry.validate_tool_request("nonexistent_tool", &json!({}));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Tool not found"));
    }
}