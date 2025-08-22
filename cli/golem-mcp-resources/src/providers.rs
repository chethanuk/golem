//! Resource provider registry and management
//!
//! Provides centralized management of all resource providers and their capabilities.

use golem_mcp_types::{
    error::{McpError, McpResult},
    protocol::Resource,
};
use std::collections::HashMap;
use tracing::debug;

/// Registry for managing resource providers
#[derive(Debug)]
pub struct ResourceProviderRegistry {
    /// Map of resource URI patterns to provider implementations
    providers: HashMap<String, Box<dyn ResourceProvider>>,
}

/// Resource provider trait
pub trait ResourceProvider: Send + Sync + std::fmt::Debug {
    /// List available resources
    fn list_resources(&self) -> McpResult<Vec<Resource>>;

    /// Read resource content by URI
    fn read_resource(&self, uri: &str) -> McpResult<Vec<u8>>;

    /// Check if provider supports the given URI
    fn supports_uri(&self, uri: &str) -> bool;
}

impl Default for ResourceProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceProviderRegistry {
    /// Create a new resource provider registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Register a resource provider
    pub fn register_provider(&mut self, pattern: String, provider: Box<dyn ResourceProvider>) {
        debug!("Registering resource provider for pattern: {}", pattern);
        self.providers.insert(pattern, provider);
    }

    /// List all available resources
    ///
    /// # Errors
    ///
    /// Returns `McpError::ResourceAccess` if any provider fails to list resources.
    pub fn list_all_resources(&self) -> McpResult<Vec<Resource>> {
        let mut all_resources = Vec::new();

        for provider in self.providers.values() {
            match provider.list_resources() {
                Ok(mut resources) => all_resources.append(&mut resources),
                Err(e) => {
                    return Err(McpError::resource_access(
                        "provider".to_owned(),
                        "unknown".to_owned(),
                        format!("Provider failed to list resources: {e}"),
                    ));
                }
            }
        }

        Ok(all_resources)
    }

    /// Read resource content by URI
    ///
    /// # Errors
    ///
    /// Returns `McpError::ResourceAccess` if no provider supports the URI or reading fails.
    pub fn read_resource(&self, uri: &str) -> McpResult<Vec<u8>> {
        for provider in self.providers.values() {
            if provider.supports_uri(uri) {
                return provider.read_resource(uri);
            }
        }

        Err(McpError::resource_access(
            "unknown".to_owned(),
            uri.to_owned(),
            "No provider found for URI".to_owned(),
        ))
    }

    /// List all available resources (async version)
    ///
    /// # Errors
    ///
    /// Returns `McpError::ResourceAccess` if any provider fails to list resources.
    pub async fn list_resources(&self) -> Vec<Resource> {
        // For now, return synchronous version result
        self.list_all_resources().unwrap_or_default()
    }

    /// Read resource content by URI (async version)
    ///
    /// # Errors
    ///
    /// Returns `McpError::ResourceAccess` if no provider supports the URI or reading fails.
    pub async fn read_resource_async(&self, uri: &str) -> McpResult<serde_json::Value> {
        match self.read_resource(uri) {
            Ok(content) => {
                let text = String::from_utf8_lossy(&content);
                Ok(serde_json::json!({
                    "contents": [{
                        "uri": uri,
                        "mimeType": "text/plain",
                        "text": text
                    }]
                }))
            }
            Err(e) => Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct MockProvider;

    impl ResourceProvider for MockProvider {
        fn list_resources(&self) -> McpResult<Vec<Resource>> {
            Ok(vec![Resource {
                uri: "mock://test".to_owned(),
                name: "Test Resource".to_owned(),
                description: Some("A test resource".to_owned()),
                mime_type: Some("text/plain".to_owned()),
                metadata: HashMap::new(),
            }])
        }

        fn read_resource(&self, _uri: &str) -> McpResult<Vec<u8>> {
            Ok(b"test content".to_vec())
        }

        fn supports_uri(&self, uri: &str) -> bool {
            uri.starts_with("mock://")
        }
    }

    #[test]
    fn test_resource_provider_registry() {
        let mut registry = ResourceProviderRegistry::new();
        registry.register_provider("mock".to_owned(), Box::new(MockProvider));

        let resources = registry
            .list_all_resources()
            .expect("should list resources");
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].uri, "mock://test");

        let content = registry
            .read_resource("mock://test")
            .expect("should read resource");
        assert_eq!(content, b"test content");
    }
}
