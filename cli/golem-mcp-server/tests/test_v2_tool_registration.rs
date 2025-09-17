// Tool registration tests for V2 implementation
use golem_mcp_server::rmcp_handler_v2::*;
use rmcp::ServerHandler;

#[tokio::test]
async fn test_server_creation() {
    let server = GolemMcpServerV2::new();
    assert!(server.is_ok(), "Server should be created successfully");
}

#[tokio::test]
async fn test_server_info_capabilities() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    let info = server.get_info();
    
    assert!(info.capabilities.tools.is_some(), "Should have tools capability");
    // Tools capability being Some means it's supported
    assert!(info.instructions.is_some(), "Should have instructions");
}

#[tokio::test]
async fn test_exactly_46_tools_registered() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    // Check tool count
    assert_eq!(
        server.tool_count(), 
        46, 
        "Should have exactly 46 tools registered"
    );
}

#[tokio::test]
async fn test_all_core_component_tools_registered() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let component_tools = vec![
        "list_components",
        "deploy_component",
        "update_component",
        "delete_component",
        "get_component",
        "try_update_workers",
        "redeploy_component",
    ];
    
    for tool in component_tools {
        assert!(
            server.has_tool(tool),
            "Component tool '{}' should be registered",
            tool
        );
    }
}

#[tokio::test]
async fn test_all_worker_tools_registered() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let worker_tools = vec![
        "list_workers",
        "create_worker",
        "invoke_worker",
        "delete_worker",
        "get_worker",
        "update_worker",
        "update_workers_batch",
        "interrupt_worker",
        "resume_worker",
        "simulated_crash_worker",
        "connect_worker",
        "get_worker_oplog",
        "revert_worker",
    ];
    
    for tool in worker_tools {
        assert!(
            server.has_tool(tool),
            "Worker tool '{}' should be registered",
            tool
        );
    }
}

#[tokio::test]
async fn test_all_profile_tools_registered() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let profile_tools = vec![
        "list_profiles",
        "switch_profile",
        "init",
        "profile_init",
        "profile_add",
        "profile_config_format",
    ];
    
    for tool in profile_tools {
        assert!(
            server.has_tool(tool),
            "Profile tool '{}' should be registered",
            tool
        );
    }
}

#[tokio::test]
async fn test_all_auth_token_tools_registered() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let auth_tools = vec![
        "list_tokens",
        "add_token",
        "delete_token",
        "get_account",
        "list_projects",
        "add_project",
    ];
    
    for tool in auth_tools {
        assert!(
            server.has_tool(tool),
            "Auth/token tool '{}' should be registered",
            tool
        );
    }
}

#[tokio::test]
async fn test_all_plugin_tools_registered() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let plugin_tools = vec![
        "list_plugins",
        "get_plugin",
        "register_plugin",
        "unregister_plugin",
        "get_component_plugin",
        "install_component_plugin",
        "uninstall_component_plugin",
    ];
    
    for tool in plugin_tools {
        assert!(
            server.has_tool(tool),
            "Plugin tool '{}' should be registered",
            tool
        );
    }
}

#[tokio::test]
async fn test_all_app_tools_registered() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let app_tools = vec![
        "new_app",
        "build_app",
        "deploy_app",
        "app_clean",
        "app_custom_command",
    ];
    
    for tool in app_tools {
        assert!(
            server.has_tool(tool),
            "App tool '{}' should be registered",
            tool
        );
    }
}

#[tokio::test]
async fn test_all_permission_tools_registered() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let permission_tools = vec![
        "share_project",
        "project_policy",
    ];
    
    for tool in permission_tools {
        assert!(
            server.has_tool(tool),
            "Permission tool '{}' should be registered",
            tool
        );
    }
}

// Metadata tools were not implemented as separate tools
// They're part of the component/worker info tools

#[tokio::test]
async fn test_no_unexpected_tools_registered() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    // These should NOT be registered (they're V1 or internal)
    assert!(!server.has_tool("golem_component_list"), "V1 tool should not be registered");
    assert!(!server.has_tool("setup_progress_streaming"), "Internal method should not be a tool");
    assert!(!server.has_tool("serve"), "Server method should not be a tool");
}

#[tokio::test]
async fn test_complete_tool_list() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    // Complete list of all 46 expected tools
    let all_tools = vec![
        // Component (7)
        "list_components", "deploy_component", "update_component", "delete_component",
        "get_component", "try_update_workers", "redeploy_component",
        
        // Worker (13)
        "list_workers", "create_worker", "invoke_worker", "delete_worker",
        "get_worker", "update_worker", "update_workers_batch", "interrupt_worker",
        "resume_worker", "simulated_crash_worker", "connect_worker", 
        "get_worker_oplog", "revert_worker",
        
        // Profile (6)
        "list_profiles", "switch_profile", "init", "profile_init",
        "profile_add", "profile_config_format",
        
        // Auth/Token (6)
        "list_tokens", "add_token", "delete_token", "get_account",
        "list_projects", "add_project",
        
        // Plugin (7)
        "list_plugins", "get_plugin", "register_plugin", "unregister_plugin",
        "get_component_plugin", "install_component_plugin", "uninstall_component_plugin",
        
        // App (5)
        "new_app", "build_app", "deploy_app", "app_clean", "app_custom_command",
        
        // Permission (2)
        "share_project", "project_policy",
        
    ];
    
    // Verify we have exactly these 46 tools
    for tool in &all_tools {
        assert!(
            server.has_tool(tool),
            "Tool '{}' should be registered",
            tool
        );
    }
    
    // Count should match
    assert_eq!(
        all_tools.len(),
        46,
        "Should have 46 tools in our complete list"
    );
}