//! Basic usage example for golem-mcp-types
//!
//! Demonstrates the fundamental MCP protocol types and their usage.

use golem_mcp_types::{
    error::McpError,
    protocol::{
        InitializeParams, JsonRpcId, JsonRpcRequest, McpCapabilities, McpClientInfo,
        McpImplementation, McpServerInfo,
    },
    serialization::{MessageBuilder, MessageSerializer},
};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Golem MCP Types - Basic Usage Example");
    println!("=====================================");

    // Create a sample JSON-RPC request
    let request = JsonRpcRequest::new(
        JsonRpcId::from("example-1"),
        "initialize".to_owned(),
        Some(serde_json::json!({
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": {
                "name": "example-client",
                "version": "1.0.0"
            }
        })),
    );
    println!("✅ Created JSON-RPC request: {}", request.method);

    // Serialize to JSON
    let message = golem_mcp_types::protocol::JsonRpcMessage::Request(request);
    let json = MessageSerializer::serialize_message(&message)?;
    println!("✅ Serialized to JSON ({} bytes)", json.len());

    // Deserialize back
    let deserialized = MessageSerializer::deserialize_message(&json)?;
    println!("✅ Deserialized successfully");

    // Create server info
    let server_info = McpServerInfo {
        name: "golem-mcp-server".to_owned(),
        version: "0.0.0".to_owned(),
        implementation: Some(McpImplementation {
            name: "golem-mcp-server".to_owned(),
            version: "0.0.0".to_owned(),
            metadata: HashMap::new(),
        }),
    };
    println!("✅ Created server info: {}", server_info.name);

    // Demonstrate error handling
    let error = McpError::authorization("Example authorization error".to_owned());
    println!(
        "✅ Created structured error (code: {}): {}",
        error.code(),
        error
    );

    // Create capabilities
    let capabilities = McpCapabilities {
        tools: Some(golem_mcp_types::protocol::ToolsCapability { list_changed: true }),
        resources: Some(golem_mcp_types::protocol::ResourcesCapability {
            subscribe: false,
            list_changed: true,
        }),
        prompts: None,
        logging: Some(golem_mcp_types::protocol::LoggingCapability {
            levels: vec![
                golem_mcp_types::protocol::LogLevel::Info,
                golem_mcp_types::protocol::LogLevel::Error,
            ],
        }),
    };
    println!(
        "✅ Created MCP capabilities with {} tool support",
        if capabilities.tools.is_some() {
            "with"
        } else {
            "without"
        }
    );

    println!("\n🎉 All basic MCP types working correctly!");
    println!("The Golem MCP server workspace is ready for development.");

    Ok(())
}
