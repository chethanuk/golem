//! Model Context Protocol (MCP) server implementation for Golem CLI
//!
//! This crate provides a complete MCP server implementation that exposes Golem CLI
//! functionality through the Model Context Protocol specification (2025-06-18).
//!
//! # Architecture
//!
//! The server is built with the following key components:
//! - `server`: Core MCP server implementation with HTTP transport
//! - `handlers`: JSON-RPC method handlers for MCP protocol messages  
//! - `session`: Session management and client state tracking
//! - `config`: Server configuration and startup parameters
//! - `http`: HTTP transport implementation for MCP over HTTP
//!
//! # Usage
//!
//! ```rust,no_run
//! use golem_mcp_server::{McpServer, McpServerConfig};
//! use tokio;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = McpServerConfig::default();
//!     let server = McpServer::new(config).await?;
//!     server.serve().await?;
//!     Ok(())
//! }
//! ```
//!
//! # MCP 2025-06-18 Specification Compliance
//!
//! This implementation follows the MCP 2025-06-18 specification with:
//! - HTTP transport only (no stdio)
//! - No JSON-RPC batching support (removed in 2025-06-18)
//! - Full support for tools, resources, and prompts capabilities
//! - Proper error handling and client capability negotiation

#![deny(clippy::all)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![deny(unsafe_code)]
// TODO - @chethanuk Change this later on
#![warn(missing_docs)] 

pub mod config;
pub mod errors;
pub mod handlers;
pub mod http;
pub mod mcp_bridge;
pub mod progress;
pub mod rmcp_handler;
pub mod server;
pub mod session;

pub use config::McpServerConfig;
pub use errors::{GolemMcpError, GolemMcpResult, ErrorCategory, ErrorSeverity, ErrorBuilder};
pub use server::McpServer;

// Multi-transport support following Amazon Q pattern
pub use rmcp_handler::{
    GolemMcpServer, 
    GolemMcpTransport,
    serve_mcp_stdio, 
    serve_mcp_http,
    stdio_transport,
    http_transport,
    http_transport_with_host,
};

/// Re-export commonly used types from golem-mcp-types
pub use golem_mcp_types::{
    error::{McpError, McpResult},
    protocol::{JsonRpcMessage, LogLevel, McpCapabilities, McpServerInfo},
    transport::{HttpTransportConfig, Transport},
};
