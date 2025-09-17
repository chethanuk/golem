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

//! Integration tests for MCP server tool registration and discovery
//!
//! This module tests that tools are properly registered and discoverable
//! through the MCP protocol using STDIO transport.

use golem_mcp_server::GolemMcpServerV2;
use rmcp::{
    model::{ListToolsRequest, ListToolsResult, Tool},
    transport::io::stdio,
    Service, ServiceExt, ServerHandler,
};
use serde_json::Value;
use std::collections::HashSet;
use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream};

/// Test helper for creating a mock STDIO transport using DuplexStream
fn create_mock_stdio() -> (DuplexStream, DuplexStream) {
    tokio::io::duplex(1024)
}

/// Test that all tools are properly registered with the tool router
#[tokio::test]
async fn test_tool_registration() {
    // Create server instance
    let server = GolemMcpServerV2::new()
        .expect("Failed to create server");

    // Get server info to verify capabilities
    let info = server.get_info();
    assert!(info.capabilities.tools.unwrap_or(false), 
        "Tools capability should be enabled");

    // Expected tools that should be registered
    let expected_tools = vec![
        "list_components",
        "deploy_component",
        "update_component",
        "delete_component",
        "list_profiles",
        "switch_profile",
        "list_workers",
        "create_worker",
        "invoke_worker",
        "delete_worker",
        "new_app",
        "build_app",
        "deploy_app",
    ];

    // Verify each tool exists in the router
    for tool_name in expected_tools {
        println!("Checking tool registration: {}", tool_name);
        // Tool router should have the tool registered
        assert!(server.tool_router.map.contains_key(tool_name),
            "Tool '{}' should be registered in tool_router", tool_name);
    }
}

/// Test tools/list returns all registered tools
#[tokio::test]
async fn test_tools_list() {
    // Create server instance
    let server = GolemMcpServerV2::new()
        .expect("Failed to create server");

    // Create mock request context
    let request = ListToolsRequest::default();
    let context = rmcp::handler::RequestContext {
        sender: None,
        meta: Default::default(),
    };

    // Call list_tools directly on the server handler
    let result = server.list_tools(request, context).await
        .expect("list_tools should succeed");

    // Verify we have tools registered
    assert!(!result.tools.is_empty(), 
        "Should have registered tools");

    // Verify expected tools are present
    let tool_names: HashSet<String> = result.tools.iter()
        .map(|t| t.name.clone())
        .collect();

    let expected_tools = vec![
        "list_components",
        "deploy_component",
        "update_component",
        "delete_component",
        "list_profiles",
        "switch_profile",
        "list_workers",
        "create_worker",
        "invoke_worker",
        "delete_worker",
        "new_app",
        "build_app",
        "deploy_app",
    ];

    for tool in expected_tools {
        assert!(tool_names.contains(tool),
            "Tool '{}' should be in tools/list response", tool);
    }
}

/// Test tool metadata is properly populated
#[tokio::test]
async fn test_tool_metadata() {
    let server = GolemMcpServerV2::new()
        .expect("Failed to create server");

    let request = ListToolsRequest::default();
    let context = rmcp::handler::RequestContext {
        sender: None,
        meta: Default::default(),
    };

    let result = server.list_tools(request, context).await
        .expect("list_tools should succeed");

    // Verify each tool has proper metadata
    for tool in &result.tools {
        assert!(!tool.name.is_empty(), 
            "Tool name should not be empty");
        assert!(tool.description.is_some(), 
            "Tool {} should have a description", tool.name);
        assert!(tool.input_schema.is_some(),
            "Tool {} should have input schema", tool.name);
        
        // Verify input schema is valid JSON schema
        if let Some(schema) = &tool.input_schema {
            let schema_json: Value = serde_json::from_value(schema.clone())
                .expect(&format!("Tool {} should have valid JSON schema", tool.name));
            
            // Check for JSON schema structure
            assert!(schema_json.get("type").is_some(),
                "Tool {} schema should have 'type' field", tool.name);
        }
    }
}

/// Test STDIO transport integration
#[tokio::test]
async fn test_stdio_transport_integration() {
    use rmcp::transport::io::from_async_read;
    use futures::StreamExt;
    
    // Create test message for tools/list
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "tools/list",
        "id": 1
    });

    // Create a mock stdio buffer
    let request_str = format!("{}\n", request.to_string());
    let mut cursor = std::io::Cursor::new(request_str.as_bytes());
    
    // Parse the request
    let mut stream = from_async_read::<Value, _>(&mut cursor);
    let parsed = stream.next().await;
    
    assert!(parsed.is_some(), "Should parse JSON-RPC message");
    let message = parsed.unwrap();
    assert_eq!(message["method"], "tools/list");
}

/// Test E2E flow with mock STDIO transport
#[tokio::test]
async fn test_e2e_stdio_communication() {
    use rmcp::model::{ClientRequest, JsonRpcMessage};
    use serde_json::json;

    // Create duplex streams for testing
    let (client_stream, server_stream) = create_mock_stdio();
    let (server_read, server_write) = tokio::io::split(server_stream);
    let (client_read, client_write) = tokio::io::split(client_stream);

    // Create server
    let server = GolemMcpServerV2::new()
        .expect("Failed to create server");

    // Start server in background
    let server_handle = tokio::spawn(async move {
        let transport = (server_read, server_write);
        let _ = rmcp::serve_server(server, transport).await;
    });

    // Send initialize request from client side
    let init_request = json!({
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-06-18",
            "capabilities": {
                "tools": {}
            },
            "clientInfo": {
                "name": "test-client",
                "version": "0.1.0"
            }
        },
        "id": 1
    });

    // Write request to client stream
    let mut writer = client_write;
    writer.write_all(format!("{}\n", init_request).as_bytes()).await
        .expect("Failed to write request");
    writer.flush().await.expect("Failed to flush");

    // Give server time to process
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Cleanup
    server_handle.abort();
}

/// Test tool invocation through the router
#[tokio::test]
async fn test_tool_invocation() {
    use rmcp::model::{CallToolRequest, ToolCall};
    use rmcp::handler::RequestContext;

    let server = GolemMcpServerV2::new()
        .expect("Failed to create server");

    // Test list_components tool invocation
    let tool_call = ToolCall {
        name: "list_components".to_string(),
        parameters: None,
    };

    let request = CallToolRequest {
        params: tool_call,
        _meta: Default::default(),
    };

    let context = RequestContext {
        sender: None,
        meta: Default::default(),
    };

    // Call the tool through the handler
    let result = server.call_tool(request, context).await;
    assert!(result.is_ok(), 
        "Tool invocation should succeed: {:?}", result.err());

    let tool_result = result.unwrap();
    assert!(!tool_result.content.is_empty(), 
        "Tool should return content");
}

/// Test that tool router is properly initialized
#[tokio::test]
async fn test_tool_router_initialization() {
    let server = GolemMcpServerV2::new()
        .expect("Failed to create server");

    // Check that tool_router is not empty
    assert!(!server.tool_router.map.is_empty(),
        "Tool router should have registered tools");

    // Verify transparent_when_not_found is set correctly
    assert!(!server.tool_router.transparent_when_not_found,
        "Tool router should not be transparent when tool not found");
}

/// Test tool parameter validation
#[tokio::test]
async fn test_tool_parameter_validation() {
    use rmcp::model::{CallToolRequest, ToolCall};
    use rmcp::handler::RequestContext;
    use serde_json::json;

    let server = GolemMcpServerV2::new()
        .expect("Failed to create server");

    // Test with invalid parameters for deploy_component
    let tool_call = ToolCall {
        name: "deploy_component".to_string(),
        parameters: Some(json!({
            // Missing required field 'wasm_path'
            "name": "test-component"
        })),
    };

    let request = CallToolRequest {
        params: tool_call,
        _meta: Default::default(),
    };

    let context = RequestContext {
        sender: None,
        meta: Default::default(),
    };

    // This should fail due to missing required parameter
    let result = server.call_tool(request, context).await;
    assert!(result.is_err(),
        "Tool invocation should fail with missing required parameter");
}