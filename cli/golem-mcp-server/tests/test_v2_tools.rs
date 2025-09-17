// Individual tool execution tests
use golem_mcp_server::rmcp_handler_v2::*;
use rmcp::handler::server::tool::Parameters;
use rmcp::model::CallToolResult;
use serde_json::json;

// Component tools tests
#[tokio::test]
async fn test_list_components_success() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let result = server.list_components().await;
    assert!(result.is_ok(), "list_components should succeed");
    
    let result = result.unwrap();
    assert!(!result.content.is_empty(), "Should return content");
}

#[tokio::test]
async fn test_deploy_component_with_invalid_path() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let params = DeployComponentParams {
        name: "test".to_string(),
        wasm_path: "/nonexistent/file.wasm".to_string(),
    };
    
    let result = server.deploy_component(Parameters(params)).await;
    // Should return an error or placeholder response
    assert!(result.is_ok(), "Should handle invalid path gracefully");
}

#[tokio::test]
async fn test_delete_component_with_invalid_uuid() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let params = DeleteComponentParams {
        component_id: "not-a-uuid".to_string(),
    };
    
    let result = server.delete_component(Parameters(params)).await;
    assert!(result.is_ok(), "Should handle invalid UUID gracefully");
}

#[tokio::test]
async fn test_get_component_details() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let params = GetComponentParams {
        component_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        version: None,
    };
    
    let result = server.get_component(Parameters(params)).await;
    assert!(result.is_ok(), "get_component should succeed");
}

#[tokio::test]
async fn test_try_update_workers_dry_run() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let params = TryUpdateWorkersParams {
        component_id: "test-component".to_string(),
        target_version: 2,
        dry_run: true,
    };
    
    let result = server.try_update_workers(Parameters(params)).await;
    assert!(result.is_ok(), "try_update_workers should succeed");
}

// Worker tools tests
#[tokio::test]
async fn test_list_workers() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let params = ListWorkersParams {
        component_id: "test-component".to_string(),
    };
    
    let result = server.list_workers(Parameters(params)).await;
    assert!(result.is_ok(), "list_workers should succeed");
}

#[tokio::test]
async fn test_create_worker_with_env() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let params = CreateWorkerParams {
        component_id: "test-component".to_string(),
        name: "worker-1".to_string(),
        env: vec![
            ("KEY1".to_string(), "value1".to_string()),
            ("KEY2".to_string(), "value2".to_string()),
        ],
        args: vec!["arg1".to_string()],
    };
    
    let result = server.create_worker(Parameters(params)).await;
    assert!(result.is_ok(), "create_worker should succeed");
}

#[tokio::test]
async fn test_invoke_worker_function() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let params = InvokeWorkerParams {
        component_id: "component-123".to_string(),
        worker_name: "worker-1".to_string(),
        function_name: "process".to_string(),
        parameters: vec![
            json!({"type": "string", "value": "hello"}),
            json!({"type": "number", "value": 42}),
        ],
    };
    
    let result = server.invoke_worker(Parameters(params)).await;
    assert!(result.is_ok(), "invoke_worker should succeed");
}

#[tokio::test]
async fn test_delete_worker() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let params = DeleteWorkerParams {
        component_id: "test-component".to_string(),
        worker_name: "worker-1".to_string(),
    };
    
    let result = server.delete_worker(Parameters(params)).await;
    assert!(result.is_ok(), "delete_worker should succeed");
}

#[tokio::test]
async fn test_get_worker_info() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let params = GetWorkerParams {
        component_id: "test-component".to_string(),
        worker_name: "worker-1".to_string(),
    };
    
    let result = server.get_worker(Parameters(params)).await;
    assert!(result.is_ok(), "get_worker should succeed");
}

#[tokio::test]
async fn test_update_worker_version() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let params = UpdateWorkerParams {
        component_id: "test-component".to_string(),
        worker_name: "worker-1".to_string(),
        target_version: 2,
        mode: golem_mcp_server::rmcp_handler_v2::UpdateMode::Automatic,
    };
    
    let result = server.update_worker(Parameters(params)).await;
    assert!(result.is_ok(), "update_worker should succeed");
}

#[tokio::test]
async fn test_interrupt_worker() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let params = InterruptWorkerParams {
        component_id: "test-component".to_string(),
        worker_name: "worker-1".to_string(),
    };
    
    let result = server.interrupt_worker(Parameters(params)).await;
    assert!(result.is_ok(), "interrupt_worker should succeed");
}

#[tokio::test]
async fn test_resume_worker() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let params = ResumeWorkerParams {
        component_id: "test-component".to_string(),
        worker_name: "worker-1".to_string(),
    };
    
    let result = server.resume_worker(Parameters(params)).await;
    assert!(result.is_ok(), "resume_worker should succeed");
}

#[tokio::test]
async fn test_get_worker_oplog() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let params = WorkerOplogParams {
        component_id: "test-component".to_string(),
        worker_name: "worker-1".to_string(),
        from_index: Some(0),
        query: Some("status=running".to_string()),
    };
    
    let result = server.get_worker_oplog(Parameters(params)).await;
    assert!(result.is_ok(), "get_worker_oplog should succeed");
}

#[tokio::test]
async fn test_simulated_crash() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let params = SimulatedCrashParams {
        component_id: "test-component".to_string(),
        worker_name: "worker-1".to_string(),
    };
    
    let result = server.simulated_crash_worker(Parameters(params)).await;
    assert!(result.is_ok(), "simulated_crash_worker should succeed");
}

// Profile tools tests
#[tokio::test]
async fn test_list_profiles() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let result = server.list_profiles().await;
    assert!(result.is_ok(), "list_profiles should succeed");
}

#[tokio::test]
async fn test_switch_profile() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let params = SwitchProfileParams {
        profile_name: "local".to_string(),
    };
    
    let result = server.switch_profile(Parameters(params)).await;
    assert!(result.is_ok(), "switch_profile should succeed");
}

#[tokio::test]
async fn test_init_profile() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let params = InitParams {
        profile_name: Some("test-profile".to_string()),
    };
    
    let result = server.init(Parameters(Some(params))).await;
    assert!(result.is_ok(), "init should succeed");
}

// Auth/Token tools tests
#[tokio::test]
async fn test_list_tokens() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let result = server.list_tokens().await;
    assert!(result.is_ok(), "list_tokens should succeed");
}

#[tokio::test]
async fn test_add_token() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let params = AddTokenParams {
        expires_at: Some("2025-12-31T23:59:59Z".to_string()),
        description: Some("Test token".to_string()),
    };
    
    let result = server.add_token(Parameters(params)).await;
    assert!(result.is_ok(), "add_token should succeed");
}

#[tokio::test]
async fn test_delete_token() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let params = DeleteTokenParams {
        token_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
    };
    
    let result = server.delete_token(Parameters(params)).await;
    assert!(result.is_ok(), "delete_token should succeed");
}

#[tokio::test]
async fn test_get_account() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let result = server.get_account().await;
    assert!(result.is_ok(), "get_account should succeed");
}

#[tokio::test]
async fn test_list_projects() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let result = server.list_projects().await;
    assert!(result.is_ok(), "list_projects should succeed");
}

// Plugin tools tests
#[tokio::test]
async fn test_list_plugins() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let params = ListPluginsParams {
        project_name: None,
        component_name: None,
    };
    
    let result = server.list_plugins(Parameters(params)).await;
    assert!(result.is_ok(), "list_plugins should succeed");
}

#[tokio::test]
async fn test_register_plugin() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let params = RegisterPluginParams {
        manifest_path: "/path/to/plugin-manifest.json".to_string(),
    };
    
    let result = server.register_plugin(Parameters(params)).await;
    assert!(result.is_ok(), "register_plugin should succeed");
}

#[tokio::test]
async fn test_install_component_plugin() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let params = InstallComponentPluginParams {
        plugin_name: "test-plugin".to_string(),
        plugin_version: "1.0.0".to_string(),
        priority: 10,
        component_name: Some("my-component".to_string()),
        params: Some({
            let mut map = std::collections::HashMap::new();
            map.insert("key1".to_string(), "value1".to_string());
            map
        }),
    };
    
    let result = server.install_component_plugin(Parameters(params)).await;
    assert!(result.is_ok(), "install_component_plugin should succeed");
}

// App tools tests
#[tokio::test]
async fn test_new_app() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let params = NewAppParams {
        app_name: "test-app".to_string(),
        template: "default".to_string(),
    };
    
    let result = server.new_app(Parameters(params)).await;
    assert!(result.is_ok(), "new_app should succeed");
}

#[tokio::test]
async fn test_build_app() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let params = BuildAppParams {
        component_name: Some("test-app".to_string()),
        build_profile: Some("release".to_string()),
    };
    
    let result = server.build_app(Parameters(params)).await;
    assert!(result.is_ok(), "build_app should succeed");
}

#[tokio::test]
async fn test_deploy_app() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let params = DeployAppParams {
        component_name: Some("test-app".to_string()),
    };
    
    let result = server.deploy_app(Parameters(params)).await;
    assert!(result.is_ok(), "deploy_app should succeed");
}

// Permission tools tests
#[tokio::test]
async fn test_share_project() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let params = ShareProjectParams {
        project_name: "my-project".to_string(),
        recipient_account_id: "account-123".to_string(),
        project_actions: vec![ProjectAction::Read, ProjectAction::Write],
    };
    
    let result = server.share_project(Parameters(params)).await;
    assert!(result.is_ok(), "share_project should succeed");
}

#[tokio::test]
async fn test_project_policy() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let params = ProjectPolicyParams {
        subcommand: PolicySubcommand::Get,
        policy: Some(serde_json::json!({
            "project": "my-project",
            "actions": ["read", "write"]
        })),
    };
    
    let result = server.project_policy(Parameters(params)).await;
    assert!(result.is_ok(), "project_policy should succeed");
}

// Batch operations test
#[tokio::test]
async fn test_update_workers_batch() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    let params = UpdateWorkersBatchParams {
        component_id: "test-component".to_string(),
        filter: Some("status=running".to_string()),
        target_version: 3,
        mode: golem_mcp_server::rmcp_handler_v2::UpdateMode::Automatic,
    };
    
    let result = server.update_workers_batch(Parameters(params)).await;
    assert!(result.is_ok(), "update_workers_batch should succeed");
}

// Complex workflow test
#[tokio::test]
async fn test_component_lifecycle_workflow() {
    let server = GolemMcpServerV2::new().expect("Server creation failed");
    
    // 1. List components
    let result = server.list_components().await;
    assert!(result.is_ok(), "Should list components");
    
    // 2. Deploy a component
    let deploy_params = DeployComponentParams {
        name: "test-component".to_string(),
        wasm_path: "/test/component.wasm".to_string(),
    };
    let result = server.deploy_component(Parameters(deploy_params)).await;
    assert!(result.is_ok(), "Should deploy component");
    
    // 3. Create a worker
    let worker_params = CreateWorkerParams {
        component_id: "test-component".to_string(),
        name: "worker-1".to_string(),
        env: vec![],
        args: vec![],
    };
    let result = server.create_worker(Parameters(worker_params)).await;
    assert!(result.is_ok(), "Should create worker");
    
    // 4. Invoke the worker
    let invoke_params = InvokeWorkerParams {
        component_id: "test-component".to_string(),
        worker_name: "worker-1".to_string(),
        function_name: "test".to_string(),
        parameters: vec![],
    };
    let result = server.invoke_worker(Parameters(invoke_params)).await;
    assert!(result.is_ok(), "Should invoke worker");
    
    // 5. Delete the worker
    let delete_params = DeleteWorkerParams {
        component_id: "test-component".to_string(),
        worker_name: "worker-1".to_string(),
    };
    let result = server.delete_worker(Parameters(delete_params)).await;
    assert!(result.is_ok(), "Should delete worker");
}