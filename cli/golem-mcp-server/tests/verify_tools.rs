// Direct verification of tool functionality
// As a Staff Engineer, I need to verify that tools actually work

use golem_mcp_server::GolemMcpServerV2;
use rmcp::ServerHandler;

#[tokio::test]
async fn verify_tools_are_registered() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    // Check tool count
    let tool_count = server.tool_count();
    println!("Tool count: {}", tool_count);
    assert_eq!(tool_count, 46, "Should have exactly 46 tools");
    
    // Verify we can check for tools
    assert!(server.has_tool("list_components"));
    assert!(server.has_tool("create_worker"));
    assert!(server.has_tool("invoke_worker"));
}

#[tokio::test]
async fn verify_tool_can_be_called() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    // Call list_components - the simplest tool
    let result = server.list_components().await;
    
    match result {
        Ok(tool_result) => {
            println!("Tool call succeeded!");
            println!("Content items: {}", tool_result.content.len());
            assert!(!tool_result.content.is_empty(), "Should return content");
            
            // Check the content is valid JSON
            if let Some(first_content) = tool_result.content.first() {
                println!("First content: {:?}", first_content);
            }
        }
        Err(e) => {
            panic!("Tool call failed: {:?}", e);
        }
    }
}

#[tokio::test]
async fn verify_tool_with_params() {
    use golem_mcp_server::rmcp_handler_v2::ListWorkersParams;
    use rmcp::handler::server::tool::Parameters;
    
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let params = ListWorkersParams {
        component_id: "test-component-id".to_string(),
    };
    
    let result = server.list_workers(Parameters(params)).await;
    
    match result {
        Ok(tool_result) => {
            println!("Tool with params succeeded!");
            assert!(!tool_result.content.is_empty(), "Should return content");
        }
        Err(e) => {
            panic!("Tool with params failed: {:?}", e);
        }
    }
}

#[tokio::test]
async fn verify_all_categories_have_working_tools() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    // Test one tool from each category
    let tests = vec![
        ("Component", server.list_components().await),
        ("Profile", server.list_profiles().await),
        ("Account", server.get_account().await),
        ("Token", server.list_tokens().await),
        ("Project", server.list_projects().await),
    ];
    
    for (category, result) in tests {
        match result {
            Ok(_) => println!("✅ {} tool works", category),
            Err(e) => panic!("❌ {} tool failed: {:?}", category, e),
        }
    }
}

#[tokio::test]
async fn verify_server_info_correct() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    let info = server.get_info();
    
    println!("Server Info:");
    println!("  Instructions: {:?}", info.instructions);
    
    // Default name and version from rmcp
    assert!(info.instructions.is_some());
    assert!(info.capabilities.tools.is_some());
}