//! Model Context Protocol (MCP) type definitions and serialization for Golem CLI
//!
//! This crate provides the foundational types and serialization support for implementing
//! an MCP server that exposes Golem CLI functionality to Model Context Protocol clients.
//!
//! # Architecture
//!
//! The MCP types are organized into several key modules:
//! - `protocol`: Core MCP protocol messages and types
//! - `transport`: Transport layer abstractions
//! - `error`: Comprehensive error handling
//! - `serialization`: JSON-RPC serialization utilities
//!
//! # MCP 2025-06-18 Specification Compliance
//!
//! This implementation follows the MCP 2025-06-18 specification, which removes
//! JSON-RPC batching support and standardizes on HTTP transport.

#![deny(clippy::all)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![deny(unsafe_code)]
#![deny(missing_docs)]

pub mod error;
pub mod protocol;
pub mod serialization;
pub mod transport;

pub use error::{McpError, McpResult};
pub use protocol::{
    JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, McpCapabilities, McpClientInfo,
    McpImplementation, McpServerInfo, McpVersion,
};
