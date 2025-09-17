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

//! Direct API client integration for Golem MCP Server
//! 
//! This module provides direct access to Golem APIs, bypassing clap and the CLI
//! to provide the best agent experience as required.

use golem_client::api::{
    ComponentClient, ComponentClientLive, WorkerClient, WorkerClientLive,
};
use golem_client::model::{
    Component, ComponentQuery, InvokeParameters, InvokeResult,
    WorkerMetadata, WorkerCreationRequest, WorkerCreationResponse,
};
use golem_client::{Context as GolemContext, Security};
use golem_common::model::{ComponentId, WorkerId};
use golem_common::model::worker::WasiConfigVars;
use golem_wasm_rpc::json::OptionallyValueAndTypeJson;
use rmcp::ErrorData;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

/// Configuration for API clients
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Base URL for Golem API (e.g., http://localhost:9881 or https://api.golem.cloud)
    pub base_url: Url,
    /// Authentication token (if required)
    pub auth_token: Option<Uuid>,
    /// Whether to allow self-signed certificates
    pub allow_insecure: bool,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            base_url: Url::parse("http://localhost:9881").expect("Valid default URL"),
            auth_token: None,
            allow_insecure: false,
        }
    }
}

/// Wrapper around Golem API clients with simplified interface
pub struct GolemApiClient {
    pub config: ApiConfig,
    http_client: reqwest::Client,
    component_client: ComponentClientLive,
    worker_client: WorkerClientLive,
}

impl GolemApiClient {
    /// Create a new API client with the given configuration
    pub fn new(config: ApiConfig) -> Result<Self, ErrorData> {
        let http_client = reqwest::Client::builder()
            .danger_accept_invalid_certs(config.allow_insecure)
            .build()
            .map_err(|e| ErrorData::internal_error(format!("Failed to create HTTP client: {}", e), None))?;

        // Create the security token
        let security = if let Some(token) = &config.auth_token {
            Security::Bearer(token.to_string())
        } else {
            Security::Empty
        };

        // Create context for component client
        let component_context = GolemContext {
            client: http_client.clone(),
            base_url: config.base_url.clone(),
            security_token: security.clone(),
        };

        // Create context for worker client  
        let worker_context = GolemContext {
            client: http_client.clone(),
            base_url: config.base_url.clone(),
            security_token: security,
        };

        Ok(Self {
            config,
            http_client,
            component_client: ComponentClientLive { context: component_context },
            worker_client: WorkerClientLive { context: worker_context },
        })
    }

    /// Get the component client
    pub fn component_client(&self) -> &ComponentClientLive {
        &self.component_client
    }

    /// Get the worker client
    pub fn worker_client(&self) -> &WorkerClientLive {
        &self.worker_client
    }

    // High-level operations that combine multiple API calls

    /// Deploy a component from WASM bytes
    /// Note: This is simplified for MCP usage. The full CLI has many more options.
    pub async fn deploy_component(
        &self,
        name: &str,
        wasm_bytes: Vec<u8>,
    ) -> Result<Component, ErrorData> {
        use golem_client::model::ComponentType;
        
        // Create component with minimal required parameters
        let query = ComponentQuery {
            project_id: None, // Use default project
            component_name: name.to_string(),
        };
        
        let component = self.component_client
            .create_component(
                &query,
                wasm_bytes,
                Some(&ComponentType::Durable),
                None, // ifs_properties
                None::<Vec<u8>>, // ifs_archive  
                None, // dynamic_linking
                None, // component_env
                None, // component_args
            )
            .await
            .map_err(|e| ErrorData::internal_error(format!("Failed to create component: {}", e), None))?;

        Ok(component)
    }

    /// List all components
    pub async fn list_components(&self) -> Result<Vec<Component>, ErrorData> {
        // get_components returns Vec<Component> directly
        let components = self.component_client
            .get_components(None, None)
            .await
            .map_err(|e| ErrorData::internal_error(format!("Failed to list components: {}", e), None))?;

        Ok(components)
    }

    /// Create a new worker
    pub async fn create_worker(
        &self,
        component_id: &ComponentId,
        worker_name: &str,
        env: Vec<(String, String)>,
        args: Vec<String>,
    ) -> Result<WorkerId, ErrorData> {
        // Create WorkerCreationRequest
        let request = WorkerCreationRequest {
            name: worker_name.to_string(),
            args,
            env: env.into_iter().collect(),
            wasi_config_vars: WasiConfigVars::default(),
        };
        
        let _response: WorkerCreationResponse = self.worker_client
            .launch_new_worker(
                &component_id.0, // Uuid
                &request,
            )
            .await
            .map_err(|e| ErrorData::internal_error(format!("Failed to create worker: {}", e), None))?;

        // Construct WorkerId to return
        let worker_id = WorkerId {
            component_id: component_id.clone(),
            worker_name: worker_name.to_string(),
        };
        
        Ok(worker_id)
    }

    /// Invoke a function on a worker
    pub async fn invoke_worker_function(
        &self,
        worker_id: &WorkerId,
        function_name: &str,
        parameters: Vec<serde_json::Value>,
    ) -> Result<InvokeResult, ErrorData> {
        // Convert serde_json::Value to OptionallyValueAndTypeJson
        let params: Vec<OptionallyValueAndTypeJson> = parameters
            .into_iter()
            .map(|v| OptionallyValueAndTypeJson {
                typ: None, // No type information available
                value: v,
            })
            .collect();
            
        let invoke_params = InvokeParameters {
            params,
        };

        let result = self.worker_client
            .invoke_and_await_function(
                &worker_id.component_id.0, // Uuid
                &worker_id.worker_name,
                None, // idempotency_key
                function_name,
                &invoke_params,
            )
            .await
            .map_err(|e| ErrorData::internal_error(format!("Failed to invoke function: {}", e), None))?;

        Ok(result)
    }

    /// List workers for a component
    pub async fn list_workers(
        &self,
        component_id: &ComponentId,
    ) -> Result<Vec<WorkerMetadata>, ErrorData> {
        let result = self.worker_client
            .get_workers_metadata(
                &component_id.0, // UUID
                None, // filters
                None, // cursor  
                Some(100), // count
                Some(false), // precise
            )
            .await
            .map_err(|e| ErrorData::internal_error(format!("Failed to list workers: {}", e), None))?;

        Ok(result.workers)
    }
}

/// Load API configuration from environment or use defaults
pub fn load_api_config() -> ApiConfig {
    let base_url = std::env::var("GOLEM_API_URL")
        .ok()
        .and_then(|s| Url::parse(&s).ok())
        .unwrap_or_else(|| Url::parse("http://localhost:9881").expect("Valid default URL"));

    let auth_token = std::env::var("GOLEM_API_TOKEN")
        .ok()
        .and_then(|s| Uuid::parse_str(&s).ok());

    let allow_insecure = std::env::var("GOLEM_ALLOW_INSECURE")
        .ok()
        .map(|s| s == "true" || s == "1")
        .unwrap_or(false);

    ApiConfig {
        base_url,
        auth_token,
        allow_insecure,
    }
}