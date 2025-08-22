//! JSON schema resource providers
//!
//! Provides access to JSON schemas for validation and tooling as MCP resources.

use crate::providers::ResourceProvider;
use golem_mcp_types::{error::McpResult, protocol::Resource};

/// Provider for JSON schema resources
#[derive(Debug, Clone)]
pub struct SchemaResourceProvider;

impl ResourceProvider for SchemaResourceProvider {
    fn list_resources(&self) -> McpResult<Vec<Resource>> {
        // Placeholder implementation
        Ok(vec![])
    }

    fn read_resource(&self, _uri: &str) -> McpResult<Vec<u8>> {
        // Placeholder implementation
        Ok(vec![])
    }

    fn supports_uri(&self, uri: &str) -> bool {
        uri.starts_with("golem://schema/")
    }
}
