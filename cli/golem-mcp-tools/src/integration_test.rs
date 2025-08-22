//! Integration tests for the MCP tools functionality
//!
//! Following industry patterns, these tests verify tool schema definitions
//! rather than execution. Execution happens in the MCP server using rmcp.

#[cfg(test)]
mod tests {
    use crate::registry::ToolRegistry;
    use serde_json::json;

    #[test]
    fn test_component_management_tool_schema() {
        let registry = ToolRegistry::new();
        
        // Test that component management tool exists with proper schema
        let tool = registry.get_tool("golem_component_management");
        assert!(tool.is_some());
        
        let tool = tool.unwrap();
        assert_eq!(tool.name, "golem_component_management");
        assert!(tool.description.contains("Manage Golem components"));
        
        // Verify schema structure
        let schema = &tool.input_schema;
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["operation"].is_object());
        assert!(schema["properties"]["component_name"].is_object());
        assert!(schema["properties"]["wasm_file"].is_object());
        assert_eq!(schema["required"][0], "operation");
    }

    #[test]
    fn test_worker_management_tool_schema() {
        let registry = ToolRegistry::new();
        
        // Test that worker management tool exists with proper schema
        let tool = registry.get_tool("golem_worker_management");
        assert!(tool.is_some());
        
        let tool = tool.unwrap();
        assert_eq!(tool.name, "golem_worker_management");
        assert!(tool.description.contains("Manage Golem workers"));
        
        // Verify schema structure
        let schema = &tool.input_schema;
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["operation"].is_object());
        assert!(schema["properties"]["component_id"].is_object());
        assert!(schema["properties"]["worker_name"].is_object());
        assert!(schema["properties"]["function_name"].is_object());
        assert_eq!(schema["required"][0], "operation");
    }

    #[test]
    fn test_application_management_tool_schema() {
        let registry = ToolRegistry::new();
        
        // Test that application management tool exists with proper schema
        let tool = registry.get_tool("golem_application_management");
        assert!(tool.is_some());
        
        let tool = tool.unwrap();
        assert_eq!(tool.name, "golem_application_management");
        assert!(tool.description.contains("Manage Golem applications"));
        
        // Verify schema structure
        let schema = &tool.input_schema;
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["operation"].is_object());
        assert!(schema["properties"]["app_name"].is_object());
        assert!(schema["properties"]["manifest_path"].is_object());
    }

    #[test]
    fn test_project_management_tool_schema() {
        let registry = ToolRegistry::new();
        
        // Test that project management tool exists with proper schema
        let tool = registry.get_tool("golem_project_management");
        assert!(tool.is_some());
        
        let tool = tool.unwrap();
        assert_eq!(tool.name, "golem_project_management");
        assert!(tool.description.contains("Manage Golem projects"));
        
        // Verify schema structure
        let schema = &tool.input_schema;
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["operation"].is_object());
        assert!(schema["properties"]["project_name"].is_object());
        assert!(schema["properties"]["template_name"].is_object());
        assert!(schema["properties"]["language"].is_object());
    }

    #[test]
    fn test_exactly_four_domain_tools() {
        let registry = ToolRegistry::new();
        
        // Following industry pattern, we should have exactly 4 domain-based tools
        let tools = registry.list_tools();
        assert_eq!(tools.len(), 4);
        
        // Verify all domain tools are present
        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"golem_component_management"));
        assert!(tool_names.contains(&"golem_worker_management"));
        assert!(tool_names.contains(&"golem_application_management"));
        assert!(tool_names.contains(&"golem_project_management"));
    }

    #[test]
    fn test_tool_metadata() {
        let registry = ToolRegistry::new();
        
        // Check component tool metadata
        let component_tool = registry.get_tool("golem_component_management").unwrap();
        assert_eq!(component_tool.metadata.get("category").unwrap(), "component");
        assert_eq!(component_tool.metadata.get("domain").unwrap(), "component_management");
        
        // Check worker tool metadata
        let worker_tool = registry.get_tool("golem_worker_management").unwrap();
        assert_eq!(worker_tool.metadata.get("category").unwrap(), "worker");
        assert_eq!(worker_tool.metadata.get("domain").unwrap(), "worker_management");
    }

    #[test]
    fn test_validate_tool_request() {
        let registry = ToolRegistry::new();
        
        // Valid tool request
        let result = registry.validate_tool_request(
            "golem_component_management",
            &json!({
                "operation": "list"
            })
        );
        assert!(result.is_ok());
        
        // Invalid tool name
        let result = registry.validate_tool_request(
            "non_existent_tool",
            &json!({})
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Tool not found"));
    }

    #[test]
    fn test_component_operations_enum() {
        let registry = ToolRegistry::new();
        let tool = registry.get_tool("golem_component_management").unwrap();
        let operations = &tool.input_schema["properties"]["operation"]["enum"];
        
        assert!(operations.as_array().unwrap().contains(&json!("list")));
        assert!(operations.as_array().unwrap().contains(&json!("add")));
        assert!(operations.as_array().unwrap().contains(&json!("update")));
        assert!(operations.as_array().unwrap().contains(&json!("delete")));
        assert!(operations.as_array().unwrap().contains(&json!("build")));
        assert!(operations.as_array().unwrap().contains(&json!("deploy")));
    }

    #[test]
    fn test_worker_operations_enum() {
        let registry = ToolRegistry::new();
        let tool = registry.get_tool("golem_worker_management").unwrap();
        let operations = &tool.input_schema["properties"]["operation"]["enum"];
        
        assert!(operations.as_array().unwrap().contains(&json!("list")));
        assert!(operations.as_array().unwrap().contains(&json!("invoke")));
        assert!(operations.as_array().unwrap().contains(&json!("new")));
        assert!(operations.as_array().unwrap().contains(&json!("delete")));
        assert!(operations.as_array().unwrap().contains(&json!("stream")));
        assert!(operations.as_array().unwrap().contains(&json!("update")));
    }

    #[test]
    fn test_language_enum_values() {
        let registry = ToolRegistry::new();
        let tool = registry.get_tool("golem_project_management").unwrap();
        let languages = &tool.input_schema["properties"]["language"]["enum"];
        
        assert!(languages.as_array().unwrap().contains(&json!("rust")));
        assert!(languages.as_array().unwrap().contains(&json!("python")));
        assert!(languages.as_array().unwrap().contains(&json!("typescript")));
        assert!(languages.as_array().unwrap().contains(&json!("javascript")));
        assert!(languages.as_array().unwrap().contains(&json!("go")));
    }
}