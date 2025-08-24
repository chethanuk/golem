//! Integration tests for progress streaming implementation
//!
//! Tests the complete progress streaming flow from tool execution to
//! progress notification delivery with timeout handling and error recovery.

use golem_mcp_server::rmcp_handler::{
    GolemMcpServer, ComponentOperation, WorkerOperation,
};
use golem_mcp_server::progress::{
    TimeoutConfig, ProgressStreamError,
    create_progress_channel, create_progress_sender, execute_with_progress,
};
use rmcp::{
    handler::server::tool::Parameters,
};
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;
use tracing::debug;

/// Test progress streaming for component build operations
#[tokio::test]
async fn test_component_build_progress_streaming() {
    let _ = tracing_subscriber::fmt::try_init();
    
    let server = GolemMcpServer::new();
    
    // Set up progress streaming
    let mut progress_receiver = server.setup_progress_streaming(Some(16)).await;
    
    // Configure for fast operations to speed up test
    server.configure_timeouts(TimeoutConfig::fast_operations()).await;
    
    // Test component build operation
    let operation = ComponentOperation {
        operation: "build".to_string(),
        component_name: Some("test-component".to_string()),
        wasm_file: None,
        project_id: None,
        extra: json!({}),
    };
    
    // Execute in background to collect progress notifications
    let server_handle = {
        let server = server.clone();
        tokio::spawn(async move {
            server
                .golem_component_management(Parameters(operation))
                .await
        })
    };
    
    // Collect progress notifications
    let mut notifications = Vec::new();
    let timeout_duration = Duration::from_secs(5);
    let start = std::time::Instant::now();
    let mut completion_received = false;
    
    while start.elapsed() < timeout_duration && !completion_received {
        if let Ok(notification) = tokio::time::timeout(
            Duration::from_millis(100),
            progress_receiver.recv()
        ).await {
            if let Some(notification) = notification {
                debug!("Received progress notification: {:?}", notification);
                let progress = notification.progress;
                notifications.push(notification);
                
                // Stop collecting once we get completion (progress = 100)
                if progress >= 100.0 {
                    completion_received = true;
                    break;
                }
            }
        }
    }
    
    // Give a little extra time to ensure all notifications are received
    if !completion_received {
        tokio::time::sleep(Duration::from_millis(50)).await;
        while let Ok(notification) = progress_receiver.try_recv() {
            debug!("Received additional progress notification: {:?}", notification);
            notifications.push(notification);
        }
    }
    
    // Wait for server task completion
    let result = server_handle.await.unwrap();
    assert!(result.is_ok(), "Component build should succeed");
    
    // Validate progress notifications
    assert!(!notifications.is_empty(), "Should receive progress notifications");
    
    // Should have at least starting and completion notifications
    assert!(notifications.len() >= 2, "Should have at least 2 notifications");
    
    // First notification should be starting (progress = 0)
    assert_eq!(notifications[0].progress, 0.0, "First notification should be starting");
    
    // Last notification should be completion (progress = 100)
    let last_notification = notifications.last().unwrap();
    assert_eq!(last_notification.progress, 100.0, "Last notification should be completion");
    
    // All notifications should have the same token
    let expected_token = "component_build";
    for notification in &notifications {
        if let rmcp::model::ProgressToken(rmcp::model::NumberOrString::String(token)) = &notification.progress_token {
            assert_eq!(token.as_ref(), expected_token, "All notifications should have same token");
        }
    }
}

/// Test progress streaming for worker invoke operations
#[tokio::test]
async fn test_worker_invoke_progress_streaming() {
    let _ = tracing_subscriber::fmt::try_init();
    
    let server = GolemMcpServer::new();
    
    // Set up progress streaming
    let mut progress_receiver = server.setup_progress_streaming(Some(16)).await;
    
    // Configure for fast operations
    server.configure_timeouts(TimeoutConfig::fast_operations()).await;
    
    // Test worker invoke operation
    let operation = WorkerOperation {
        operation: "invoke".to_string(),
        component_id: Some("test-component".to_string()),
        worker_name: Some("test-worker".to_string()),
        function_name: Some("hello".to_string()),
        function_parameters: None,
        extra: json!({}),
    };
    
    // Execute and collect notifications
    let server_handle = {
        let server = server.clone();
        tokio::spawn(async move {
            server
                .golem_worker_management(Parameters(operation))
                .await
        })
    };
    
    let mut notifications = Vec::new();
    let timeout_duration = Duration::from_secs(5);
    let start = std::time::Instant::now();
    
    while start.elapsed() < timeout_duration {
        if let Ok(notification) = tokio::time::timeout(
            Duration::from_millis(100),
            progress_receiver.recv()
        ).await {
            if let Some(notification) = notification {
                let progress = notification.progress;
                notifications.push(notification);
                if progress >= 100.0 {
                    break;
                }
            }
        }
        
        if server_handle.is_finished() {
            break;
        }
    }
    
    let result = server_handle.await.unwrap();
    assert!(result.is_ok(), "Worker invoke should succeed");
    
    // Validate notifications
    assert!(!notifications.is_empty(), "Should receive progress notifications");
    
    let expected_token = "worker_invoke";
    for notification in &notifications {
        if let rmcp::model::ProgressToken(rmcp::model::NumberOrString::String(token)) = &notification.progress_token {
            assert_eq!(token.as_ref(), expected_token);
        }
    }
}

/// Test progress streaming timeout behavior
#[tokio::test]
async fn test_progress_streaming_timeout() {
    let (sender, _receiver) = create_progress_channel(10);
    let progress_sender = create_progress_sender(sender, "timeout_test");
    
    // Create operation that will timeout
    let slow_operation = async {
        sleep(Duration::from_secs(2)).await;
        Ok::<_, std::io::Error>("should_timeout")
    };
    
    let result = execute_with_progress(
        slow_operation,
        progress_sender,
        Duration::from_millis(100), // Very short timeout
    ).await;
    
    assert!(result.is_err(), "Operation should timeout");
    
    match result.unwrap_err() {
        ProgressStreamError::Timeout { operation_name, timeout_secs } => {
            assert_eq!(operation_name, "timeout_test");
            assert_eq!(timeout_secs, 0); // 100ms rounds to 0 seconds
        }
        other => panic!("Expected timeout error, got: {:?}", other),
    }
}

/// Test zero-cost abstraction for non-progress operations
#[tokio::test]
async fn test_zero_cost_abstraction_for_short_operations() {
    let server = GolemMcpServer::new();
    
    // Don't set up progress streaming - should still work
    
    // Test quick component list operation
    let operation = ComponentOperation {
        operation: "list".to_string(),
        component_name: None,
        wasm_file: None,
        project_id: None,
        extra: json!({}),
    };
    
    let start = std::time::Instant::now();
    let result = server
        .golem_component_management(Parameters(operation))
        .await;
    let duration = start.elapsed();
    
    assert!(result.is_ok(), "List operation should succeed");
    assert!(duration < Duration::from_millis(500), "Should be fast without progress overhead");
}

/// Test concurrent progress streaming operations
#[tokio::test]
async fn test_concurrent_progress_streaming() {
    let _ = tracing_subscriber::fmt::try_init();
    
    let server = GolemMcpServer::new();
    let mut progress_receiver = server.setup_progress_streaming(Some(64)).await;
    
    server.configure_timeouts(TimeoutConfig::fast_operations()).await;
    
    // Start multiple operations concurrently
    let operations = vec![
        ("build", ComponentOperation {
            operation: "build".to_string(),
            component_name: Some("component1".to_string()),
            wasm_file: None,
            project_id: None,
            extra: json!({}),
        }),
        ("deploy", ComponentOperation {
            operation: "deploy".to_string(),
            component_name: Some("component2".to_string()),
            wasm_file: None,
            project_id: None,
            extra: json!({}),
        }),
    ];
    
    let handles: Vec<_> = operations
        .into_iter()
        .map(|(op_name, operation)| {
            let server = server.clone();
            tokio::spawn(async move {
                (op_name, server.golem_component_management(Parameters(operation)).await)
            })
        })
        .collect();
    
    // Collect notifications from all operations
    let mut all_notifications = Vec::new();
    let mut completed_operations = 0;
    let timeout_duration = Duration::from_secs(10);
    let start = std::time::Instant::now();
    
    while start.elapsed() < timeout_duration && completed_operations < 2 {
        if let Ok(notification) = tokio::time::timeout(
            Duration::from_millis(100),
            progress_receiver.recv()
        ).await {
            if let Some(notification) = notification {
                debug!("Received concurrent notification: {:?}", notification);
                let progress = notification.progress;
                all_notifications.push(notification);
                
                // Count completion notifications
                if progress >= 100.0 {
                    completed_operations += 1;
                }
            }
        }
    }
    
    // Wait for all handles to complete
    for handle in handles {
        let (op_name, result) = handle.await.unwrap();
        assert!(result.is_ok(), "Operation {} should succeed", op_name);
    }
    
    // Validate we got notifications from both operations
    assert!(!all_notifications.is_empty(), "Should receive notifications");
    
    // Should have notifications from both component_build and component_deploy
    let mut build_notifications = 0;
    let mut deploy_notifications = 0;
    
    for notification in &all_notifications {
        if let rmcp::model::ProgressToken(rmcp::model::NumberOrString::String(token)) = &notification.progress_token {
            match token.as_ref() {
                "component_build" => build_notifications += 1,
                "component_deploy" => deploy_notifications += 1,
                _ => {}
            }
        }
    }
    
    assert!(build_notifications > 0, "Should have build notifications");
    assert!(deploy_notifications > 0, "Should have deploy notifications");
    
    debug!("Concurrent test completed - build: {}, deploy: {}", build_notifications, deploy_notifications);
}

/// Test error handling in progress streaming
#[tokio::test]
async fn test_progress_streaming_error_handling() {
    let (sender, _receiver) = create_progress_channel(10);
    let progress_sender = create_progress_sender(sender, "error_test");
    
    // Create operation that will fail
    let failing_operation = async {
        sleep(Duration::from_millis(50)).await;
        Err::<String, _>(std::io::Error::new(
            std::io::ErrorKind::Other,
            "simulated failure"
        ))
    };
    
    let result = execute_with_progress(
        failing_operation,
        progress_sender,
        Duration::from_secs(1),
    ).await;
    
    assert!(result.is_err(), "Operation should fail");
    
    match result.unwrap_err() {
        ProgressStreamError::OperationFailed { operation_name, source } => {
            assert_eq!(operation_name, "error_test");
            assert!(source.to_string().contains("simulated failure"));
        }
        other => panic!("Expected operation failure error, got: {:?}", other),
    }
}

/// Benchmark test to validate zero-cost abstractions
/// This test helps ensure that progress streaming doesn't add significant overhead
/// when progress notifications are not configured
#[tokio::test]
async fn benchmark_zero_cost_abstraction() {
    const ITERATIONS: usize = 100;
    
    let server = GolemMcpServer::new();
    
    // Test without progress streaming
    let start = std::time::Instant::now();
    for _ in 0..ITERATIONS {
        let operation = ComponentOperation {
            operation: "list".to_string(),
            component_name: None,
            wasm_file: None,
            project_id: None,
            extra: json!({}),
        };
        
        let result = server
            .golem_component_management(Parameters(operation))
            .await;
        assert!(result.is_ok());
    }
    let without_progress_duration = start.elapsed();
    
    // Test with progress streaming configured (but for non-long-running operation)
    let _progress_receiver = server.setup_progress_streaming(Some(64)).await;
    
    let start = std::time::Instant::now();
    for _ in 0..ITERATIONS {
        let operation = ComponentOperation {
            operation: "list".to_string(),
            component_name: None,
            wasm_file: None,
            project_id: None,
            extra: json!({}),
        };
        
        let result = server
            .golem_component_management(Parameters(operation))
            .await;
        assert!(result.is_ok());
    }
    let with_progress_duration = start.elapsed();
    
    // The overhead should be minimal (less than 20% slower)
    let overhead_ratio = with_progress_duration.as_nanos() as f64 / without_progress_duration.as_nanos() as f64;
    
    debug!(
        "Performance comparison: without={:?}, with={:?}, ratio={:.2}",
        without_progress_duration,
        with_progress_duration,
        overhead_ratio
    );
    
    assert!(
        overhead_ratio < 1.2,
        "Progress streaming should have minimal overhead for non-long-running operations. Ratio: {:.2}",
        overhead_ratio
    );
}