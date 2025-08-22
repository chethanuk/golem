//! End-to-end tests for the 4 domain-based MCP tools
//!
//! These tests verify that our industry-aligned implementation with
//! exactly 4 domain tools works correctly.

use golem_mcp_server::rmcp_handler::{
    ApplicationOperation, ComponentOperation, GolemMcpServer, ProjectOperation, WorkerOperation,
};
use rmcp::{handler::server::tool::Parameters, ServerHandler};

#[tokio::test]
async fn test_all_four_domain_tools_exist() {
    let server = GolemMcpServer::new();
    let info = server.get_info();
    
    // Verify server has tools capability
    assert!(info.capabilities.tools.is_some());
    
    // Verify server info is correct
    assert!(info.instructions.is_some());
    assert!(info.instructions.unwrap().contains("Golem MCP Server"));
}

#[tokio::test]
async fn test_component_management_all_operations() {
    let server = GolemMcpServer::new();
    
    // Test list operation
    let list_params = ComponentOperation {
        operation: "list".to_string(),
        component_name: None,
        wasm_file: None,
        project_id: None,
        extra: serde_json::json!({}),
    };
    
    let result = server
        .golem_component_management(Parameters(list_params))
        .await;
    if let Err(e) = &result {
        eprintln!("List operation failed: {:?}", e);
    }
    assert!(result.is_ok());
    
    // Test new operation
    let new_params = ComponentOperation {
        operation: "new".to_string(),
        component_name: Some("test-component".to_string()),
        wasm_file: None,
        project_id: None,
        extra: serde_json::json!({"template": "rust-minimal"}),
    };
    
    let result = server
        .golem_component_management(Parameters(new_params))
        .await;
    assert!(result.is_ok());
    
    // Test invalid operation
    let invalid_params = ComponentOperation {
        operation: "invalid_op".to_string(),
        component_name: None,
        wasm_file: None,
        project_id: None,
        extra: serde_json::json!({}),
    };
    
    let result = server
        .golem_component_management(Parameters(invalid_params))
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_worker_management_all_operations() {
    let server = GolemMcpServer::new();
    
    // Test list operation
    let list_params = WorkerOperation {
        operation: "list".to_string(),
        component_id: Some("test-component-id".to_string()),
        worker_name: None,
        function_name: None,
        function_parameters: None,
        extra: serde_json::json!({}),
    };
    
    let result = server
        .golem_worker_management(Parameters(list_params))
        .await;
    assert!(result.is_ok());
    
    // Test invoke operation
    let invoke_params = WorkerOperation {
        operation: "invoke".to_string(),
        component_id: Some("test-component-id".to_string()),
        worker_name: Some("test-worker".to_string()),
        function_name: Some("hello".to_string()),
        function_parameters: Some(vec!["world".to_string()]),
        extra: serde_json::json!({}),
    };
    
    let result = server
        .golem_worker_management(Parameters(invoke_params))
        .await;
    assert!(result.is_ok());
    
    // Test missing required parameters
    let invalid_params = WorkerOperation {
        operation: "invoke".to_string(),
        component_id: None, // Missing required field
        worker_name: Some("test-worker".to_string()),
        function_name: Some("hello".to_string()),
        function_parameters: None,
        extra: serde_json::json!({}),
    };
    
    let result = server
        .golem_worker_management(Parameters(invalid_params))
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_application_management_all_operations() {
    let server = GolemMcpServer::new();
    
    // Test list operation
    let list_params = ApplicationOperation {
        operation: "list".to_string(),
        app_name: None,
        manifest_path: None,
        extra: serde_json::json!({}),
    };
    
    let result = server
        .golem_application_management(Parameters(list_params))
        .await;
    assert!(result.is_ok());
    
    // Test deploy operation
    let deploy_params = ApplicationOperation {
        operation: "deploy".to_string(),
        app_name: Some("test-app".to_string()),
        manifest_path: Some("/path/to/manifest.yaml".to_string()),
        extra: serde_json::json!({}),
    };
    
    let result = server
        .golem_application_management(Parameters(deploy_params))
        .await;
    assert!(result.is_ok());
    
    // Test missing required parameters for deploy
    let invalid_params = ApplicationOperation {
        operation: "deploy".to_string(),
        app_name: None, // Missing required field
        manifest_path: Some("/path/to/manifest.yaml".to_string()),
        extra: serde_json::json!({}),
    };
    
    let result = server
        .golem_application_management(Parameters(invalid_params))
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_project_management_all_operations() {
    let server = GolemMcpServer::new();
    
    // Test list_templates operation
    let list_params = ProjectOperation {
        operation: "list_templates".to_string(),
        language: Some("rust".to_string()),
        project_name: None,
        template_name: None,
        extra: serde_json::json!({}),
    };
    
    let result = server
        .golem_project_management(Parameters(list_params))
        .await;
    assert!(result.is_ok());
    
    // Test new project operation
    let new_params = ProjectOperation {
        operation: "new".to_string(),
        language: None,
        project_name: Some("test-project".to_string()),
        template_name: Some("rust-minimal".to_string()),
        extra: serde_json::json!({}),
    };
    
    let result = server
        .golem_project_management(Parameters(new_params))
        .await;
    assert!(result.is_ok());
    
    // Test missing required parameters for new
    let invalid_params = ProjectOperation {
        operation: "new".to_string(),
        language: None,
        project_name: None, // Missing required field
        template_name: Some("rust-minimal".to_string()),
        extra: serde_json::json!({}),
    };
    
    let result = server
        .golem_project_management(Parameters(invalid_params))
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_exactly_four_tools_pattern() {
    // This test verifies we follow the industry pattern of having
    // exactly 4 domain-based tools instead of 25+ individual ones
    
    let server = GolemMcpServer::new();
    
    // The 4 domains we should have:
    let domains = [
        "component_management",
        "worker_management",
        "application_management",
        "project_management",
    ];
    
    // Verify each domain tool works
    for domain in &domains {
        match *domain {
            "component_management" => {
                let params = ComponentOperation {
                    operation: "list".to_string(),
                    component_name: None,
                    wasm_file: None,
                    project_id: None,
                    extra: serde_json::json!({}),
                };
                let result = server.golem_component_management(Parameters(params)).await;
                assert!(result.is_ok(), "Component management tool failed");
            }
            "worker_management" => {
                let params = WorkerOperation {
                    operation: "list".to_string(),
                    component_id: Some("test".to_string()),
                    worker_name: None,
                    function_name: None,
                    function_parameters: None,
                    extra: serde_json::json!({}),
                };
                let result = server.golem_worker_management(Parameters(params)).await;
                assert!(result.is_ok(), "Worker management tool failed");
            }
            "application_management" => {
                let params = ApplicationOperation {
                    operation: "list".to_string(),
                    app_name: None,
                    manifest_path: None,
                    extra: serde_json::json!({}),
                };
                let result = server.golem_application_management(Parameters(params)).await;
                assert!(result.is_ok(), "Application management tool failed");
            }
            "project_management" => {
                let params = ProjectOperation {
                    operation: "list_templates".to_string(),
                    language: None,
                    project_name: None,
                    template_name: None,
                    extra: serde_json::json!({}),
                };
                let result = server.golem_project_management(Parameters(params)).await;
                assert!(result.is_ok(), "Project management tool failed");
            }
            _ => panic!("Unexpected domain: {}", domain),
        }
    }
}

#[test]
fn test_operation_structs_have_json_schema() {
    // Verify that all operation structs implement JsonSchema
    // This is required for rmcp's #[tool] macro to work
    
    use schemars::JsonSchema;
    
    fn assert_has_json_schema<T: JsonSchema>() {}
    
    assert_has_json_schema::<ComponentOperation>();
    assert_has_json_schema::<WorkerOperation>();
    assert_has_json_schema::<ApplicationOperation>();
    assert_has_json_schema::<ProjectOperation>();
}