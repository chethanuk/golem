//! Configuration file resource providers
//!
//! Provides access to CLI configuration and profile settings as MCP resources.

use crate::providers::ResourceProvider;
use golem_mcp_types::{error::McpResult, protocol::Resource};

/// Provider for CLI configuration files
#[derive(Debug, Clone)]
pub struct ConfigResourceProvider;

impl ResourceProvider for ConfigResourceProvider {
    fn list_resources(&self) -> McpResult<Vec<Resource>> {
        // Placeholder implementation
        Ok(vec![])
    }

    fn read_resource(&self, _uri: &str) -> McpResult<Vec<u8>> {
        // Placeholder implementation
        Ok(vec![])
    }

    fn supports_uri(&self, uri: &str) -> bool {
        uri.starts_with("golem://config/")
    }
}
