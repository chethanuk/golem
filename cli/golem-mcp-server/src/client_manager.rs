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

//! Client manager for direct API access bypassing Clap
//! 
//! This module implements the "bypassing Clap" pattern by creating API clients
//! directly from configuration without going through CLI command parsing.
//! This follows best practices from Microsoft VSCode MCP, OpenAI agents, and others.

use golem_client::api::{
    AccountClientLive as AccountClient,
    ApiCertificateClientLive as ApiCertificateClient,
    ApiDefinitionClientLive as ApiDefinitionClient,
    ApiDeploymentClientLive as ApiDeploymentClient,
    ApiDomainClientLive as ApiDomainClient,
    ApiSecurityClientLive as ApiSecurityClient,
    ComponentClientLive as ComponentClient,
    GrantClientLive as GrantClient,
    HealthCheckClientLive as HealthCheckClient,
    LimitsClientLive as LimitsClient,
    LoginClientLive as LoginClient,
    PluginClientLive as PluginClient,
    ProjectClientLive as ProjectClient,
    ProjectGrantClientLive as ProjectGrantClient,
    ProjectPolicyClientLive as ProjectPolicyClient,
    TokenClientLive as TokenClient,
    WorkerClientLive as WorkerClient,
};
use golem_client::{Context as ClientContext, Security};
use url::Url;
use crate::errors::{GolemMcpError as McpError, GolemMcpResult};

/// All Golem API clients available for cloud mode
/// This bypasses Clap entirely and provides direct API access
pub struct CloudClients {
    pub account: AccountClient,
    pub api_certificate: ApiCertificateClient,
    pub api_definition: ApiDefinitionClient,
    pub api_deployment: ApiDeploymentClient,
    pub api_domain: ApiDomainClient,
    pub api_security: ApiSecurityClient,
    pub component: ComponentClient,
    pub grant: GrantClient,
    pub health_check: HealthCheckClient,
    pub limits: LimitsClient,
    pub login: LoginClient,
    pub plugin: PluginClient,
    pub project: ProjectClient,
    pub project_grant: ProjectGrantClient,
    pub project_policy: ProjectPolicyClient,
    pub token: TokenClient,
    pub worker: WorkerClient,
}

impl CloudClients {
    /// Create cloud clients from URL and token
    /// This is the core of "bypassing Clap" - direct API client creation
    pub fn new(base_url: &str, token: Option<String>) -> Result<Self, McpError> {
        let url = Url::parse(base_url)
            .map_err(|e| McpError::ConfigurationError {
                message: format!("Invalid cloud URL: {}", e),
                config_key: Some("cloud_url".to_string()),
                expected_format: Some("https://api.example.com".to_string()),
            })?;
            
        let context = || ClientContext {
            base_url: url.clone(),
            client: reqwest::Client::new(),
            security_token: if let Some(token_str) = &token {
                Security::Bearer(token_str.clone())
            } else {
                Security::Empty
            },
        };
        
        Ok(Self {
            account: AccountClient { context: context() },
            api_certificate: ApiCertificateClient { context: context() },
            api_definition: ApiDefinitionClient { context: context() },
            api_deployment: ApiDeploymentClient { context: context() },
            api_domain: ApiDomainClient { context: context() },
            api_security: ApiSecurityClient { context: context() },
            component: ComponentClient { context: context() },
            grant: GrantClient { context: context() },
            health_check: HealthCheckClient { context: context() },
            limits: LimitsClient { context: context() },
            login: LoginClient { context: context() },
            plugin: PluginClient { context: context() },
            project: ProjectClient { context: context() },
            project_grant: ProjectGrantClient { context: context() },
            project_policy: ProjectPolicyClient { context: context() },
            token: TokenClient { context: context() },
            worker: WorkerClient { context: context() },
        })
    }
}

/// Limited clients for OSS/dev mode
pub struct OssClients {
    pub component: ComponentClient,
    pub worker: WorkerClient,
    pub health_check: HealthCheckClient,
}

impl OssClients {
    /// Create OSS clients from URL
    pub fn new(base_url: &str) -> Result<Self, McpError> {
        let url = Url::parse(base_url)
            .map_err(|e| McpError::ConfigurationError {
                message: format!("Invalid OSS URL: {}", e),
                config_key: Some("oss_url".to_string()),
                expected_format: Some("http://localhost:9881".to_string()),
            })?;
            
        let context = || ClientContext {
            base_url: url.clone(),
            client: reqwest::Client::new(),
            security_token: Security::Empty,
        };
        
        Ok(Self {
            component: ComponentClient { context: context() },
            worker: WorkerClient { context: context() },
            health_check: HealthCheckClient { context: context() },
        })
    }
}

/// Client manager that handles profile-based client initialization
/// This is the main interface for bypassing Clap and using direct API calls
pub struct ClientManager {
    cloud_clients: Option<CloudClients>,
    oss_clients: Option<OssClients>,
    current_mode: ClientMode,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ClientMode {
    Cloud { url: String, token: Option<String> },
    Oss { url: String },
    Uninitialized,
}

impl ClientManager {
    pub fn new() -> Self {
        Self {
            cloud_clients: None,
            oss_clients: None,
            current_mode: ClientMode::Uninitialized,
        }
    }
    
    /// Initialize for cloud mode
    pub fn init_cloud(&mut self, url: &str, token: Option<String>) -> Result<(), McpError> {
        let new_mode = ClientMode::Cloud { 
            url: url.to_string(), 
            token: token.clone() 
        };
        
        // Only reinitialize if mode changed
        if self.current_mode != new_mode {
            self.cloud_clients = Some(CloudClients::new(url, token)?);
            self.oss_clients = None;
            self.current_mode = new_mode;
        }
        
        Ok(())
    }
    
    /// Initialize for OSS mode
    pub fn init_oss(&mut self, url: &str) -> Result<(), McpError> {
        let new_mode = ClientMode::Oss { 
            url: url.to_string() 
        };
        
        // Only reinitialize if mode changed
        if self.current_mode != new_mode {
            self.oss_clients = Some(OssClients::new(url)?);
            self.cloud_clients = None;
            self.current_mode = new_mode;
        }
        
        Ok(())
    }
    
    /// Get cloud clients (returns error if in OSS mode)
    pub fn cloud(&self) -> Result<&CloudClients, McpError> {
        self.cloud_clients.as_ref()
            .ok_or_else(|| McpError::InvalidParameters {
                operation: "cloud_api_access".to_string(),
                message: "Cloud features not available in OSS mode".to_string(),
                missing_fields: vec![],
                invalid_fields: vec![],
            })
    }
    
    /// Get OSS clients
    pub fn oss(&self) -> Result<&OssClients, McpError> {
        self.oss_clients.as_ref()
            .ok_or_else(|| McpError::ConfigurationError {
                message: "OSS clients not initialized".to_string(),
                config_key: None,
                expected_format: None,
            })
    }
    
    /// Get component client (available in both modes)
    pub fn component(&self) -> Result<&ComponentClient, McpError> {
        match (&self.cloud_clients, &self.oss_clients) {
            (Some(cloud), _) => Ok(&cloud.component),
            (None, Some(oss)) => Ok(&oss.component),
            _ => Err(McpError::ConfigurationError {
                message: "No clients initialized".to_string(),
                config_key: None,
                expected_format: None,
            })
        }
    }
    
    /// Get worker client (available in both modes)
    pub fn worker(&self) -> Result<&WorkerClient, McpError> {
        match (&self.cloud_clients, &self.oss_clients) {
            (Some(cloud), _) => Ok(&cloud.worker),
            (None, Some(oss)) => Ok(&oss.worker),
            _ => Err(McpError::ConfigurationError {
                message: "No clients initialized".to_string(),
                config_key: None,
                expected_format: None,
            })
        }
    }
}