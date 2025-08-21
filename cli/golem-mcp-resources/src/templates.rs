//! Template resource access
//!
//! Provides access to Golem templates and examples as MCP resources.

use crate::providers::ResourceProvider;
use golem_mcp_types::{error::McpResult, protocol::Resource};

/// Provider for Golem templates
#[derive(Debug, Clone)]
pub struct TemplateResourceProvider;

impl ResourceProvider for TemplateResourceProvider {
    fn list_resources(&self) -> McpResult<Vec<Resource>> {
        // Placeholder implementation
        Ok(vec![])
    }

    fn read_resource(&self, _uri: &str) -> McpResult<Vec<u8>> {
        // Placeholder implementation
        Ok(vec![])
    }

    fn supports_uri(&self, uri: &str) -> bool {
        uri.starts_with("golem://template/")
    }
}