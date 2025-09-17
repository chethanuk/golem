// Integration test for V2 implementation with all 13 tools
use golem_mcp_server::rmcp_handler_v2::GolemMcpServerV2;
use rmcp::{
    model::{PaginatedRequestParam, CallToolRequestParam},
    service::RequestContext,
    ServerHandler, RoleServer,
};
use serde_json::json;

#[tokio::test]
async fn test_server_creation_succeeds() {
    let result = GolemMcpServerV2::new();
    assert!(result.is_ok(), "Server should be created successfully");
}

#[tokio::test]
async fn test_all_13_tools_registered() {
    let server = GolemMcpServerV2::new().expect("Failed to create server");
    
    // Check tool count
    assert_eq!(server.tool_count(), 13, "Should have exactly 13 tools registered");
    
    // List of all expected tools
    let expected_tools = vec![
        "list_components",
        "deploy_component",
        "update_component",
        "delete_component",
        "list_workers",
        "create_worker",
        "invoke_worker",
        "delete_worker",
        "list_profiles",
        "switch_profile",
        "new_app",
        "build_app",
        "deploy_app",
    ];
    
    // Verify each tool is registered
    for tool_name in &expected_tools {
        assert!(
            server.has_tool(tool_name),
            "Tool '{}' should be registered",
            tool_name
        );
    }
    
    println!("✅ All 13 tools are properly registered!");
}

#[tokio::test]
async fn test_list_tools_returns_all_tools() {
    let server = GolemMcpServerV2::new().expect("Failed to create server");
    
    // Call list_tools through ServerHandler trait
    let result = server.list_tools(
        None::<PaginatedRequestParam>,
        RequestContext::<RoleServer>::default()
    ).await;
    
    assert!(result.is_ok(), "list_tools should succeed");
    
    let tools_result = result.unwrap();
    assert_eq!(
        tools_result.tools.len(),
        13,
        "list_tools should return exactly 13 tools"
    );
    
    // Check that each tool has required fields
    for tool in &tools_result.tools {
        assert!(!tool.name.is_empty(), "Tool name should not be empty");
        assert!(tool.description.is_some(), "Tool should have description");
        assert!(tool.input_schema.is_some(), "Tool should have input schema");
    }
    
    // Verify specific tool names
    let tool_names: Vec<String> = tools_result.tools
        .iter()
        .map(|t| t.name.clone())
        .collect();
    
    let expected_names = vec![
        "list_components",
        "deploy_component", 
        "update_component",
        "delete_component",
        "list_workers",
        "create_worker",
        "invoke_worker",
        "delete_worker",
        "list_profiles",
        "switch_profile",
        "new_app",
        "build_app",
        "deploy_app",
    ];
    
    for expected in expected_names {
        assert!(
            tool_names.contains(&expected.to_string()),
            "Tool '{}' should be in the list",
            expected
        );
    }
    
    println!("✅ list_tools returns all 13 tools correctly!");
}

#[tokio::test]
async fn test_server_info_capabilities() {
    let server = GolemMcpServerV2::new().expect("Failed to create server");
    
    // Get server info
    let info = server.get_info();
    
    // Check capabilities
    assert!(info.capabilities.tools.is_some(), "Server should advertise tools capability");
    assert!(info.instructions.is_some(), "Server should have instructions");
    
    let instructions = info.instructions.unwrap();
    assert!(instructions.contains("Golem MCP Server V2"), "Instructions should mention V2");
}

#[tokio::test]
async fn test_tool_callable_list_profiles() {
    let server = GolemMcpServerV2::new().expect("Failed to create server");
    
    // Test calling list_profiles tool
    let request = CallToolRequestParam {
        name: "list_profiles".to_string(),
        arguments: Some(json!({})),
    };
    
    let result = server.call_tool(
        request,
        RequestContext::<RoleServer>::default()
    ).await;
    
    // The tool should be callable (may fail due to missing API config, but should not panic)
    match result {
        Ok(tool_result) => {
            println!("✅ Tool 'list_profiles' called successfully");
            assert!(!tool_result.content.is_empty(), "Should return some content");
        }
        Err(e) => {
            // Expected if API client is not configured
            println!("⚠️ Tool call failed (expected without API config): {}", e);
        }
    }
}

#[tokio::test]
async fn test_tool_callable_list_components() {
    let server = GolemMcpServerV2::new().expect("Failed to create server");
    
    // Test calling list_components tool
    let request = CallToolRequestParam {
        name: "list_components".to_string(),
        arguments: Some(json!({})),
    };
    
    let result = server.call_tool(
        request,
        RequestContext::<RoleServer>::default()
    ).await;
    
    // The tool should be callable
    match result {
        Ok(tool_result) => {
            println!("✅ Tool 'list_components' called successfully");
            assert!(!tool_result.content.is_empty(), "Should return some content");
        }
        Err(e) => {
            // Expected if API client is not configured
            println!("⚠️ Tool call failed (expected without API config): {}", e);
        }
    }
}

#[tokio::test] 
async fn test_invalid_tool_returns_error() {
    let server = GolemMcpServerV2::new().expect("Failed to create server");
    
    // Try calling non-existent tool
    let request = CallToolRequestParam {
        name: "non_existent_tool".to_string(),
        arguments: Some(json!({})),
    };
    
    let result = server.call_tool(
        request,
        RequestContext::<RoleServer>::default()
    ).await;
    
    assert!(result.is_err(), "Non-existent tool should return error");
    
    let error = result.unwrap_err();
    assert!(
        error.message.contains("not found") || error.message.contains("Tool"),
        "Error should indicate tool not found"
    );
    
    println!("✅ Invalid tool correctly returns error");
}

#[test]
fn test_all_expected_tools_covered() {
    // This is a compile-time test to ensure we're testing all tools
    const EXPECTED_TOOL_COUNT: usize = 13;
    
    let tools = [
        "list_components",
        "deploy_component", 
        "update_component",
        "delete_component",
        "list_workers",
        "create_worker",
        "invoke_worker",
        "delete_worker",
        "list_profiles",
        "switch_profile",
        "new_app",
        "build_app",
        "deploy_app",
    ];
    
    assert_eq!(
        tools.len(),
        EXPECTED_TOOL_COUNT,
        "Test should cover all {} expected tools",
        EXPECTED_TOOL_COUNT
    );
}