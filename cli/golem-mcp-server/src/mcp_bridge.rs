//! Bridge between MCP server and golem-cli command handlers
//!
//! This module provides the integration layer between the MCP server's
//! tool operations and the actual golem-cli command handlers.

use crate::rmcp_handler::{
    ApplicationOperation, ComponentOperation, ProjectOperation, WorkerOperation,
};
use anyhow::Result;
use async_trait::async_trait;
use rmcp::ErrorData as McpError;

/// Trait for bridging MCP operations to CLI handlers
///
/// This trait defines the interface that must be implemented to connect
/// MCP tool operations to the actual CLI command handlers.
#[async_trait]
pub trait McpHandlerBridge: Send + Sync {
    /// Execute a component management operation
    async fn execute_component_operation(
        &self,
        operation: ComponentOperation,
    ) -> Result<String, McpError>;

    /// Execute a worker management operation
    async fn execute_worker_operation(
        &self,
        operation: WorkerOperation,
    ) -> Result<String, McpError>;

    /// Execute an application management operation
    async fn execute_application_operation(
        &self,
        operation: ApplicationOperation,
    ) -> Result<String, McpError>;

    /// Execute a project management operation
    async fn execute_project_operation(
        &self,
        operation: ProjectOperation,
    ) -> Result<String, McpError>;
}

/// Mock implementation of McpHandlerBridge for testing
///
/// This implementation returns mock responses without actually calling
/// the CLI handlers. Used when the server is running without CLI integration.
#[derive(Clone)]
pub struct MockHandlerBridge;

#[async_trait]
impl McpHandlerBridge for MockHandlerBridge {
    async fn execute_component_operation(
        &self,
        operation: ComponentOperation,
    ) -> Result<String, McpError> {
        match operation.operation.as_str() {
            "list" => Ok("Component list operation executed. Components: []".to_string()),
            "add" => {
                let component_name = operation
                    .component_name
                    .ok_or_else(|| McpError::invalid_params("Missing component_name", None))?;
                let wasm_file = operation
                    .wasm_file
                    .ok_or_else(|| McpError::invalid_params("Missing wasm_file", None))?;
                Ok(format!(
                    "Component '{}' added from '{}'",
                    component_name, wasm_file
                ))
            }
            _ => Err(McpError::invalid_params(
                format!("Unknown operation: {}", operation.operation),
                None,
            )),
        }
    }

    async fn execute_worker_operation(
        &self,
        operation: WorkerOperation,
    ) -> Result<String, McpError> {
        match operation.operation.as_str() {
            "list" => {
                let component_id = operation
                    .component_id
                    .ok_or_else(|| McpError::invalid_params("Missing component_id", None))?;
                Ok(format!("Worker list for component {}: []", component_id))
            }
            "invoke" => {
                let component_id = operation
                    .component_id
                    .ok_or_else(|| McpError::invalid_params("Missing component_id", None))?;
                let worker_name = operation
                    .worker_name
                    .ok_or_else(|| McpError::invalid_params("Missing worker_name", None))?;
                let function_name = operation
                    .function_name
                    .ok_or_else(|| McpError::invalid_params("Missing function_name", None))?;
                Ok(format!(
                    "Function '{}' invoked on worker '{}' (component: {})",
                    function_name, worker_name, component_id
                ))
            }
            _ => Err(McpError::invalid_params(
                format!("Unknown operation: {}", operation.operation),
                None,
            )),
        }
    }

    async fn execute_application_operation(
        &self,
        operation: ApplicationOperation,
    ) -> Result<String, McpError> {
        match operation.operation.as_str() {
            "list" => Ok("Application list: []".to_string()),
            "deploy" => {
                let app_name = operation
                    .app_name
                    .ok_or_else(|| McpError::invalid_params("Missing app_name", None))?;
                let manifest_path = operation
                    .manifest_path
                    .ok_or_else(|| McpError::invalid_params("Missing manifest_path", None))?;
                Ok(format!(
                    "Application '{}' deployed from '{}'",
                    app_name, manifest_path
                ))
            }
            _ => Err(McpError::invalid_params(
                format!("Unknown operation: {}", operation.operation),
                None,
            )),
        }
    }

    async fn execute_project_operation(
        &self,
        operation: ProjectOperation,
    ) -> Result<String, McpError> {
        match operation.operation.as_str() {
            "list_templates" => {
                let language = operation.language.as_deref();
                let filter_msg = language.map_or(String::new(), |l| format!(" (filtered: {})", l));
                Ok(format!(
                    "Available templates{}: rust-minimal, python-basic, typescript-starter",
                    filter_msg
                ))
            }
            "new" => {
                let project_name = operation
                    .project_name
                    .ok_or_else(|| McpError::invalid_params("Missing project_name", None))?;
                let template_name = operation
                    .template_name
                    .ok_or_else(|| McpError::invalid_params("Missing template_name", None))?;
                Ok(format!(
                    "Project '{}' created from template '{}'",
                    project_name, template_name
                ))
            }
            _ => Err(McpError::invalid_params(
                format!("Unknown operation: {}", operation.operation),
                None,
            )),
        }
    }
}

/// Error conversion utilities
pub mod error_conversion {
    use super::*;

    /// Convert anyhow::Error to MCP ErrorData
    pub fn anyhow_to_mcp_error(err: anyhow::Error) -> McpError {
        // Try to extract specific error types if possible
        if let Some(msg) = err.downcast_ref::<String>() {
            McpError::internal_error(msg.clone(), None)
        } else if let Some(msg) = err.downcast_ref::<&str>() {
            McpError::internal_error(msg.to_string(), None)
        } else {
            // Default conversion
            McpError::internal_error(err.to_string(), None)
        }
    }

    /// Convert Result<T, anyhow::Error> to Result<T, McpError>
    pub fn convert_result<T>(result: Result<T>) -> Result<T, McpError> {
        result.map_err(anyhow_to_mcp_error)
    }
}

#[cfg(feature = "cli-integration")]
pub mod cli_integration {
    use super::*;
    use golem_cli::command::component::ComponentSubcommand;
    use golem_cli::command::worker::WorkerSubcommand;
    use golem_cli::command::app::AppSubcommand;
    use golem_cli::command::cloud::project::ProjectSubcommand;
    use golem_cli::command_handler::component::ComponentCommandHandler;
    use golem_cli::command_handler::worker::WorkerCommandHandler;
    use golem_cli::command_handler::app::AppCommandHandler;
    use golem_cli::command_handler::cloud::project::CloudProjectCommandHandler;
    use golem_cli::context::Context;
    use std::sync::Arc;

    /// Real implementation of McpHandlerBridge that uses golem-cli handlers
    pub struct CliHandlerBridge {
        context: Arc<Context>,
    }

    impl CliHandlerBridge {
        /// Create a new CLI handler bridge with the given context
        pub fn new(context: Arc<Context>) -> Self {
            Self { context }
        }

        /// Create a new CLI handler bridge with default context
        pub async fn new_default() -> Result<Self> {
            let context = Arc::new(Context::default());
            Ok(Self { context })
        }
    }

    #[async_trait]
    impl McpHandlerBridge for CliHandlerBridge {
        async fn execute_component_operation(
            &self,
            operation: ComponentOperation,
        ) -> Result<String, McpError> {
            let handler = ComponentCommandHandler::new(self.context.clone());
            
            // Convert MCP operation to CLI subcommand
            let subcommand = convert_to_component_subcommand(operation)?;
            
            // Execute the command
            handler
                .handle_command(subcommand)
                .await
                .map_err(error_conversion::anyhow_to_mcp_error)?;
            
            // TODO: Capture and return actual output
            Ok("Component operation completed successfully".to_string())
        }

        async fn execute_worker_operation(
            &self,
            operation: WorkerOperation,
        ) -> Result<String, McpError> {
            let handler = WorkerCommandHandler::new(self.context.clone());
            
            // Convert MCP operation to CLI subcommand
            let subcommand = convert_to_worker_subcommand(operation)?;
            
            // Execute the command
            handler
                .handle_command(subcommand)
                .await
                .map_err(error_conversion::anyhow_to_mcp_error)?;
            
            // TODO: Capture and return actual output
            Ok("Worker operation completed successfully".to_string())
        }

        async fn execute_application_operation(
            &self,
            operation: ApplicationOperation,
        ) -> Result<String, McpError> {
            let handler = AppCommandHandler::new(self.context.clone());
            
            // Convert MCP operation to CLI subcommand
            let subcommand = convert_to_app_subcommand(operation)?;
            
            // Execute the command
            handler
                .handle_command(subcommand)
                .await
                .map_err(error_conversion::anyhow_to_mcp_error)?;
            
            // TODO: Capture and return actual output
            Ok("Application operation completed successfully".to_string())
        }

        async fn execute_project_operation(
            &self,
            operation: ProjectOperation,
        ) -> Result<String, McpError> {
            let handler = CloudProjectCommandHandler::new(self.context.clone());
            
            // Convert MCP operation to CLI subcommand
            let subcommand = convert_to_project_subcommand(operation)?;
            
            // Execute the command
            handler
                .handle_command(subcommand)
                .await
                .map_err(error_conversion::anyhow_to_mcp_error)?;
            
            // TODO: Capture and return actual output
            Ok("Project operation completed successfully".to_string())
        }
    }

    /// Convert MCP ComponentOperation to CLI ComponentSubcommand
    fn convert_to_component_subcommand(
        operation: ComponentOperation,
    ) -> Result<ComponentSubcommand, McpError> {
        use golem_cli::command::{ComponentOptionalComponentNames, BuildArgs, ForceBuildArg, UpdateOrRedeployArgs};
        use golem_common::model::PackageName;
        use std::str::FromStr;
        
        match operation.operation.as_str() {
            "list" | "templates" => {
                Ok(ComponentSubcommand::Templates {
                    filter: None, // Could extract from operation.extra if needed
                })
            }
            "new" => {
                let component_name = operation.component_name
                    .and_then(|name| PackageName::from_str(&name).ok());
                Ok(ComponentSubcommand::New {
                    component_template: None, // Could be extracted from operation.extra
                    component_name,
                })
            }
            "build" => {
                let component_name = ComponentOptionalComponentNames::default(); // Could be populated from operation.component_name
                Ok(ComponentSubcommand::Build {
                    component_name,
                    build: BuildArgs::default(),
                })
            }
            "deploy" => {
                let component_name = ComponentOptionalComponentNames::default();
                Ok(ComponentSubcommand::Deploy {
                    component_name,
                    force_build: ForceBuildArg::default(),
                    update_or_redeploy: UpdateOrRedeployArgs::default(),
                })
            }
            _ => Err(McpError::invalid_params(
                format!("Unknown component operation: {}", operation.operation),
                None,
            )),
        }
    }

    /// Convert MCP WorkerOperation to CLI WorkerSubcommand
    fn convert_to_worker_subcommand(
        operation: WorkerOperation,
    ) -> Result<WorkerSubcommand, McpError> {
        use golem_cli::command::{WorkerNameArg, NewWorkerArgument};
        use golem_common::model::WorkerId;
        
        match operation.operation.as_str() {
            "new" => {
                let worker_name = WorkerNameArg::default(); // Could be populated from operation.worker_name
                Ok(WorkerSubcommand::New {
                    worker_name,
                    arguments: Vec::new(), // Could be populated from operation.extra
                    env: Vec::new(),
                })
            }
            "invoke" => {
                let worker_name = WorkerNameArg::default();
                Ok(WorkerSubcommand::Invoke {
                    worker_name,
                    function_name: operation.function_name.unwrap_or_default(),
                    arguments: operation.function_parameters.unwrap_or_default(),
                    invocation_key: None,
                    env: Vec::new(),
                })
            }
            _ => {
                // For now, default to invoke for unknown operations  
                let worker_name = WorkerNameArg::default();
                Ok(WorkerSubcommand::Invoke {
                    worker_name,
                    function_name: operation.function_name.unwrap_or("main".to_string()),
                    arguments: Vec::new(),
                    invocation_key: None,
                    env: Vec::new(),
                })
            }
        }
    }

    /// Convert MCP ApplicationOperation to CLI AppSubcommand
    fn convert_to_app_subcommand(
        operation: ApplicationOperation,
    ) -> Result<AppSubcommand, McpError> {
        match operation.operation.as_str() {
            "deploy" => {
                // Basic deploy operation - would need more sophisticated parameter mapping
                Ok(AppSubcommand::Deploy {
                    manifest_path: operation.manifest_path.unwrap_or_default(),
                    force_build: false,
                    redeploy: false,
                    update_mode: None,
                    skip_types_download: false,
                    skip_up_to_date_checks: false,
                })
            }
            _ => Err(McpError::invalid_params(
                format!("Unknown application operation: {}", operation.operation),
                None,
            )),
        }
    }

    /// Convert MCP ProjectOperation to CLI ProjectSubcommand
    fn convert_to_project_subcommand(
        operation: ProjectOperation,
    ) -> Result<ProjectSubcommand, McpError> {
        match operation.operation.as_str() {
            "list_templates" => {
                Ok(ProjectSubcommand::ListTemplates {
                    language: operation.language,
                })
            }
            "new" => {
                Ok(ProjectSubcommand::New {
                    project_name: operation.project_name.unwrap_or_default(),
                    template_name: operation.template_name,
                    language: operation.language,
                })
            }
            _ => Err(McpError::invalid_params(
                format!("Unknown project operation: {}", operation.operation),
                None,
            )),
        }
    }
}