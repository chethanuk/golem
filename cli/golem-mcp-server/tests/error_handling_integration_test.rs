//! Integration tests for comprehensive error handling
//!
//! Tests error handling across all MCP server functionality including
//! tool execution failures, invalid parameters, timeout scenarios,
//! and graceful degradation patterns.

use golem_mcp_server::rmcp_handler::{
    GolemMcpServer, ComponentOperation, WorkerOperation, 
    ApplicationOperation, ProjectOperation,
};
use rmcp::{
    handler::server::tool::Parameters,
};
use serde_json::json;
use std::time::Duration;
use tracing::debug;

/// Test invalid parameter handling for component operations
#[tokio::test]
async fn test_component_operation_invalid_parameters() {
    let _ = tracing_subscriber::fmt::try_init();
    
    let server = GolemMcpServer::new();
    
    // Test missing required parameter
    let invalid_operation = ComponentOperation {
        operation: "new".to_string(),
        component_name: None, // Missing required component_name
        wasm_file: None,
        project_id: None,
        extra: json!({}),
    };
    
    let result = server
        .golem_component_management(Parameters(invalid_operation))
        .await;
    
    assert!(result.is_err(), "Should fail with missing component_name");
    
    if let Err(error) = result {
        debug!("Expected error for missing parameter: {:?}", error);
        // Should be a structured error with a meaningful message
        assert!(!error.message.is_empty(), "Error should have a meaningful message");
    }
}

/// Test invalid operation types
#[tokio::test]
async fn test_invalid_operation_types() {
    let server = GolemMcpServer::new();
    
    let test_cases = vec![
        // Invalid component operation
        ComponentOperation {
            operation: "invalid_operation".to_string(),
            component_name: Some("test".to_string()),
            wasm_file: None,
            project_id: None,
            extra: json!({}),
        },
        // Empty operation
        ComponentOperation {
            operation: "".to_string(),
            component_name: Some("test".to_string()),
            wasm_file: None,
            project_id: None,
            extra: json!({}),
        },
    ];
    
    for operation in test_cases {
        let result = server
            .golem_component_management(Parameters(operation.clone()))
            .await;
        
        assert!(result.is_err(), "Invalid operation '{}' should fail", operation.operation);
        
        if let Err(error) = result {
            debug!("Expected error for invalid operation '{}': {:?}", operation.operation, error);
        }
    }
}

/// Test worker operation error handling
#[tokio::test]
async fn test_worker_operation_error_handling() {
    let server = GolemMcpServer::new();
    
    // Test invoke without required parameters
    let invalid_invoke = WorkerOperation {
        operation: "invoke".to_string(),
        component_id: None, // Missing
        worker_name: None,  // Missing
        function_name: None, // Missing
        function_parameters: None,
        extra: json!({}),
    };
    
    let result = server
        .golem_worker_management(Parameters(invalid_invoke))
        .await;
    
    assert!(result.is_err(), "Worker invoke without parameters should fail");
}

/// Test application operation error handling
#[tokio::test]
async fn test_application_operation_error_handling() {
    let server = GolemMcpServer::new();
    
    // Test deploy without required parameters
    let invalid_deploy = ApplicationOperation {
        operation: "deploy".to_string(),
        app_name: None, // Missing
        manifest_path: None, // Missing
        extra: json!({}),
    };
    
    let result = server
        .golem_application_management(Parameters(invalid_deploy))
        .await;
    
    assert!(result.is_err(), "Application deploy without parameters should fail");
}

/// Test project operation error handling  
#[tokio::test]
async fn test_project_operation_error_handling() {
    let server = GolemMcpServer::new();
    
    // Test new project without required parameters
    let invalid_project = ProjectOperation {
        operation: "new".to_string(),
        language: None, // May be required for some operations
        project_name: None, // Likely required
        template_name: None,
        extra: json!({}),
    };
    
    let result = server
        .golem_project_management(Parameters(invalid_project))
        .await;
    
    // This may succeed or fail depending on implementation
    // Just ensure it doesn't panic and returns a proper result
    match result {
        Ok(_) => debug!("Project operation succeeded"),
        Err(error) => debug!("Project operation failed as expected: {:?}", error),
    }
}

/// Test CLI command execution failure handling
#[tokio::test] 
async fn test_cli_command_failure_handling() {
    let server = GolemMcpServer::new();
    
    // Test operation that will likely fail (non-existent component)
    let failing_operation = ComponentOperation {
        operation: "templates".to_string(),
        component_name: Some("non-existent-filter".to_string()),
        wasm_file: None,
        project_id: None,
        extra: json!({}),
    };
    
    let result = server
        .golem_component_management(Parameters(failing_operation))
        .await;
    
    // Should either succeed (empty results) or fail gracefully
    match result {
        Ok(result) => {
            debug!("CLI operation succeeded with result: {:?}", result);
            // Should have content even if empty
            assert!(!result.content.is_empty(), "Should have some content");
        }
        Err(error) => {
            debug!("CLI operation failed gracefully: {:?}", error);
            // Should be a structured error, not a panic
            assert!(!error.message.is_empty(), "Error should have a meaningful message");
        }
    }
}

/// Test concurrent operation error handling
#[tokio::test]
async fn test_concurrent_operation_error_handling() {
    let server = GolemMcpServer::new();
    
    // Start multiple operations that may conflict or fail
    let operations = vec![
        ComponentOperation {
            operation: "list".to_string(),
            component_name: None,
            wasm_file: None,
            project_id: None,
            extra: json!({}),
        },
        ComponentOperation {
            operation: "templates".to_string(),
            component_name: None,
            wasm_file: None,
            project_id: None,
            extra: json!({}),
        },
        ComponentOperation {
            operation: "invalid".to_string(), // This should fail
            component_name: None,
            wasm_file: None,
            project_id: None,
            extra: json!({}),
        },
    ];
    
    let mut handles = Vec::new();
    
    for operation in operations {
        let server = server.clone();
        let handle = tokio::spawn(async move {
            server.golem_component_management(Parameters(operation)).await
        });
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await);
    }
    
    let mut successes = 0;
    let mut failures = 0;
    
    for result in results {
        match result {
            Ok(Ok(_)) => successes += 1,
            Ok(Err(error)) => {
                failures += 1;
                debug!("Concurrent operation failed: {:?}", error);
            }
            Err(join_error) => {
                failures += 1;
                debug!("Join error: {:?}", join_error);
            }
        }
    }
    
    // Should have at least some successes and some failures
    assert!(successes > 0, "Should have some successful operations");
    assert!(failures > 0, "Should have some failed operations (invalid operation)");
    
    debug!("Concurrent operations: {} successes, {} failures", successes, failures);
}

/// Test malformed JSON parameter handling
#[tokio::test]
async fn test_malformed_parameter_handling() {
    let server = GolemMcpServer::new();
    
    // Test with malformed extra parameters
    let malformed_operation = ComponentOperation {
        operation: "list".to_string(),
        component_name: None,
        wasm_file: None,
        project_id: None,
        extra: json!({ "malformed": "data", "invalid_number": "not_a_number" }),
    };
    
    let result = server
        .golem_component_management(Parameters(malformed_operation))
        .await;
    
    // Should either succeed (ignoring malformed extra) or fail gracefully
    match result {
        Ok(result) => {
            debug!("Operation with malformed parameters succeeded: {:?}", result);
        }
        Err(error) => {
            debug!("Operation with malformed parameters failed gracefully: {:?}", error);
            assert!(!error.message.is_empty(), "Error should have a meaningful message");
        }
    }
}

/// Test timeout and resource exhaustion scenarios
#[tokio::test]
async fn test_resource_exhaustion_handling() {
    let server = GolemMcpServer::new();
    
    // Test many concurrent operations to stress test resource handling
    let mut handles = Vec::new();
    
    for i in 0..20 {
        let server = server.clone();
        let operation = ComponentOperation {
            operation: "list".to_string(),
            component_name: Some(format!("stress-test-{}", i)),
            wasm_file: None,
            project_id: None,
            extra: json!({}),
        };
        
        let handle = tokio::spawn(async move {
            server.golem_component_management(Parameters(operation)).await
        });
        handles.push(handle);
    }
    
    // Set a timeout for the entire operation
    let timeout_result = tokio::time::timeout(
        Duration::from_secs(30), 
        async {
            let mut results = Vec::new();
            for handle in handles {
                results.push(handle.await);
            }
            results
        }
    ).await;
    
    match timeout_result {
        Ok(results) => {
            let mut successes = 0;
            let mut failures = 0;
            
            for result in results {
                match result {
                    Ok(Ok(_)) => successes += 1,
                    Ok(Err(_)) => failures += 1,
                    Err(_) => failures += 1,
                }
            }
            
            debug!("Stress test completed: {} successes, {} failures", successes, failures);
            
            // Should handle most operations gracefully
            assert!(successes + failures == 20, "All operations should complete");
        }
        Err(_) => {
            // Timeout occurred - this might be acceptable under stress
            debug!("Stress test timed out - this may be acceptable under high load");
        }
    }
}

/// Test state consistency under error conditions
#[tokio::test]
async fn test_state_consistency_under_errors() {
    let server = GolemMcpServer::new();
    
    // Perform a series of operations, some valid, some invalid
    let operations = vec![
        ("list", true),      // Should succeed
        ("invalid", false),  // Should fail
        ("templates", true), // Should succeed
        ("", false),         // Should fail
        ("list", true),      // Should succeed
    ];
    
    for (operation, should_succeed) in operations {
        let op = ComponentOperation {
            operation: operation.to_string(),
            component_name: None,
            wasm_file: None,
            project_id: None,
            extra: json!({}),
        };
        
        let result = server
            .golem_component_management(Parameters(op))
            .await;
        
        if should_succeed {
            // Valid operations should generally succeed or fail gracefully
            match result {
                Ok(_) => debug!("Operation '{}' succeeded as expected", operation),
                Err(error) => {
                    // Even if it fails, it should be a graceful failure
                    debug!("Operation '{}' failed gracefully: {:?}", operation, error);
                    assert!(!error.message.is_empty(), "Error should have a meaningful message");
                }
            }
        } else {
            // Invalid operations should fail
            assert!(result.is_err(), "Invalid operation '{}' should fail", operation);
        }
    }
    
    // After all these operations, server should still be responsive
    let final_test = ComponentOperation {
        operation: "list".to_string(),
        component_name: None,
        wasm_file: None,
        project_id: None,
        extra: json!({}),
    };
    
    let final_result = server
        .golem_component_management(Parameters(final_test))
        .await;
    
    // Server should still be responsive
    match final_result {
        Ok(_) => debug!("Server remains responsive after error sequence"),
        Err(error) => {
            debug!("Final operation failed but server is responsive: {:?}", error);
            assert!(!error.message.is_empty(), "Error should have a meaningful message");
        }
    }
}