//! MCP resource providers for Golem manifests and configurations
//!
//! This crate implements MCP resource providers that expose Golem project
//! artifacts and configurations as accessible resources through the
//! Model Context Protocol.
//!
//! # Architecture
//!
//! The resources are organized into several key modules:
//! - `providers`: Resource provider implementations
//! - `manifest`: Golem manifest file resource handling
//! - `config`: Configuration file resource providers  
//! - `templates`: Template resource access
//! - `watcher`: File system watching for resource changes
//!
//! # Resource Types
//!
//! - **Manifest Resources**: golem.yaml files and project configurations
//! - **Template Resources**: Available Golem templates and examples
//! - **Config Resources**: CLI configuration and profile settings
//! - **Schema Resources**: JSON schemas for validation and tooling

#![deny(clippy::all)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![deny(unsafe_code)]
#![deny(missing_docs)]

pub mod config;
pub mod manifest;
pub mod providers;
pub mod schema;
pub mod templates;
pub mod watcher;

pub use providers::ResourceProviderRegistry;

/// Re-export commonly used types
pub use golem_mcp_types::{
    error::{McpError, McpResult},
    protocol::Resource,
};