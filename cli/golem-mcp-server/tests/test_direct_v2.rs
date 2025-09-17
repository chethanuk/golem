// Direct test of V2 implementation
use golem_mcp_server::GolemMcpServerV2;
use rmcp::{ServerHandler, RoleServer};

#[tokio::test]
async fn test_list_tools_directly() {
    // Initialize server
    let server = GolemMcpServerV2::new().expect("Failed to create server");
    
    // Get server info
    let info = server.get_info();
    println!("Server: {} v{}", info.name, info.version);
    assert_eq!(info.name, "golem-mcp-server");
    
    // Test list_tools directly
    let result = ServerHandler::list_tools(
        &server, 
        None,
        rmcp::service::RequestContext::<RoleServer>::new()
    ).await;
    
    assert!(result.is_ok(), "list_tools should succeed");
    let tools_result = result.unwrap();
    
    println!("Found {} tools:", tools_result.tools.len());
    for tool in &tools_result.tools {
        println!("  - {}: {}", tool.name, tool.description.as_deref().unwrap_or("No description"));
    }
    
    // Verify we have exactly 13 tools
    assert_eq!(tools_result.tools.len(), 13, "Should have 13 tools");
    
    // Verify some expected tool names
    let tool_names: Vec<&str> = tools_result.tools.iter().map(|t| t.name.as_str()).collect();
    assert!(tool_names.contains(&"list_components"));
    assert!(tool_names.contains(&"deploy_component"));
    assert!(tool_names.contains(&"list_workers"));
    assert!(tool_names.contains(&"create_worker"));
}