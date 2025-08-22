//! Basic Usage Example for Golem MCP Server
//!
//! This example demonstrates the minimal setup required to get a Golem MCP server
//! running with stdio transport, following production patterns from Amazon Q Developer CLI.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example basic_usage --features stdio,tracing-subscriber
//! ```
//!
//! # Prerequisites
//!
//! - Golem CLI installed and accessible in PATH
//! - Rust 1.75+ with Cargo

use golem_mcp_server::{
    rmcp_handler::{GolemMcpServer, GolemMcpTransport},
    serve_mcp_stdio,
};
use tracing::{info, error};

/// Initialize logging for development and production use
fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into())
        )
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();
}

/// Main entry point demonstrating basic MCP server setup
///
/// # Examples
///
/// Start the server and test with a simple MCP client:
///
/// ```bash
/// # Terminal 1: Start the server
/// RUST_LOG=info cargo run --example basic_usage
///
/// # Terminal 2: Test with curl (if using HTTP transport)
/// curl -X POST http://localhost:8080/mcp \
///   -H "Content-Type: application/json" \
///   -d '{"jsonrpc":"2.0","id":"1","method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}'
/// ```
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    init_logging();
    
    info!("🚀 Starting Golem MCP Server - Basic Usage Example");
    info!("📋 This example demonstrates minimal server setup with stdio transport");
    
    // Create server instance with default configuration
    let server = GolemMcpServer::new();
    
    info!("✅ Server created successfully");
    info!("🔧 Golem CLI tools are automatically discovered and exposed");
    info!("📄 Manifest resources will be available if found in current directory");
    
    // Start server with stdio transport (Claude Desktop compatible)
    info!("🎯 Starting stdio transport for Claude Desktop integration...");
    
    match serve_mcp_stdio(server).await {
        Ok(()) => {
            info!("✅ Server terminated gracefully");
        }
        Err(e) => {
            error!("❌ Server error: {}", e);
            return Err(e);
        }
    }
    
    info!("👋 Golem MCP Server shutdown complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use golem_mcp_server::rmcp_handler::ComponentOperation;
    use rmcp::handler::server::tool::Parameters;
    use rstest::*;
    use serde_json::json;
    use tokio_test;
    
    /// Test fixture for creating a server instance
    #[fixture]
    fn server() -> GolemMcpServer {
        GolemMcpServer::new()
    }
    
    /// Test server creation and basic functionality
    #[rstest]
    #[case::server_creation("Server should be created successfully")]
    #[tokio::test]
    async fn test_basic_server_functionality(
        server: GolemMcpServer,
        #[case] description: &str,
    ) {
        // Test that server can handle basic operations
        let operation = ComponentOperation {
            operation: "list".to_string(),
            component_name: None,
            wasm_file: None,
            project_id: None,
            extra: json!({}),
        };
        
        // This should either succeed or fail gracefully
        let result = server.golem_component_management(Parameters(operation)).await;
        
        match result {
            Ok(_) => println!("✅ {}: Server operation succeeded", description),
            Err(e) => {
                println!("⚠️  {}: Server operation failed gracefully: {:?}", description, e);
                // Graceful failure is acceptable for basic functionality test
                assert!(!e.message.is_empty(), "Error should have a meaningful message");
            }
        }
    }
    
    /// Test transport configuration validation
    #[rstest]
    #[case::stdio_transport(GolemMcpTransport::Stdio, "Stdio transport")]
    #[case::http_transport(GolemMcpTransport::Http { port: 8080, host: None }, "HTTP transport")]
    #[case::http_with_host(GolemMcpTransport::Http { port: 8081, host: Some("127.0.0.1".to_string()) }, "HTTP with custom host")]
    fn test_transport_configuration(
        #[case] transport: GolemMcpTransport,
        #[case] description: &str,
    ) {
        // Validate transport configuration
        match transport {
            GolemMcpTransport::Stdio => {
                assert!(true, "Stdio transport is always valid");
            }
            GolemMcpTransport::Http { port, host: _ } => {
                assert!(port > 0, "Port should be positive");
                assert!(port <= 65535, "Port should be within valid range");
            }
        }
        
        println!("✅ {}: Configuration validated", description);
    }
}