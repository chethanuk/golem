// Copyright 2024-2025 Golem Cloud
//
// Licensed under the Golem Source License v1.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://license.golem.cloud/LICENSE
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Authentication context management for MCP server
//!
//! This module handles authentication context including account ID retrieval,
//! which is necessary for cloud API calls.

use crate::errors::GolemMcpError as McpError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Authentication context containing account information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthContext {
    /// The authentication token
    pub token: Uuid,
    /// The account ID associated with this token
    pub account_id: String,
    /// Optional email address
    pub email: Option<String>,
    /// Optional default project ID
    pub project_id: Option<String>,
}

impl AuthContext {
    /// Convert to bearer token for HTTP headers
    pub fn to_bearer(&self) -> String {
        format!("Bearer {}", self.token)
    }
}

/// Manages authentication context for the MCP server
pub struct AuthManager {
    /// Cached authentication context
    auth_context: Option<AuthContext>,
    /// Path to config file
    config_path: Option<PathBuf>,
}

impl AuthManager {
    /// Create a new auth manager
    pub fn new() -> Self {
        Self {
            auth_context: None,
            config_path: None,
        }
    }
    
    /// Get or initialize auth context
    pub async fn get_or_init(&mut self, token: Uuid) -> Result<AuthContext, McpError> {
        // Return cached context if available
        if let Some(context) = &self.auth_context {
            if context.token == token {
                return Ok(context.clone());
            }
        }
        
        // Try to read from config file
        if let Ok(context) = self.read_from_config(token).await {
            self.auth_context = Some(context.clone());
            return Ok(context);
        }
        
        // Try environment variables as fallback
        if let Ok(context) = self.read_from_env(token) {
            self.auth_context = Some(context.clone());
            return Ok(context);
        }
        
        // If all else fails, return error
        Err(McpError::ConfigurationError {
            message: "Cannot determine account ID. Please ensure golem-cli is configured or set GOLEM_ACCOUNT_ID environment variable.".to_string(),
            config_key: Some("account_id".to_string()),
            expected_format: Some("UUID or account identifier".to_string()),
        })
    }
    
    /// Read auth context from golem config file
    async fn read_from_config(&mut self, token: Uuid) -> Result<AuthContext, McpError> {
        // Find config directory
        let config_dir = dirs::config_dir()
            .ok_or_else(|| McpError::ConfigurationError {
                message: "Cannot find config directory".to_string(),
                config_key: None,
                expected_format: None,
            })?;
            
        let config_path = config_dir.join("golem").join("config.toml");
        
        if !config_path.exists() {
            return Err(McpError::ConfigurationError {
                message: format!("Config file not found at {:?}", config_path),
                config_key: None,
                expected_format: None,
            });
        }
        
        // Read and parse config file
        let config_content = std::fs::read_to_string(&config_path)
            .map_err(|e| McpError::ConfigurationError {
                message: format!("Failed to read config file: {}", e),
                config_key: None,
                expected_format: None,
            })?;
            
        // Parse TOML
        let config: GolemConfig = toml::from_str(&config_content)
            .map_err(|e| McpError::ConfigurationError {
                message: format!("Failed to parse config file: {}", e),
                config_key: None,
                expected_format: None,
            })?;
            
        // Find the profile with matching token
        for profile in config.profiles.values() {
            if let Some(auth) = &profile.auth {
                if let Some(oauth2) = &auth.oauth2 {
                    if let Some(data) = &oauth2.data {
                        // Check if token matches
                        if data.id == token {
                            return Ok(AuthContext {
                                token,
                                account_id: data.account_id.clone(),
                                email: Some(data.email.clone()),
                                project_id: profile.default_project_id.clone(),
                            });
                        }
                    }
                }
            }
        }
        
        Err(McpError::ConfigurationError {
            message: "No matching authentication found in config".to_string(),
            config_key: Some("auth".to_string()),
            expected_format: None,
        })
    }
    
    /// Read auth context from environment variables
    fn read_from_env(&self, token: Uuid) -> Result<AuthContext, McpError> {
        let account_id = std::env::var("GOLEM_ACCOUNT_ID")
            .map_err(|_| McpError::ConfigurationError {
                message: "GOLEM_ACCOUNT_ID environment variable not set".to_string(),
                config_key: Some("GOLEM_ACCOUNT_ID".to_string()),
                expected_format: Some("Account UUID".to_string()),
            })?;
            
        let email = std::env::var("GOLEM_ACCOUNT_EMAIL").ok();
        let project_id = std::env::var("GOLEM_PROJECT_ID").ok();
        
        Ok(AuthContext {
            token,
            account_id,
            email,
            project_id,
        })
    }
}

// Config file structures (simplified from golem-cli)
#[derive(Debug, Deserialize)]
struct GolemConfig {
    #[serde(default)]
    profiles: std::collections::HashMap<String, ProfileConfig>,
}

#[derive(Debug, Deserialize)]
struct ProfileConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    auth: Option<AuthConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_project_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AuthConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    oauth2: Option<OAuth2Config>,
}

#[derive(Debug, Deserialize)]
struct OAuth2Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<OAuth2Data>,
}

#[derive(Debug, Deserialize)]
struct OAuth2Data {
    id: Uuid,
    account_id: String,
    email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_auth_context_from_env() {
        std::env::set_var("GOLEM_ACCOUNT_ID", "test-account-123");
        std::env::set_var("GOLEM_ACCOUNT_EMAIL", "test@example.com");
        
        let mut manager = AuthManager::new();
        let token = Uuid::new_v4();
        
        // This should read from env vars
        let context = manager.read_from_env(token).unwrap();
        
        assert_eq!(context.account_id, "test-account-123");
        assert_eq!(context.email, Some("test@example.com".to_string()));
        
        // Clean up
        std::env::remove_var("GOLEM_ACCOUNT_ID");
        std::env::remove_var("GOLEM_ACCOUNT_EMAIL");
    }
    
    #[test]
    fn test_bearer_token_format() {
        let token = Uuid::new_v4();
        let context = AuthContext {
            token,
            account_id: "test-123".to_string(),
            email: None,
            project_id: None,
        };
        
        let bearer = context.to_bearer();
        assert!(bearer.starts_with("Bearer "));
        assert!(bearer.contains(&token.to_string()));
    }
}