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

pub mod api_client;
pub mod api_client_extensions;
pub mod auth;
pub mod client_manager;
pub mod config;
pub mod errors;
pub mod handlers;
pub mod http;
pub mod progress;
// V1 removed - using V2 only
pub mod rmcp_handler_v2;
pub mod server;
pub mod session;

pub use config::McpServerConfig;
pub use errors::{GolemMcpError, GolemMcpResult, ErrorCategory, ErrorSeverity, ErrorBuilder, TransportType};
pub use server::McpServer;

// Export V2 as the primary implementation
pub use rmcp_handler_v2::{GolemMcpServerV2, GolemMcpTransport};
