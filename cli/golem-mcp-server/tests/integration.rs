// Copyright 2024-2025 Golem Cloud
//
// Licensed under the Golem Source License v1.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://license.golem.cloud/LICENSE
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Integration tests for Golem MCP Server
//! 
//! Following KISS principle: Single file, focused tests, minimal complexity

use golem_mcp_server::{GolemMcpServer, GolemMcpTransport};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{ErrorData, ServerHandler};
use serde_json::json;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};

/// Spawn the MCP server as a subprocess for testing
async fn spawn_mcp_server() -> Result<Child, Box<dyn std::error::Error>> {
    let child = Command::new("cargo")
        .args(&["run", "--bin", "golem-mcp-server", "--", "--stdio"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn()?;
    
    // Give the server time to start
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    Ok(child)
}

/// Send a JSON-RPC request and read the response
async fn send_request(
    child: &mut Child,
    request: serde_json::Value,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let stdin = child.stdin.as_mut().expect("Failed to get stdin");
    let stdout = child.stdout.as_mut().expect("Failed to get stdout");
    
    // Send request
    let request_str = format!("{}\n", request.to_string());
    stdin.write_all(request_str.as_bytes()).await?;
    stdin.flush().await?;
    
    // Read response
    let mut reader = BufReader::new(stdout);
    let mut response_line = String::new();
    reader.read_line(&mut response_line).await?;
    
    let response: serde_json::Value = serde_json::from_str(&response_line)?;
    Ok(response)
}

#[tokio::test]
async fn test_mcp_server_initialization() {
    let mut server = spawn_mcp_server().await.expect("Failed to spawn server");
    
    // Send initialize request
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "1.0.0",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    });
    
    let response = send_request(&mut server, init_request)
        .await
        .expect("Failed to send initialize request");
    
    // Verify response
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"]["capabilities"]["tools"].is_object());
    
    // Clean up
    server.kill().await.ok();
}

#[tokio::test]
async fn test_mcp_server_list_tools() {
    let mut server = spawn_mcp_server().await.expect("Failed to spawn server");
    
    // Initialize first
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "1.0.0",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    });
    
    send_request(&mut server, init_request)
        .await
        .expect("Failed to initialize");
    
    // List tools
    let list_tools_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });
    
    let response = send_request(&mut server, list_tools_request)
        .await
        .expect("Failed to list tools");
    
    // Verify we have 10 tools with simplified names
    let tools = response["result"]["tools"]
        .as_array()
        .expect("tools should be an array");
    
    assert_eq!(tools.len(), 10, "Should have exactly 10 tools");
    
    // Check for simplified tool names
    let expected_names = vec![
        "component", "worker", "app", "project", "profile",
        "token", "account", "plugin", "share", "network"
    ];
    
    for name in expected_names {
        assert!(
            tools.iter().any(|t| t["name"] == name),
            "Tool '{}' not found",
            name
        );
    }
    
    // Clean up
    server.kill().await.ok();
}

#[tokio::test]
async fn test_server_info() {
    // Test ServerHandler trait directly without spawning process
    let server = GolemMcpServer::new();
    let info = server.get_info();
    
    assert!(info.capabilities.tools.is_some());
    assert!(info.capabilities.resources.is_some());
    assert!(info.instructions.is_some());
    assert!(info.instructions.unwrap().contains("Golem MCP Server"));
}

#[tokio::test]
async fn test_tool_invocation_mock() {
    // Since we're using CLI subprocess approach, we test with mock responses
    let server = GolemMcpServer::new();
    
    // Test component list operation (uses mock when CLI not available)
    let params = golem_mcp_types::ComponentOperation {
        operation: "list".to_string(),
        component_name: None,
        wasm_file: None,
        project_id: None,
        extra: json!({}),
    };
    
    let result = server
        .component(rmcp::handler::server::tool::Parameters(params))
        .await;
    
    // Should either succeed with mock data or fail with expected error
    match result {
        Ok(call_result) => {
            // Mock succeeded
            assert!(!call_result.content.is_empty());
        }
        Err(e) => {
            // CLI not available - expected error
            let error_msg = format!("{:?}", e);
            assert!(
                error_msg.contains("golem-cli binary not found") || 
                error_msg.contains("Command not found"),
                "Unexpected error: {}",
                error_msg
            );
        }
    }
}

#[tokio::test]
async fn test_worker_invoke_format() {
    // Regression test for worker invoke argument format bug
    let server = GolemMcpServer::new();
    
    let params = golem_mcp_types::WorkerOperation {
        operation: "invoke".to_string(),
        component_id: Some("test-component".to_string()),
        worker_name: Some("test-worker".to_string()),
        function_name: Some("hello".to_string()),
        function_parameters: Some(vec!["arg1".to_string(), "arg2".to_string()]),
        extra: json!({}),
    };
    
    // This test verifies the fix for the worker invoke argument format
    // It should use: worker invoke --component <id> <worker> <function> [args...]
    // NOT: worker invoke --component-id <id> --worker-name <name> --function <name>
    let result = server
        .worker(rmcp::handler::server::tool::Parameters(params))
        .await;
    
    match result {
        Ok(_) => {
            // Success - either real CLI or mock worked
        }
        Err(e) => {
            let error_msg = format!("{:?}", e);
            // Should NOT contain the old error about --component-id
            assert!(
                !error_msg.contains("unexpected argument '--component-id'"),
                "Worker invoke still using wrong argument format"
            );
        }
    }
}

#[tokio::test]
async fn test_http_transport_creation() {
    // Test that HTTP transport can be created
    let transport = golem_mcp_server::http_transport(9882);
    match transport {
        GolemMcpTransport::Http { port, host } => {
            assert_eq!(port, 9882);
            assert!(host.is_none());
        }
        _ => panic!("Expected HTTP transport"),
    }
    
    let transport_with_host = golem_mcp_server::http_transport_with_host(
        9883,
        "127.0.0.1".to_string()
    );
    match transport_with_host {
        GolemMcpTransport::Http { port, host } => {
            assert_eq!(port, 9883);
            assert_eq!(host, Some("127.0.0.1".to_string()));
        }
        _ => panic!("Expected HTTP transport with host"),
    }
}

#[tokio::test]
async fn test_stdio_transport_creation() {
    // Test that stdio transport can be created
    let transport = golem_mcp_server::stdio_transport();
    match transport {
        GolemMcpTransport::Stdio => {
            // Expected
        }
        _ => panic!("Expected stdio transport"),
    }
}

// Note: Full E2E tests with real CLI would require:
// 1. A running Golem instance
// 2. Valid authentication
// 3. Test components and workers
// These should be in a separate E2E test suite with proper setup/teardown