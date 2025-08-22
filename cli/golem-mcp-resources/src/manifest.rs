//! Golem manifest file resource handling
//!
//! Provides access to golem.yaml files and project configurations as MCP resources.

use crate::providers::ResourceProvider;
use golem_mcp_types::{error::{McpError, McpResult}, protocol::Resource};
use std::collections::HashMap;

/// Provider for Golem manifest files
#[derive(Debug, Clone)]
pub struct ManifestResourceProvider {
    /// Base directory to search for manifests
    base_dir: std::path::PathBuf,
}

#[allow(dead_code)]
impl ManifestResourceProvider {
    /// Get the base directory
    pub fn base_dir(&self) -> &std::path::Path {
        &self.base_dir
    }
}

impl ManifestResourceProvider {
    /// Create a new manifest resource provider
    #[must_use]
    pub fn new(base_dir: std::path::PathBuf) -> Self {
        Self { base_dir }
    }
}

impl ResourceProvider for ManifestResourceProvider {
    fn list_resources(&self) -> McpResult<Vec<Resource>> {
        let mut resources = Vec::new();
        
        // 1. Find Golem manifest files (Enhanced but simple from Compare.MD lines 220-257)
        let manifests = self.find_manifest_files(3)?; // Max depth 3
        for manifest_path in manifests {
            let relative_path = manifest_path.strip_prefix(&self.base_dir)
                .unwrap_or(&manifest_path);
            
            resources.push(Resource {
                uri: format!("golem://manifest/{}", relative_path.display()),
                name: manifest_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("manifest")
                    .to_string(),
                description: Some("Golem application manifest".to_string()),
                mime_type: Some(self.get_manifest_mime_type(&manifest_path)),
                metadata: HashMap::new(),
            });
        }
        
        // 2. Find component WASM files
        let wasm_files = self.find_wasm_files(2)?; // Max depth 2
        for wasm_path in wasm_files {
            let relative_path = wasm_path.strip_prefix(&self.base_dir)
                .unwrap_or(&wasm_path);
                
            resources.push(Resource {
                uri: format!("golem://component/{}", relative_path.display()),
                name: wasm_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("component.wasm")
                    .to_string(),
                description: Some("WebAssembly component".to_string()),
                mime_type: Some("application/wasm".to_string()),
                metadata: HashMap::new(),
            });
        }
        
        // 3. Project structure metadata
        if let Ok(project_info) = self.extract_project_metadata() {
            resources.push(Resource {
                uri: "golem://project/metadata".to_string(),
                name: "Project Information".to_string(),
                description: Some(format!("Project: {}", project_info.name)),
                mime_type: Some("application/json".to_string()),
                metadata: HashMap::new(),
            });
        }
        
        Ok(resources)
    }

    fn read_resource(&self, uri: &str) -> McpResult<Vec<u8>> {
        if let Some(path) = uri.strip_prefix("golem://manifest/") {
            let full_path = self.base_dir.join(path);
            std::fs::read(&full_path)
                .map_err(|e| McpError::internal(
                    format!("Failed to read manifest {}: {}", path, e)
                ))
        } else if let Some(path) = uri.strip_prefix("golem://component/") {
            let full_path = self.base_dir.join(path);
            std::fs::read(&full_path)
                .map_err(|e| McpError::internal(
                    format!("Failed to read component {}: {}", path, e)
                ))
        } else if uri == "golem://project/metadata" {
            let metadata = self.extract_project_metadata()?;
            serde_json::to_vec(&metadata)
                .map_err(|e| McpError::internal(
                    format!("Failed to serialize project metadata: {}", e)
                ))
        } else {
            Err(McpError::invalid_request(
                format!("Unsupported URI: {}", uri)
            ))
        }
    }

    fn supports_uri(&self, uri: &str) -> bool {
        uri.starts_with("golem://manifest/") || 
        uri.starts_with("golem://component/") ||
        uri == "golem://project/metadata"
    }
}

/// Project metadata structure
#[derive(Debug, serde::Serialize)]
struct ProjectMetadata {
    name: String,
    #[serde(rename = "type")]
    project_type: String,
    manifests: Vec<String>,
    components: Vec<String>,
}

impl ManifestResourceProvider {
    /// Find Golem manifest files (golem.yaml, golem.toml, golem.json)
    /// Following Compare.MD strategy lines 259-262
    fn find_manifest_files(&self, max_depth: usize) -> McpResult<Vec<std::path::PathBuf>> {
        let mut manifests = Vec::new();
        
        if let Ok(entries) = walkdir::WalkDir::new(&self.base_dir)
            .max_depth(max_depth)
            .into_iter()
            .collect::<Result<Vec<_>, _>>() {
            
            for entry in entries {
                let path = entry.path();
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if matches!(filename, "golem.yaml" | "golem.yml" | "golem.toml" | "golem.json") {
                        manifests.push(path.to_path_buf());
                    }
                }
            }
        }
        
        Ok(manifests)
    }
    
    /// Find WebAssembly component files
    fn find_wasm_files(&self, max_depth: usize) -> McpResult<Vec<std::path::PathBuf>> {
        let mut wasm_files = Vec::new();
        
        if let Ok(entries) = walkdir::WalkDir::new(&self.base_dir)
            .max_depth(max_depth)
            .into_iter()
            .collect::<Result<Vec<_>, _>>() {
            
            for entry in entries {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if ext == "wasm" {
                        wasm_files.push(path.to_path_buf());
                    }
                }
            }
        }
        
        Ok(wasm_files)
    }
    
    /// Get MIME type for manifest files
    fn get_manifest_mime_type(&self, path: &std::path::Path) -> String {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext {
                "yaml" | "yml" => "application/yaml".to_string(),
                "toml" => "application/toml".to_string(), 
                "json" => "application/json".to_string(),
                _ => "text/plain".to_string(),
            }
        } else {
            "text/plain".to_string()
        }
    }
    
    /// Extract project metadata for resource exposure
    /// Following Compare.MD strategy lines 246-257
    fn extract_project_metadata(&self) -> McpResult<ProjectMetadata> {
        let project_name = self.base_dir.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("golem-project")
            .to_string();
            
        let manifests = self.find_manifest_files(3)?
            .into_iter()
            .filter_map(|p| p.strip_prefix(&self.base_dir).ok().map(|rel| rel.display().to_string()))
            .collect();
            
        let components = self.find_wasm_files(2)?
            .into_iter()
            .filter_map(|p| p.strip_prefix(&self.base_dir).ok().map(|rel| rel.display().to_string()))
            .collect();
            
        Ok(ProjectMetadata {
            name: project_name,
            project_type: "golem".to_string(),
            manifests,
            components,
        })
    }
}
