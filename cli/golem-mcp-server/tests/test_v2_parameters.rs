// Parameter validation and serialization tests
use golem_mcp_server::rmcp_handler_v2::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[test]
fn test_deploy_component_params_serialization() {
    let params = DeployComponentParams {
        name: "test-component".to_string(),
        wasm_path: "/path/to/component.wasm".to_string(),
    };
    
    // Serialize to JSON
    let json = serde_json::to_value(&params).unwrap();
    assert_eq!(json["name"], "test-component");
    assert_eq!(json["wasm_path"], "/path/to/component.wasm");
    
    // Deserialize back
    let deserialized: DeployComponentParams = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(deserialized.name, params.name);
    assert_eq!(deserialized.wasm_path, params.wasm_path);
    
    // Verify JSON structure
    let json_str = serde_json::to_string(&params).unwrap();
    assert!(json_str.contains("test-component"));
    assert!(json_str.contains("/path/to/component.wasm"));
}

#[test]
fn test_create_worker_params_with_env_and_args() {
    let params = CreateWorkerParams {
        component_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        name: "worker-1".to_string(),
        env: vec![
            ("KEY1".to_string(), "value1".to_string()),
            ("KEY2".to_string(), "value2".to_string()),
        ],
        args: vec!["arg1".to_string(), "arg2".to_string()],
    };
    
    let json = serde_json::to_value(&params).unwrap();
    assert_eq!(json["component_id"], "550e8400-e29b-41d4-a716-446655440000");
    assert_eq!(json["name"], "worker-1");
    assert_eq!(json["env"].as_array().unwrap().len(), 2);
    assert_eq!(json["args"].as_array().unwrap().len(), 2);
}

#[test]
fn test_optional_fields_serialization() {
    // Test with None values
    let params1 = ListPluginsParams {
        project_name: None,
        component_name: None,
    };
    
    let json1 = serde_json::to_value(&params1).unwrap();
    assert!(!json1.get("project_name").is_some(), "None fields should not serialize");
    assert!(!json1.get("component_name").is_some(), "None fields should not serialize");
    
    // Test with Some values
    let params2 = ListPluginsParams {
        project_name: Some("project-1".to_string()),
        component_name: Some("component-1".to_string()),
    };
    
    let json2 = serde_json::to_value(&params2).unwrap();
    assert_eq!(json2["project_name"], "project-1");
    assert_eq!(json2["component_name"], "component-1");
}

#[test]
fn test_enum_serialization() {
    // Test ProfileType enum
    let cloud = ProfileType::Cloud;
    let local = ProfileType::Local;
    let custom = ProfileType::Custom;
    
    assert_eq!(serde_json::to_value(&cloud).unwrap(), "Cloud");
    assert_eq!(serde_json::to_value(&local).unwrap(), "Local");
    assert_eq!(serde_json::to_value(&custom).unwrap(), "Custom");
    
    // Test DeploymentStrategy enum
    let blue_green = DeploymentStrategy::BlueGreen;
    let rolling = DeploymentStrategy::Rolling;
    let recreate = DeploymentStrategy::Recreate;
    
    assert_eq!(serde_json::to_value(&blue_green).unwrap(), "BlueGreen");
    assert_eq!(serde_json::to_value(&rolling).unwrap(), "Rolling");
    assert_eq!(serde_json::to_value(&recreate).unwrap(), "Recreate");
}

#[test]
fn test_all_param_structs_derive_required_traits() {
    // Helper function to verify traits
    fn assert_param_traits<T: Serialize + for<'de> Deserialize<'de> + JsonSchema + Clone + std::fmt::Debug>() {}
    
    // Component params
    assert_param_traits::<DeployComponentParams>();
    assert_param_traits::<UpdateComponentParams>();
    assert_param_traits::<DeleteComponentParams>();
    assert_param_traits::<GetComponentParams>();
    assert_param_traits::<TryUpdateWorkersParams>();
    assert_param_traits::<RedeployComponentParams>();
    
    // Worker params
    assert_param_traits::<CreateWorkerParams>();
    assert_param_traits::<InvokeWorkerParams>();
    assert_param_traits::<DeleteWorkerParams>();
    assert_param_traits::<GetWorkerParams>();
    assert_param_traits::<UpdateWorkerParams>();
    assert_param_traits::<UpdateWorkersBatchParams>();
    assert_param_traits::<InterruptWorkerParams>();
    assert_param_traits::<ResumeWorkerParams>();
    assert_param_traits::<SimulatedCrashParams>();
    assert_param_traits::<ConnectWorkerParams>();
    assert_param_traits::<WorkerOplogParams>();
    assert_param_traits::<RevertWorkerParams>();
    
    // Profile params
    assert_param_traits::<SwitchProfileParams>();
    assert_param_traits::<InitParams>();
    assert_param_traits::<ProfileAddParams>();
    assert_param_traits::<ProfileConfigFormatParams>();
    
    // Auth/Token params
    assert_param_traits::<AddTokenParams>();
    assert_param_traits::<DeleteTokenParams>();
    assert_param_traits::<AddProjectParams>();
    
    // Plugin params
    assert_param_traits::<ListPluginsParams>();
    assert_param_traits::<GetPluginParams>();
    assert_param_traits::<RegisterPluginParams>();
    assert_param_traits::<UnregisterPluginParams>();
    assert_param_traits::<GetComponentPluginParams>();
    assert_param_traits::<InstallComponentPluginParams>();
    assert_param_traits::<UninstallComponentPluginParams>();
    
    // App params
    assert_param_traits::<NewAppParams>();
    assert_param_traits::<BuildAppParams>();
    assert_param_traits::<DeployAppParams>();
    assert_param_traits::<AppCleanParams>();
    assert_param_traits::<AppCustomCommandParams>();
    
    // Permission params
    assert_param_traits::<ShareProjectParams>();
    assert_param_traits::<ProjectPolicyParams>();
}

#[test]
fn test_uuid_validation_in_params() {
    // Valid UUID
    let valid_params = DeleteComponentParams {
        component_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
    };
    
    let json = serde_json::to_value(&valid_params).unwrap();
    assert_eq!(json["component_id"], "550e8400-e29b-41d4-a716-446655440000");
    
    // Invalid UUID format (will be caught during execution, not serialization)
    let invalid_params = DeleteComponentParams {
        component_id: "not-a-uuid".to_string(),
    };
    
    // Should still serialize (validation happens in the tool implementation)
    let json = serde_json::to_value(&invalid_params).unwrap();
    assert_eq!(json["component_id"], "not-a-uuid");
}

#[test]
fn test_complex_nested_params() {
    let params = InstallComponentPluginParams {
        plugin_name: "test-plugin".to_string(),
        plugin_version: "1.0.0".to_string(),
        priority: 10,
        component_name: Some("my-component".to_string()),
        params: Some({
            let mut map = std::collections::HashMap::new();
            map.insert("key1".to_string(), "value1".to_string());
            map.insert("key2".to_string(), "value2".to_string());
            map
        }),
    };
    
    let json = serde_json::to_value(&params).unwrap();
    assert_eq!(json["plugin_name"], "test-plugin");
    assert_eq!(json["plugin_version"], "1.0.0");
    assert_eq!(json["priority"], 10);
    assert_eq!(json["component_name"], "my-component");
    assert_eq!(json["params"]["key1"], "value1");
    assert_eq!(json["params"]["key2"], "value2");
    
    // Deserialize back
    let deserialized: InstallComponentPluginParams = 
        serde_json::from_value(json).unwrap();
    assert_eq!(deserialized.plugin_name, "test-plugin");
    assert_eq!(deserialized.priority, 10);
    assert!(deserialized.params.is_some());
}

#[test]
fn test_default_values() {
    // Test default for DeploymentStrategy
    let params = RedeployComponentParams {
        component_id: "test-id".to_string(),
        strategy: Default::default(),
    };
    
    let json = serde_json::to_value(&params.strategy).unwrap();
    assert_eq!(json, "BlueGreen", "Default should be BlueGreen");
    
    // Test default for dry_run
    let try_params: TryUpdateWorkersParams = serde_json::from_value(json!({
        "component_id": "test-id",
        "target_version": 2
    })).unwrap();
    
    assert!(try_params.dry_run, "dry_run should default to true");
}

#[test]
fn test_project_action_enum() {
    let actions = vec![
        ProjectAction::Read,
        ProjectAction::Write,
        ProjectAction::Delete,
        ProjectAction::Admin,
    ];
    
    for action in actions {
        let json = serde_json::to_value(&action).unwrap();
        let deserialized: ProjectAction = serde_json::from_value(json).unwrap();
        assert!(matches!(deserialized, ProjectAction::Read | ProjectAction::Write | ProjectAction::Delete | ProjectAction::Admin));
    }
}

#[test]
fn test_policy_subcommand_enum() {
    let commands = vec![
        PolicySubcommand::Get,
        PolicySubcommand::Set,
        PolicySubcommand::Delete,
        PolicySubcommand::List,
    ];
    
    for cmd in commands {
        let json = serde_json::to_value(&cmd).unwrap();
        let deserialized: PolicySubcommand = serde_json::from_value(json).unwrap();
        assert!(matches!(deserialized, PolicySubcommand::Get | PolicySubcommand::Set | PolicySubcommand::Delete | PolicySubcommand::List));
    }
}

#[test]
fn test_invoke_worker_params_with_json_parameters() {
    let params = InvokeWorkerParams {
        component_id: "component-123".to_string(),
        worker_name: "worker-1".to_string(),
        function_name: "process".to_string(),
        parameters: vec![
            json!({"type": "string", "value": "hello"}),
            json!({"type": "number", "value": 42}),
        ],
    };
    
    let json = serde_json::to_value(&params).unwrap();
    assert_eq!(json["function_name"], "process");
    assert_eq!(json["parameters"].as_array().unwrap().len(), 2);
    assert_eq!(json["parameters"][0]["value"], "hello");
    assert_eq!(json["parameters"][1]["value"], 42);
}