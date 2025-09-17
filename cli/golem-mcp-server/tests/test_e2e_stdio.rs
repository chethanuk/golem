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

//! E2E testing patterns for MCP servers with STDIO transport
//!
//! This module demonstrates best practices for end-to-end testing
//! of MCP servers using the STDIO transport mechanism.

use futures::{SinkExt, StreamExt};
use rmcp::{
    model::{
        ClientRequest, ClientResult, JsonRpcMessage,
        InitializeRequest, InitializeResult, ClientCapabilities,
        ClientInfo, ListToolsRequest,
    },
    transport::{
        io::{from_async_read, from_async_write},
        sink_stream::SinkStreamTransport,
    },
    Service, ServiceExt,
};
use serde_json::{json, Value};
use std::io::Cursor;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, DuplexStream};

/// Create a test harness for STDIO testing
pub struct StdioTestHarness {
    client_to_server: DuplexStream,
    server_to_client: DuplexStream,
}

impl StdioTestHarness {
    /// Create a new test harness with duplex streams
    pub fn new() -> Self {
        let (client_to_server, server_from_client) = tokio::io::duplex(4096);
        let (server_to_client, client_from_server) = tokio::io::duplex(4096);
        
        Self {
            client_to_server,
            server_to_client,
        }
    }

    /// Get the server-side transport
    pub fn server_transport(self) -> (DuplexStream, DuplexStream) {
        (self.client_to_server, self.server_to_client)
    }

    /// Get the client-side transport  
    pub fn client_transport(self) -> (DuplexStream, DuplexStream) {
        (self.server_to_client, self.client_to_server)
    }
}

/// Test helper for sending JSON-RPC messages
async fn send_jsonrpc_message(
    writer: &mut (impl AsyncWriteExt + Unpin),
    message: Value,
) -> Result<(), std::io::Error> {
    let message_str = format!("{}\n", message.to_string());
    writer.write_all(message_str.as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}

/// Test helper for receiving JSON-RPC messages
async fn receive_jsonrpc_message(
    reader: &mut BufReader<impl AsyncBufReadExt + Unpin>,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    let message: Value = serde_json::from_str(&line)?;
    Ok(message)
}

/// E2E test for complete MCP session over STDIO
#[tokio::test]
async fn test_complete_mcp_session() {
    use golem_mcp_server::GolemMcpServerV2;
    
    // Create test harness
    let harness = StdioTestHarness::new();
    let (server_read, server_write) = harness.server_transport();
    
    // Create and start server
    let server = GolemMcpServerV2::new()
        .expect("Failed to create server");
    
    let server_handle = tokio::spawn(async move {
        let transport = (server_read, server_write);
        let _ = rmcp::serve_server(server, transport).await;
    });

    // Simulate client connection
    let harness = StdioTestHarness::new();
    let (mut client_write, client_read) = harness.client_transport();
    let mut client_reader = BufReader::new(client_read);

    // Step 1: Send initialize request
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

    send_jsonrpc_message(&mut client_write, init_request).await
        .expect("Failed to send initialize");

    // Step 2: Receive initialize response
    let init_response = receive_jsonrpc_message(&mut client_reader).await
        .expect("Failed to receive initialize response");
    
    assert_eq!(init_response["id"], 1);
    assert!(init_response["result"].is_object());
    assert!(init_response["result"]["capabilities"]["tools"].as_bool().unwrap_or(false));

    // Step 3: Send initialized notification
    let initialized_notif = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });

    send_jsonrpc_message(&mut client_write, initialized_notif).await
        .expect("Failed to send initialized notification");

    // Step 4: List available tools
    let list_tools_request = json!({
        "jsonrpc": "2.0",
        "method": "tools/list",
        "id": 2
    });

    send_jsonrpc_message(&mut client_write, list_tools_request).await
        .expect("Failed to send tools/list");

    // Step 5: Receive tools list
    let tools_response = receive_jsonrpc_message(&mut client_reader).await
        .expect("Failed to receive tools list");
    
    assert_eq!(tools_response["id"], 2);
    assert!(tools_response["result"]["tools"].is_array());
    
    let tools = tools_response["result"]["tools"].as_array().unwrap();
    assert!(!tools.is_empty(), "Should have registered tools");

    // Step 6: Call a tool
    let call_tool_request = json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "list_components",
            "parameters": {}
        },
        "id": 3
    });

    send_jsonrpc_message(&mut client_write, call_tool_request).await
        .expect("Failed to send tool call");

    // Step 7: Receive tool response
    let tool_response = receive_jsonrpc_message(&mut client_reader).await
        .expect("Failed to receive tool response");
    
    assert_eq!(tool_response["id"], 3);
    assert!(tool_response["result"].is_object());

    // Cleanup
    server_handle.abort();
}

/// Test STDIO message codec
#[tokio::test]
async fn test_stdio_message_codec() {
    use rmcp::transport::io::{from_async_read, from_async_write};
    
    // Test encoding messages
    let messages = vec![
        json!({
            "jsonrpc": "2.0",
            "method": "test",
            "id": 1
        }),
        json!({
            "jsonrpc": "2.0",
            "method": "test2",
            "params": {"key": "value"},
            "id": 2
        }),
    ];

    let mut buffer = Vec::new();
    let mut writer = from_async_write(&mut buffer);

    // Write messages
    for msg in &messages {
        writer.send(msg.clone()).await
            .expect("Failed to write message");
    }
    writer.close().await.expect("Failed to close writer");

    // Read messages back
    let mut cursor = Cursor::new(&buffer);
    let mut reader = from_async_read::<Value, _>(&mut cursor);

    for expected_msg in messages {
        let msg = reader.next().await
            .expect("Failed to read message");
        assert_eq!(msg, expected_msg);
    }
}

/// Test concurrent tool calls
#[tokio::test]
async fn test_concurrent_tool_calls() {
    use golem_mcp_server::GolemMcpServerV2;
    use std::sync::Arc;
    
    let server = Arc::new(GolemMcpServerV2::new()
        .expect("Failed to create server"));
    
    // Spawn multiple concurrent tool calls
    let mut handles = vec![];
    
    for i in 0..5 {
        let server_clone = Arc::clone(&server);
        let handle = tokio::spawn(async move {
            let request = rmcp::model::ListToolsRequest::default();
            let context = rmcp::handler::RequestContext {
                sender: None,
                meta: Default::default(),
            };
            
            server_clone.list_tools(request, context).await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        let result = handle.await
            .expect("Task failed")
            .expect("list_tools failed");
        assert!(!result.tools.is_empty());
    }
}

/// Test error handling for invalid tool calls
#[tokio::test]
async fn test_invalid_tool_call() {
    use golem_mcp_server::GolemMcpServerV2;
    use rmcp::model::{CallToolRequest, ToolCall};
    
    let server = GolemMcpServerV2::new()
        .expect("Failed to create server");
    
    // Call non-existent tool
    let tool_call = ToolCall {
        name: "non_existent_tool".to_string(),
        parameters: None,
    };

    let request = CallToolRequest {
        params: tool_call,
        _meta: Default::default(),
    };

    let context = rmcp::handler::RequestContext {
        sender: None,  
        meta: Default::default(),
    };

    let result = server.call_tool(request, context).await;
    assert!(result.is_err(), "Should fail for non-existent tool");
    
    if let Err(e) = result {
        assert!(e.message.contains("not found") || e.message.contains("not implemented"),
            "Error should indicate tool not found");
    }
}

/// Test STDIO transport with large messages
#[tokio::test]
async fn test_large_message_handling() {
    use rmcp::transport::io::{from_async_read, from_async_write};
    
    // Create a large payload
    let large_data: Vec<String> = (0..1000)
        .map(|i| format!("item_{}", i))
        .collect();
    
    let large_message = json!({
        "jsonrpc": "2.0",
        "method": "test_large",
        "params": {
            "data": large_data
        },
        "id": 1
    });

    let mut buffer = Vec::new();
    let mut writer = from_async_write(&mut buffer);
    
    // Write large message
    writer.send(large_message.clone()).await
        .expect("Failed to write large message");
    writer.close().await.expect("Failed to close");

    // Read it back
    let mut cursor = Cursor::new(&buffer);
    let mut reader = from_async_read::<Value, _>(&mut cursor);
    
    let received = reader.next().await
        .expect("Failed to read large message");
    
    assert_eq!(received, large_message);
    assert_eq!(
        received["params"]["data"].as_array().unwrap().len(),
        1000
    );
}