//! MCP tools integration for Golem CLI commands
//!
//! This crate provides the MCP tools implementation that exposes Golem CLI
//! functionality as callable tools through the Model Context Protocol.
//!
//! # Architecture
//!
//! The tools are organized into several key modules:
//! - `registry`: Tool registration and discovery
//! - `executor`: Tool execution and process management
//! - `schema`: JSON schema definitions for tool inputs
//! - `tools`: Individual tool implementations for Golem commands
//!
//! # Tool Categories
//!
//! - **Component Tools**: Create, update, list, and manage Golem components
//! - **Worker Tools**: Create, invoke, and manage Golem workers  
//! - **Project Tools**: Manage Golem projects and configurations
//! - **Template Tools**: Work with Golem templates and scaffolding

#![deny(clippy::all)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![deny(unsafe_code)]
#![deny(missing_docs)]

pub mod executor;
pub mod registry;
pub mod schema;
pub mod tools;

pub use executor::ToolExecutor;
pub use registry::ToolRegistry;

/// Re-export commonly used types
pub use golem_mcp_types::{
    error::{McpError, McpResult},
    protocol::Tool,
};