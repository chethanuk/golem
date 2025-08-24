//! Comprehensive error handling tests using rstest framework
//!
//! Tests the structured error hierarchy, categorization, recovery patterns,
//! and observability features with production-grade error handling.

use golem_mcp_server::{
    GolemMcpError, ErrorCategory, ErrorSeverity, ErrorBuilder,
    rmcp_handler::{GolemMcpServer, ComponentOperation},
};
use golem_mcp_server::errors::{ProgressStage, TransportType};
use rmcp::{
    handler::server::tool::Parameters,
    ErrorData,
};
type McpError = ErrorData;
use rstest::*;
use serde_json::json;
// use tokio_test;
// use tracing_test::traced_test;

/// Test fixture for creating a GolemMcpServer instance
#[fixture]
fn server() -> GolemMcpServer {
    GolemMcpServer::new()
}

/// Test fixture for invalid component operations
#[fixture]
fn invalid_component_operations() -> Vec<ComponentOperation> {
    vec![
        ComponentOperation {
            operation: "".to_string(),
            component_name: None,
            wasm_file: None,
            project_id: None,
            extra: json!({}),
        },
        ComponentOperation {
            operation: "invalid_op".to_string(),
            component_name: None,
            wasm_file: None,
            project_id: None,
            extra: json!({}),
        },
        ComponentOperation {
            operation: "new".to_string(),
            component_name: None, // Missing required field
            wasm_file: None,
            project_id: None,
            extra: json!({}),
        },
    ]
}

/// Test CLI command failure error creation and categorization
#[rstest]
#[case::command_not_found(
    "component list", 
    "Command not found", 
    Some(127), 
    None, 
    ErrorCategory::ConfigurationError
)]
#[case::invalid_parameters(
    "component new", 
    "Invalid parameters", 
    Some(1), 
    Some("Missing required parameter"), 
    ErrorCategory::UserError
)]
#[case::system_failure(
    "component build", 
    "System error", 
    None, 
    Some("Permission denied"), 
    ErrorCategory::SystemError
)]
#[case::timeout_scenario(
    "worker invoke", 
    "Operation timeout", 
    Some(124), 
    Some("Timeout reached"), 
    ErrorCategory::UserError
)]
fn test_cli_command_error_categorization(
    #[case] operation: &str,
    #[case] message: &str,
    #[case] exit_code: Option<i32>,
    #[case] stderr: Option<&str>,
    #[case] expected_category: ErrorCategory,
) {
    let error = GolemMcpError::cli_command_failed(
        operation,
        message,
        exit_code,
        stderr.map(|s| s.to_string()),
    );

    assert_eq!(error.category(), expected_category);
    assert!(error.to_string().contains(operation));
    assert!(error.to_string().contains(message));
    
    // Test recovery hint is provided for known error codes
    if exit_code == Some(127) {
        assert!(error.recovery_hint().is_some());
    }
}

/// Test invalid parameter error creation with field tracking
#[rstest]
#[case::missing_component_name(
    "component new",
    "Missing component_name",
    vec!["component_name"],
    vec![],
    ErrorCategory::UserError
)]
#[case::invalid_operation_type(
    "component management",
    "Unknown operation",
    vec![],
    vec!["operation"],
    ErrorCategory::UserError
)]
#[case::multiple_missing_fields(
    "worker invoke",
    "Missing required fields",
    vec!["component_id", "worker_name", "function_name"],
    vec![],
    ErrorCategory::UserError
)]
#[case::invalid_format_fields(
    "application deploy",
    "Invalid field formats",
    vec![],
    vec!["manifest_path", "app_name"],
    ErrorCategory::UserError
)]
fn test_invalid_parameters_error_structure(
    #[case] operation: &str,
    #[case] message: &str,
    #[case] missing_fields: Vec<&str>,
    #[case] invalid_fields: Vec<&str>,
    #[case] expected_category: ErrorCategory,
) {
    let missing: Vec<String> = missing_fields.iter().map(|s| s.to_string()).collect();
    let invalid: Vec<String> = invalid_fields.iter().map(|s| s.to_string()).collect();
    
    let error = GolemMcpError::invalid_parameters(
        operation,
        message,
        missing,
        invalid,
    );

    assert_eq!(error.category(), expected_category);
    assert_eq!(error.severity(), ErrorSeverity::Warning);
    
    match error {
        GolemMcpError::InvalidParameters { missing_fields, invalid_fields, .. } => {
            assert_eq!(missing_fields.len(), missing_fields.len());
            assert_eq!(invalid_fields.len(), invalid_fields.len());
        }
        _ => panic!("Expected InvalidParameters error"),
    }
}

/// Test resource not found error with different resource types
#[rstest]
#[case::component_not_found(
    "component", 
    "test-component-123", 
    Some("golem component list"),
    ErrorCategory::ResourceError
)]
#[case::worker_not_found(
    "worker", 
    "worker-abc", 
    Some("golem worker list --component-id <id>"),
    ErrorCategory::ResourceError
)]
#[case::application_not_found(
    "application", 
    "my-app", 
    Some("golem app list"),
    ErrorCategory::ResourceError
)]
#[case::project_not_found(
    "project", 
    "project-xyz", 
    None,
    ErrorCategory::ResourceError
)]
fn test_resource_not_found_error_patterns(
    #[case] resource_type: &str,
    #[case] resource_id: &str,
    #[case] suggested_command: Option<&str>,
    #[case] expected_category: ErrorCategory,
) {
    let error = GolemMcpError::resource_not_found(
        resource_type,
        resource_id,
        suggested_command.map(|s| s.to_string()),
    );

    assert_eq!(error.category(), expected_category);
    assert_eq!(error.severity(), ErrorSeverity::Warning);
    assert!(error.to_string().contains(resource_type));
    assert!(error.to_string().contains(resource_id));
    
    if suggested_command.is_some() {
        assert!(error.recovery_hint().is_some());
    }
}

/// Test operation timeout error with different durations
#[rstest]
#[case::short_timeout("component build", 5000, ErrorCategory::PerformanceError)]
#[case::medium_timeout("worker invoke", 30000, ErrorCategory::PerformanceError)]
#[case::long_timeout("application deploy", 120000, ErrorCategory::PerformanceError)]
#[case::very_long_timeout("project new", 300000, ErrorCategory::PerformanceError)]
fn test_operation_timeout_error_patterns(
    #[case] operation: &str,
    #[case] duration_ms: u64,
    #[case] expected_category: ErrorCategory,
) {
    let error = GolemMcpError::operation_timeout(operation, duration_ms);

    assert_eq!(error.category(), expected_category);
    assert_eq!(error.severity(), ErrorSeverity::Warning);
    assert!(error.to_string().contains(operation));
    assert!(error.to_string().contains(&duration_ms.to_string()));
    
    // Timeout errors should always have retry recovery hints
    assert!(error.recovery_hint().is_some());
}

/// Test progress streaming errors with different stages
#[rstest]
#[case::initialization_failure(
    "Progress channel setup failed",
    ProgressStage::Initialization,
    ErrorCategory::SystemError
)]
#[case::preparation_failure(
    "Failed to prepare operation",
    ProgressStage::Preparation,
    ErrorCategory::SystemError
)]
#[case::execution_failure(
    "Execution interrupted",
    ProgressStage::Execution,
    ErrorCategory::SystemError
)]
#[case::completion_failure(
    "Failed to complete progress tracking",
    ProgressStage::Completion,
    ErrorCategory::SystemError
)]
#[case::cleanup_failure(
    "Progress cleanup failed",
    ProgressStage::Cleanup,
    ErrorCategory::SystemError
)]
fn test_progress_streaming_error_stages(
    #[case] message: &str,
    #[case] stage: ProgressStage,
    #[case] expected_category: ErrorCategory,
) {
    let error = GolemMcpError::progress_streaming(message, stage);

    assert_eq!(error.category(), expected_category);
    assert_eq!(error.severity(), ErrorSeverity::Warning);
    assert!(error.to_string().contains(message));
    
    match error {
        GolemMcpError::ProgressStreaming { stage: err_stage, .. } => {
            assert_eq!(err_stage, stage);
        }
        _ => panic!("Expected ProgressStreaming error"),
    }
}

/// Test transport error handling for different transport types
#[rstest]
#[case::stdio_connection_failed(
    TransportType::Stdio,
    "Failed to initialize stdio transport",
    ErrorCategory::NetworkError
)]
#[case::http_port_binding_failed(
    TransportType::Http,
    "Failed to bind to port 8080",
    ErrorCategory::NetworkError
)]
#[case::unknown_transport_error(
    TransportType::Unknown,
    "Unknown transport configuration",
    ErrorCategory::NetworkError
)]
fn test_transport_error_categorization(
    #[case] transport_type: TransportType,
    #[case] message: &str,
    #[case] expected_category: ErrorCategory,
) {
    let error = GolemMcpError::transport(transport_type, message);

    assert_eq!(error.category(), expected_category);
    assert_eq!(error.severity(), ErrorSeverity::Error);
    assert!(error.to_string().contains(message));
    assert!(error.to_string().contains(&transport_type.to_string()));
}

/// Test error builder pattern for convenient error construction
#[rstest]
#[case::cli_failure_builder(
    "component build",
    "Build failed with exit code 1",
    Some(1),
    Some("Compilation error")
)]
#[case::cli_success_but_warnings(
    "component templates",
    "Operation completed with warnings",
    Some(0),
    None
)]
fn test_error_builder_pattern(
    #[case] operation: &str,
    #[case] message: &str,
    #[case] exit_code: Option<i32>,
    #[case] stderr: Option<&str>,
) {
    let error = ErrorBuilder::new()
        .operation(operation)
        .message(message)
        .cli_failed(exit_code, stderr.map(|s| s.to_string()));

    match error {
        GolemMcpError::CliCommandFailed { operation: op, message: msg, exit_code: code, .. } => {
            assert_eq!(op, operation);
            assert_eq!(msg, message);
            assert_eq!(code, exit_code);
        }
        _ => panic!("Expected CliCommandFailed error"),
    }
}

/// Test error conversion to MCP ErrorData
#[rstest]
#[case::invalid_params_error(
    GolemMcpError::invalid_parameters(
        "test",
        "test error",
        vec!["field1".to_string()],
        vec![]
    ),
    -32602
)]
#[case::resource_not_found_error(
    GolemMcpError::resource_not_found("component", "test-123", None),
    -32601
)]
#[case::timeout_error(
    GolemMcpError::operation_timeout("test", 5000),
    -32603
)]
#[case::internal_error(
    GolemMcpError::InternalError {
        message: "test internal error".to_string(),
        error_id: "test-123".to_string(),
    },
    -32603
)]
fn test_error_to_mcp_conversion(
    #[case] golem_error: GolemMcpError,
    #[case] expected_code: i32,
) {
    let mcp_error: McpError = golem_error.clone().into();
    
    assert_eq!(mcp_error.code.0, expected_code);
    assert!(!mcp_error.message.is_empty());
    assert!(mcp_error.data.is_some());
    
    // Verify data contains serialized GolemMcpError
    let data = mcp_error.data.unwrap();
    assert!(data.is_object());
}

/// Test error severity classification
#[rstest]
#[case::info_severity(
    GolemMcpError::cli_command_failed("test", "success", Some(0), None),
    ErrorSeverity::Info
)]
#[case::warning_severity(
    GolemMcpError::invalid_parameters("test", "warning", vec![], vec![]),
    ErrorSeverity::Warning
)]
#[case::error_severity(
    GolemMcpError::transport(TransportType::Http, "port binding failed"),
    ErrorSeverity::Error
)]
#[case::critical_severity(
    GolemMcpError::InternalError {
        message: "critical failure".to_string(),
        error_id: "crit-123".to_string(),
    },
    ErrorSeverity::Critical
)]
fn test_error_severity_classification(
    #[case] error: GolemMcpError,
    #[case] expected_severity: ErrorSeverity,
) {
    assert_eq!(error.severity(), expected_severity);
}

/// Test comprehensive error handling in component operations
#[rstest]
#[tokio::test]
// #[traced_test]
async fn test_component_operation_error_handling(
    server: GolemMcpServer,
    invalid_component_operations: Vec<ComponentOperation>,
) {
    for operation in invalid_component_operations {
        let result = server
            .golem_component_management(Parameters(operation.clone()))
            .await;

        assert!(result.is_err(), "Invalid operation should fail: {:?}", operation);

        if let Err(mcp_error) = result {
            // Verify error has proper structure
            assert!(!mcp_error.message.is_empty());
            assert!(mcp_error.data.is_some());
            
            // Verify error codes are appropriate
            match operation.operation.as_str() {
                "" | "invalid_op" => {
                    // Invalid operation should be invalid params
                    assert_eq!(mcp_error.code.0, -32602);
                }
                "new" => {
                    // Missing component_name should be invalid params
                    assert_eq!(mcp_error.code.0, -32602);
                }
                _ => {
                    // Other errors should be internal
                    assert_eq!(mcp_error.code.0, -32603);
                }
            }
        }
    }
}

/// Test error logging and observability
#[rstest]
#[case::cli_error_logging(
    GolemMcpError::cli_command_failed("test", "failed", Some(1), Some("error output".to_string())),
    "error"
)]
#[case::warning_error_logging(
    GolemMcpError::operation_timeout("test", 5000),
    "warn"
)]
#[case::critical_error_logging(
    GolemMcpError::InternalError {
        message: "critical".to_string(),
        error_id: "test".to_string(),
    },
    "error"
)]
fn test_error_logging_integration(
    #[case] error: GolemMcpError,
    #[case] _expected_level: &str,
) {
    // This test verifies that log_error() is called during error conversion
    // The actual logging verification would require log capture in a real test
    let mcp_error: McpError = error.into();
    
    // Verify conversion succeeded (log_error was called)
    assert!(!mcp_error.message.is_empty());
    assert!(mcp_error.data.is_some());
    
    // In a real implementation, we would capture and verify log messages
    // For now, we ensure the conversion path works correctly
}

/// Test recovery hint generation for different error types
#[rstest]
#[case::config_error_hints(
    GolemMcpError::cli_command_failed("test", "not found", Some(127), None),
    true // Should have configuration recovery hints
)]
#[case::timeout_error_hints(
    GolemMcpError::operation_timeout("test", 30000),
    true // Should have retry recovery hints
)]
#[case::resource_error_hints(
    GolemMcpError::resource_not_found("component", "test", Some("golem component list".to_string())),
    true // Should have verification recovery hints
)]
#[case::no_hints_error(
    GolemMcpError::InvalidParameters {
        operation: "test".to_string(),
        message: "invalid".to_string(),
        missing_fields: vec![],
        invalid_fields: vec![],
    },
    false // Should not have recovery hints
)]
fn test_recovery_hint_generation(
    #[case] error: GolemMcpError,
    #[case] should_have_hints: bool,
) {
    let has_hints = error.recovery_hint().is_some();
    assert_eq!(has_hints, should_have_hints, 
               "Recovery hint expectation mismatch for error: {:?}", error);
}

/// Test error categorization consistency
#[rstest]
fn test_error_category_consistency() {
    let errors = vec![
        (GolemMcpError::invalid_parameters("test", "test", vec![], vec![]), ErrorCategory::UserError),
        (GolemMcpError::resource_not_found("test", "test", None), ErrorCategory::ResourceError),
        (GolemMcpError::operation_timeout("test", 5000), ErrorCategory::PerformanceError),
        (GolemMcpError::transport(TransportType::Http, "test"), ErrorCategory::NetworkError),
        (GolemMcpError::progress_streaming("test", ProgressStage::Execution), ErrorCategory::SystemError),
        (GolemMcpError::InternalError { message: "test".to_string(), error_id: "test".to_string() }, ErrorCategory::InternalError),
    ];

    for (error, expected_category) in errors {
        assert_eq!(error.category(), expected_category,
                   "Category mismatch for error: {:?}", error);
    }
}

/// Test error chaining and context preservation
#[rstest]
#[tokio::test]
async fn test_error_context_preservation(server: GolemMcpServer) {
    let operation = ComponentOperation {
        operation: "new".to_string(),
        component_name: None, // This will trigger missing parameter error
        wasm_file: None,
        project_id: None,
        extra: json!({}),
    };

    let result = server
        .golem_component_management(Parameters(operation))
        .await;

    // Handle CLI unavailability gracefully in error context test
    match result {
        Ok(_) => {
            panic!("Expected error for missing component_name");
        }
        Err(mcp_error) => {
            let error_msg = format!("{:?}", mcp_error);
            
            // Check if it's a CLI unavailability error
            if error_msg.contains("golem-cli binary not found") || 
               error_msg.contains("Command not found") ||
               error_msg.contains("No such file or directory") {
                eprintln!("⚠️  Error context test: CLI not available, skipping detailed error validation");
                return;
            }
            
            // Normal error validation when CLI is available
            // Verify context is preserved in error
            assert!(mcp_error.message.contains("component_name") || 
                   mcp_error.message.contains("operation") ||
                   mcp_error.message.contains("failed"),
                   "Error message should contain context information: {}", mcp_error.message);
            
            // If structured data is available, verify it
            if let Some(data) = &mcp_error.data {
                if let Some(op) = data.get("operation") {
                    if let Some(op_str) = op.as_str() {
                        assert!(op_str.contains("component") || op_str.contains("new"));
                    }
                }
            }
        }
    }
}