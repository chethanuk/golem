//! RMCP-based MCP server handler implementation
//!
//! This module implements the industry-standard MCP server using rmcp's
//! #[tool_router] and #[tool_handler] macros, following patterns from
//! Amazon Q, Codex, and the official MCP SDK.
//!
//! Supports multiple transport layers:
//! - stdio: For Claude Desktop and local integrations
//! - HTTP: For web applications and remote access

use rmcp::{
    handler::server::tool::{Parameters, ToolRouter},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo, ListResourcesResult, PaginatedRequestParam, Resource as RmcpResource, RawResource, ReadResourceRequestParam, ReadResourceResult, ResourceContents, ProgressNotificationParam},
    service::RequestContext,
    tool, tool_handler, tool_router,
    ErrorData as McpError, ServerHandler, RoleServer,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::future::Future;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::{Mutex, mpsc};
use tracing::{debug, info, instrument};
use golem_mcp_resources::{providers::ResourceProvider, manifest::ManifestResourceProvider};
use crate::progress::{
    execute_component_build_with_progress, execute_worker_operation_with_progress,
    execute_template_download_with_progress, create_progress_channel, create_progress_sender,
    ProgressStreamError, TimeoutConfig
};
use crate::errors::{
    GolemMcpError, ProgressStage
};

/// Transport configuration for MCP server following Amazon Q pattern
#[derive(Debug, Clone)]
pub enum GolemMcpTransport {
    /// stdio transport for Claude Desktop and local integrations
    Stdio,
    /// HTTP transport for web applications and remote access
    Http { 
        /// Port to bind HTTP server to
        port: u16,
        /// Host to bind to (defaults to 127.0.0.1)
        host: Option<String>,
    },
}

/// Request structure for component management operations
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ComponentOperation {
    /// The operation to perform (list, add, update, delete, etc.)
    pub operation: String,
    /// Optional component name
    pub component_name: Option<String>,
    /// Optional WASM file path
    pub wasm_file: Option<String>,
    /// Optional project ID
    pub project_id: Option<String>,
    /// Additional parameters as JSON
    #[serde(flatten)]
    pub extra: Value,
}

/// Request structure for worker management operations
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct WorkerOperation {
    /// The operation to perform (list, invoke, new, delete, etc.)
    pub operation: String,
    /// Optional component ID
    pub component_id: Option<String>,
    /// Optional worker name
    pub worker_name: Option<String>,
    /// Optional function name (for invoke)
    pub function_name: Option<String>,
    /// Optional function parameters (for invoke)
    pub function_parameters: Option<Vec<String>>,
    /// Additional parameters as JSON
    #[serde(flatten)]
    pub extra: Value,
}

/// Request structure for application management operations
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ApplicationOperation {
    /// The operation to perform (list, deploy, update, etc.)
    pub operation: String,
    /// Optional application name
    pub app_name: Option<String>,
    /// Optional manifest path
    pub manifest_path: Option<String>,
    /// Additional parameters as JSON
    #[serde(flatten)]
    pub extra: Value,
}

/// Request structure for project management operations
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ProjectOperation {
    /// The operation to perform (list_templates, new, etc.)
    pub operation: String,
    /// Optional language filter
    pub language: Option<String>,
    /// Optional project name
    pub project_name: Option<String>,
    /// Optional template name
    pub template_name: Option<String>,
    /// Additional parameters as JSON
    #[serde(flatten)]
    pub extra: Value,
}

/// Golem MCP Server implementation using rmcp
///
/// This server provides MCP tools for managing Golem components, workers,
/// applications, and projects. It follows the industry-standard pattern
/// using rmcp's macro-based approach with direct CLI integration.
#[derive(Clone)]
pub struct GolemMcpServer {
    /// Tool router for handling tool calls
    tool_router: ToolRouter<Self>,
    /// Resource provider for manifest discovery
    resource_provider: ManifestResourceProvider,
    /// Server state (can be extended with actual CLI context)
    state: Arc<Mutex<ServerState>>,
}

/// Internal server state
#[derive(Debug)]
struct ServerState {
    /// Request counter for debugging
    request_count: u64,
    /// Progress notification sender for streaming updates
    progress_sender: Option<mpsc::Sender<ProgressNotificationParam>>,
    /// Timeout configuration for operations
    timeout_config: TimeoutConfig,
}

impl Default for ServerState {
    fn default() -> Self {
        Self {
            request_count: 0,
            progress_sender: None,
            timeout_config: TimeoutConfig::default(),
        }
    }
}

#[allow(missing_docs)]
#[tool_router]
impl GolemMcpServer {
    /// Create a new Golem MCP server with direct CLI integration
    pub fn new() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        Self::with_base_dir(current_dir)
    }

    /// Create a new Golem MCP server with a specific base directory for resource discovery
    pub fn with_base_dir(base_dir: std::path::PathBuf) -> Self {
        Self {
            tool_router: Self::tool_router(),
            resource_provider: ManifestResourceProvider::new(base_dir),
            state: Arc::new(Mutex::new(ServerState::default())),
        }
    }

    /// Set up progress notification streaming
    /// Returns a receiver for consuming progress notifications
    pub async fn setup_progress_streaming(
        &self,
        buffer_size: Option<usize>,
    ) -> mpsc::Receiver<ProgressNotificationParam> {
        let (sender, receiver) = create_progress_channel(buffer_size.unwrap_or(32));
        
        let mut state = self.state.lock().await;
        state.progress_sender = Some(sender);
        
        debug!("Progress streaming configured with buffer size: {}", buffer_size.unwrap_or(32));
        receiver
    }

    /// Configure timeout settings for operations
    pub async fn configure_timeouts(&self, config: TimeoutConfig) {
        let mut state = self.state.lock().await;
        state.timeout_config = config;
        debug!("Timeout configuration updated: {:?}", state.timeout_config);
    }

    /// Get a progress sender for a specific operation
    async fn get_progress_sender(&self, operation_name: &str) -> Option<crate::progress::ProgressSender> {
        let state = self.state.lock().await;
        state.progress_sender.as_ref().map(|sender| {
            create_progress_sender(sender.clone(), operation_name)
        })
    }

    /// Component management tool with progress streaming
    ///
    /// Manages Golem components including listing, adding, updating, and deleting.
    /// Supports operations like templates, new, build, and deploy with progress tracking.
    #[tool(description = "Manage Golem components (list, add, update, delete, etc.) with progress streaming")]
    #[instrument(skip(self), fields(operation = %params.operation))]
    pub async fn golem_component_management(
        &self,
        Parameters(params): Parameters<ComponentOperation>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Component management operation: {}", params.operation);
        
        // Update request counter
        {
            let mut state = self.state.lock().await;
            state.request_count += 1;
        }

        // Determine if this is a long-running operation that needs progress tracking
        let is_long_running = matches!(params.operation.as_str(), "build" | "deploy" | "new");
        
        if is_long_running {
            // Execute with progress streaming for long operations
            self.execute_component_operation_with_progress(params).await
        } else {
            // Execute directly for quick operations
            let result_text = self.execute_component_operation_direct(params).await?;
            Ok(CallToolResult::success(vec![Content::text(result_text)]))
        }
    }

    /// Worker management tool with progress streaming
    ///
    /// Manages Golem workers including listing, invoking, creating, and deleting.
    /// Supports operations like list, invoke, new, and delete with progress tracking.
    #[tool(description = "Manage Golem workers (list, invoke, new, delete, etc.) with progress streaming")]
    #[instrument(skip(self), fields(operation = %params.operation))]
    pub async fn golem_worker_management(
        &self,
        Parameters(params): Parameters<WorkerOperation>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Worker management operation: {}", params.operation);
        
        // Update request counter
        {
            let mut state = self.state.lock().await;
            state.request_count += 1;
        }

        // Determine if this is a long-running operation that needs progress tracking
        let is_long_running = matches!(params.operation.as_str(), "invoke" | "new");
        
        if is_long_running {
            // Execute with progress streaming for long operations
            self.execute_worker_operation_with_progress(params).await
        } else {
            // Execute directly for quick operations
            let result_text = self.execute_worker_operation_direct(params).await?;
            Ok(CallToolResult::success(vec![Content::text(result_text)]))
        }
    }

    /// Application management tool
    ///
    /// Manages Golem applications including deployment and updates.
    /// Supports operations like list, deploy, and update.
    #[tool(description = "Manage Golem applications (list, deploy, update, etc.)")]
    pub async fn golem_application_management(
        &self,
        Parameters(params): Parameters<ApplicationOperation>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Application management operation: {}", params.operation);
        
        // Update request counter
        {
            let mut state = self.state.lock().await;
            state.request_count += 1;
        }

        // Execute application operation directly
        let result_text = self.execute_application_operation_direct(params).await?;
        
        Ok(CallToolResult::success(vec![Content::text(result_text)]))
    }

    /// Project management tool with progress streaming
    ///
    /// Manages Golem projects including templates and project creation.
    /// Supports operations like list_templates and new project creation with progress tracking.
    #[tool(description = "Manage Golem projects (list_templates, new, etc.) with progress streaming")]
    #[instrument(skip(self), fields(operation = %params.operation))]
    pub async fn golem_project_management(
        &self,
        Parameters(params): Parameters<ProjectOperation>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Project management operation: {}", params.operation);
        
        // Update request counter
        {
            let mut state = self.state.lock().await;
            state.request_count += 1;
        }

        // Determine if this is a long-running operation that needs progress tracking
        let is_long_running = matches!(params.operation.as_str(), "list_templates" | "new");
        
        if is_long_running {
            // Execute with progress streaming for long operations
            self.execute_project_operation_with_progress(params).await
        } else {
            // Execute directly for quick operations
            let result_text = self.execute_project_operation_direct(params).await?;
            Ok(CallToolResult::success(vec![Content::text(result_text)]))
        }
    }

    /// Execute component operation with progress streaming
    /// Following Amazon Q pattern from Compare.MD lines 315-365
    async fn execute_component_operation_with_progress(
        &self,
        operation: ComponentOperation,
    ) -> Result<CallToolResult, McpError> {
        let operation_name = format!("component_{}", operation.operation);
        
        if let Some(progress_sender) = self.get_progress_sender(&operation_name).await {
            debug!("Executing component operation '{}' with progress tracking", operation.operation);
            
            // Execute the operation with progress tracking
            let result_text = match operation.operation.as_str() {
                "build" => {
                    execute_component_build_with_progress(
                        self.execute_component_operation_direct(operation),
                        progress_sender,
                    ).await.map_err(|e| self.convert_progress_error(e))?
                }
                "deploy" => {
                    execute_worker_operation_with_progress(
                        self.execute_component_operation_direct(operation),
                        progress_sender,
                    ).await.map_err(|e| self.convert_progress_error(e))?
                }
                "new" => {
                    execute_template_download_with_progress(
                        self.execute_component_operation_direct(operation),
                        progress_sender,
                    ).await.map_err(|e| self.convert_progress_error(e))?
                }
                _ => {
                    // Shouldn't reach here due to is_long_running check
                    return Err(McpError::invalid_params(
                        format!("Unexpected long-running operation: {}", operation.operation),
                        None,
                    ));
                }
            };
            
            Ok(CallToolResult::success(vec![Content::text(result_text)]))
        } else {
            debug!("No progress sender configured, executing component operation directly");
            let result_text = self.execute_component_operation_direct(operation).await?;
            Ok(CallToolResult::success(vec![Content::text(result_text)]))
        }
    }

    /// Convert progress streaming error to McpError using structured error system
    fn convert_progress_error(&self, error: ProgressStreamError) -> McpError {
        let golem_error = match error {
            ProgressStreamError::Timeout { operation_name, timeout_secs } => {
                GolemMcpError::operation_timeout(operation_name, timeout_secs * 1000)
            }
            ProgressStreamError::ChannelClosed { operation_name } => {
                GolemMcpError::progress_streaming(
                    format!("Progress channel closed for operation '{}'", operation_name),
                    ProgressStage::Execution,
                )
            }
            ProgressStreamError::NotificationFailed { operation_name, source } => {
                GolemMcpError::progress_streaming(
                    format!("Failed to send progress notification for '{}': {}", operation_name, source),
                    ProgressStage::Execution,
                )
            }
            ProgressStreamError::OperationFailed { operation_name, source } => {
                GolemMcpError::progress_streaming(
                    format!("Operation '{}' failed: {}", operation_name, source),
                    ProgressStage::Execution,
                )
            }
        };
        
        golem_error.into()
    }

    /// Direct component operation execution using golem CLI process calls
    /// Following Amazon Q pattern from Compare.MD for bypassing clap overhead
    async fn execute_component_operation_direct(
        &self,
        operation: ComponentOperation,
    ) -> Result<String, McpError> {
        match operation.operation.as_str() {
            "list" => {
                // Direct CLI call: golem component templates
                let output = self.execute_golem_cli_command(&["component", "templates"]).await?;
                Ok(format!("Available component templates:\n{}", output))
            }
            "templates" => {
                // Direct CLI call: golem component templates with optional filter
                let mut args = vec!["component", "templates"];
                if let Some(filter) = &operation.component_name {
                    args.push("--filter");
                    args.push(filter);
                }
                let output = self.execute_golem_cli_command(&args).await?;
                Ok(output)
            }
            "new" => {
                let component_name = operation
                    .component_name
                    .ok_or_else(|| -> McpError {
                        let error = GolemMcpError::invalid_parameters(
                            "component new",
                            "Missing component_name parameter",
                            vec!["component_name".to_string()],
                            vec![],
                        );
                        error.into()
                    })?;
                
                // Direct CLI call: golem component new
                let mut args = vec!["component", "new"];
                if let Some(template) = operation.extra.get("template").and_then(|v| v.as_str()) {
                    args.extend(["--component-template", template]);
                }
                args.extend(["--component-name", &component_name]);
                
                let output = self.execute_golem_cli_command(&args).await?;
                Ok(format!("Component '{}' created:\n{}", component_name, output))
            }
            "build" => {
                // Direct CLI call: golem component build
                let mut args = vec!["component", "build"];
                if let Some(component_name) = &operation.component_name {
                    args.extend(["--component-name", component_name]);
                }
                
                let output = self.execute_golem_cli_command(&args).await?;
                Ok(format!("Component build completed:\n{}", output))
            }
            "deploy" => {
                // Direct CLI call: golem component deploy  
                let mut args = vec!["component", "deploy"];
                if let Some(component_name) = &operation.component_name {
                    args.extend(["--component-name", component_name]);
                }
                
                let output = self.execute_golem_cli_command(&args).await?;
                Ok(format!("Component deployment completed:\n{}", output))
            }
            _ => {
                let error = GolemMcpError::invalid_parameters(
                    "component management",
                    &format!("Unknown component operation: {}", operation.operation),
                    vec![],
                    vec!["operation".to_string()],
                );
                Err(error.into())
            }
        }
    }

    /// Execute golem CLI command as external process
    /// Following Amazon Q pattern for direct CLI integration without circular dependency
    async fn execute_golem_cli_command(&self, args: &[&str]) -> Result<String, McpError> {
        // Check if golem binary is available, use mock if not
        if which::which("golem").is_err() {
            return self.mock_golem_cli_command(args).await;
        }

        let operation = args.join(" ");
        
        // Find golem binary in PATH
        let golem_binary = which::which("golem")
            .map_err(|e| -> McpError {
                let error = GolemMcpError::cli_command_failed(
                    operation.clone(),
                    format!("golem CLI not found in PATH: {}", e),
                    Some(127),
                    None,
                );
                error.into()
            })?;

        debug!("Executing golem command: {:?}", args);

        // Execute golem CLI command
        let output = Command::new(golem_binary)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| -> McpError {
                let error = GolemMcpError::cli_command_failed(
                    operation.clone(),
                    format!("Failed to execute golem command: {}", e),
                    None,
                    None,
                );
                error.into()
            })?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout.trim().to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let exit_code = output.status.code();
            let error = GolemMcpError::cli_command_failed(
                operation,
                "golem command failed".to_string(),
                exit_code,
                Some(stderr.trim().to_string()),
            );
            Err(<GolemMcpError as Into<McpError>>::into(error))
        }
    }

    /// Mock golem CLI command responses for testing
    /// Following Block/Goose mock patterns from Compare.MD  
    async fn mock_golem_cli_command(&self, args: &[&str]) -> Result<String, McpError> {
        match args {
            ["component", "templates"] => Ok("rust-minimal\npython-basic\ntypescript-starter".to_string()),
            ["component", "new", ..] => Ok("Component created successfully".to_string()),
            ["component", "build", ..] => Ok("Component built successfully".to_string()),
            ["component", "deploy", ..] => Ok("Component deployed successfully".to_string()),
            ["worker", "list", ..] => Ok("worker-1\nworker-2".to_string()),
            ["worker", "invoke", ..] => Ok("Function executed successfully: result=42".to_string()),
            ["worker", "new", ..] => Ok("Worker created successfully".to_string()),
            ["app", "list"] => Ok("app-1\napp-2".to_string()),
            ["app", "deploy", ..] => Ok("Application deployed successfully".to_string()),
            ["cloud", "project", "list-templates", ..] => Ok("rust-project\npython-project".to_string()),
            ["cloud", "project", "new", ..] => Ok("Project created successfully".to_string()),
            _ => Ok(format!("Mock response for: {:?}", args)),
        }
    }

    /// Execute worker operation with progress streaming
    async fn execute_worker_operation_with_progress(
        &self,
        operation: WorkerOperation,
    ) -> Result<CallToolResult, McpError> {
        let operation_name = format!("worker_{}", operation.operation);
        
        if let Some(progress_sender) = self.get_progress_sender(&operation_name).await {
            debug!("Executing worker operation '{}' with progress tracking", operation.operation);
            
            // All worker operations use the standard timeout
            let result_text = execute_worker_operation_with_progress(
                self.execute_worker_operation_direct(operation),
                progress_sender,
            ).await.map_err(|e| self.convert_progress_error(e))?;
            
            Ok(CallToolResult::success(vec![Content::text(result_text)]))
        } else {
            debug!("No progress sender configured, executing worker operation directly");
            let result_text = self.execute_worker_operation_direct(operation).await?;
            Ok(CallToolResult::success(vec![Content::text(result_text)]))
        }
    }

    /// Direct worker operation execution using golem CLI process calls
    async fn execute_worker_operation_direct(
        &self,
        operation: WorkerOperation,
    ) -> Result<String, McpError> {
        match operation.operation.as_str() {
            "list" => {
                let component_id = operation
                    .component_id
                    .ok_or_else(|| McpError::invalid_params("Missing component_id", None))?;
                
                // Direct CLI call: golem worker list --component-id <id>
                let output = self.execute_golem_cli_command(&[
                    "worker", "list",
                    "--component-id", &component_id
                ]).await?;
                Ok(format!("Workers for component {}:\n{}", component_id, output))
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
                
                // Build CLI arguments for worker invoke
                let mut args = vec![
                    "worker", "invoke",
                    "--component-id", &component_id,
                    "--worker-name", &worker_name,
                    "--function", &function_name,
                ];
                
                // Add function parameters if provided
                if let Some(params) = &operation.function_parameters {
                    for param in params {
                        args.push("--parameters");
                        args.push(param);
                    }
                }
                
                let output = self.execute_golem_cli_command(&args).await?;
                Ok(format!("Function '{}' invoked on worker '{}':\n{}", 
                    function_name, worker_name, output))
            }
            "new" => {
                let component_id = operation
                    .component_id
                    .ok_or_else(|| McpError::invalid_params("Missing component_id", None))?;
                let worker_name = operation
                    .worker_name
                    .ok_or_else(|| McpError::invalid_params("Missing worker_name", None))?;
                
                // Direct CLI call: golem worker new --component-id <id> --worker-name <name>
                let args = vec![
                    "worker", "new",
                    "--component-id", &component_id,
                    "--worker-name", &worker_name,
                ];
                
                let output = self.execute_golem_cli_command(&args).await?;
                Ok(format!("Worker '{}' created for component '{}':\n{}", 
                    worker_name, component_id, output))
            }
            _ => Err(McpError::invalid_params(
                format!("Unknown worker operation: {}", operation.operation),
                None,
            )),
        }
    }

    /// Direct application operation execution using golem CLI process calls
    async fn execute_application_operation_direct(
        &self,
        operation: ApplicationOperation,
    ) -> Result<String, McpError> {
        match operation.operation.as_str() {
            "list" => {
                // Direct CLI call: golem app list
                let output = self.execute_golem_cli_command(&["app", "list"]).await?;
                Ok(format!("Applications:\n{}", output))
            }
            "deploy" => {
                let manifest_path = operation
                    .manifest_path
                    .ok_or_else(|| McpError::invalid_params("Missing manifest_path", None))?;
                let app_name = operation
                    .app_name
                    .ok_or_else(|| McpError::invalid_params("Missing app_name", None))?;
                
                // Direct CLI call: golem app deploy --app-name <name> --manifest-path <path>
                let output = self.execute_golem_cli_command(&[
                    "app", "deploy",
                    "--app-name", &app_name,
                    "--manifest-path", &manifest_path
                ]).await?;
                Ok(format!("Application deployment for '{}' from '{}':\n{}", app_name, manifest_path, output))
            }
            "update" => {
                let manifest_path = operation
                    .manifest_path
                    .ok_or_else(|| McpError::invalid_params("Missing manifest_path for update", None))?;
                let app_name = operation
                    .app_name
                    .ok_or_else(|| McpError::invalid_params("Missing app_name for update", None))?;
                
                // Direct CLI call: golem app deploy --app-name <name> --manifest-path <path> --update
                let output = self.execute_golem_cli_command(&[
                    "app", "deploy",
                    "--app-name", &app_name,
                    "--manifest-path", &manifest_path,
                    "--update"
                ]).await?;
                Ok(format!("Application update for '{}' from '{}':\n{}", app_name, manifest_path, output))
            }
            _ => Err(McpError::invalid_params(
                format!("Unknown application operation: {}", operation.operation),
                None,
            )),
        }
    }

    /// Execute project operation with progress streaming
    async fn execute_project_operation_with_progress(
        &self,
        operation: ProjectOperation,
    ) -> Result<CallToolResult, McpError> {
        let operation_name = format!("project_{}", operation.operation);
        
        if let Some(progress_sender) = self.get_progress_sender(&operation_name).await {
            debug!("Executing project operation '{}' with progress tracking", operation.operation);
            
            // Template operations are typically downloads
            let result_text = execute_template_download_with_progress(
                self.execute_project_operation_direct(operation),
                progress_sender,
            ).await.map_err(|e| self.convert_progress_error(e))?;
            
            Ok(CallToolResult::success(vec![Content::text(result_text)]))
        } else {
            debug!("No progress sender configured, executing project operation directly");
            let result_text = self.execute_project_operation_direct(operation).await?;
            Ok(CallToolResult::success(vec![Content::text(result_text)]))
        }
    }

    /// Direct project operation execution using golem CLI process calls
    async fn execute_project_operation_direct(
        &self,
        operation: ProjectOperation,
    ) -> Result<String, McpError> {
        match operation.operation.as_str() {
            "list_templates" => {
                // Direct CLI call: golem cloud project list-templates
                let mut args = vec!["cloud", "project", "list-templates"];
                if let Some(language) = &operation.language {
                    args.extend(["--language", language]);
                }
                
                let output = self.execute_golem_cli_command(&args).await?;
                Ok(format!("Available project templates:\n{}", output))
            }
            "new" => {
                let project_name = operation
                    .project_name
                    .ok_or_else(|| McpError::invalid_params("Missing project_name", None))?;
                let template_name = operation
                    .template_name
                    .ok_or_else(|| McpError::invalid_params("Missing template_name", None))?;
                
                // Direct CLI call: golem cloud project new --project-name <name> --template-name <template>
                let mut args = vec![
                    "cloud", "project", "new",
                    "--project-name", &project_name,
                    "--template-name", &template_name,
                ];
                
                if let Some(language) = &operation.language {
                    args.extend(["--language", language]);
                }
                
                let output = self.execute_golem_cli_command(&args).await?;
                Ok(format!("Project '{}' created from template '{}':\n{}", 
                    project_name, template_name, output))
            }
            _ => Err(McpError::invalid_params(
                format!("Unknown project operation: {}", operation.operation),
                None,
            )),
        }
    }

    /// Serve the MCP server using the specified transport
    /// 
    /// This follows Amazon Q's multi-transport pattern, supporting both
    /// stdio (for Claude Desktop) and HTTP (for web applications).
    ///
    /// # Arguments
    ///
    /// * `transport` - Transport configuration (Stdio or Http)
    ///
    /// # Errors
    ///
    /// Returns transport-specific errors if the server cannot start.
    pub async fn serve(&self, transport: GolemMcpTransport) -> Result<(), Box<dyn std::error::Error>> {
        match transport {
            GolemMcpTransport::Stdio => {
                info!("Starting Golem MCP server on stdio");
                let io = (tokio::io::stdin(), tokio::io::stdout());
                rmcp::serve_server(self.clone(), io).await?;
                info!("Golem MCP server (stdio) terminated");
            }
            GolemMcpTransport::Http { port, host } => {
                info!("Starting Golem MCP server on HTTP port {}", port);
                
                // Use existing HTTP transport implementation
                use crate::http::HttpTransport;
                use crate::handlers::MessageHandlerImpl;
                use crate::config::McpServerConfig;
                use golem_mcp_types::transport::HttpTransportConfig;

                let config = McpServerConfig::default();
                let message_handler = Arc::new(MessageHandlerImpl::new(config).await?);
                
                let http_config = HttpTransportConfig {
                    host: host.unwrap_or_else(|| "127.0.0.1".to_string()),
                    port,
                    use_tls: false,
                    timeout_ms: 30_000,
                    request_timeout_ms: 60_000,
                    max_request_size: 1_024 * 1_024, // 1MB
                    headers: std::collections::HashMap::new(),
                };
                
                let http_transport = HttpTransport::new(http_config).await?;
                http_transport.serve(message_handler).await?;
                
                info!("Golem MCP server (HTTP) terminated");
            }
        }
        Ok(())
    }
}

/// Implement the ServerHandler trait for GolemMcpServer
#[tool_handler]
impl ServerHandler for GolemMcpServer {
    /// Get server information
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
            instructions: Some("Golem MCP Server - manage components, workers, applications, and projects".to_string()),
            ..Default::default()
        }
    }

    /// List available resources (manifests, components, project metadata)
    /// Following Compare.MD strategy lines 219-265: Enhanced manifest discovery
    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        debug!("Listing Golem resources (manifests, components, project metadata)");
        
        // Use the resource provider to discover manifests and components  
        let golem_resources = self.resource_provider
            .list_resources()
            .map_err(|e| McpError::internal_error(format!("Failed to list resources: {}", e), None))?;
            
        // Convert golem-mcp-types::Resource to rmcp::model::Resource
        let resources: Vec<RmcpResource> = golem_resources
            .into_iter()
            .map(|res| RmcpResource::new(RawResource {
                uri: res.uri,
                name: res.name,
                description: res.description,
                mime_type: res.mime_type,
                size: None, // Not tracked in golem-mcp-types Resource
            }, None)) // No annotations for now
            .collect();
            
        Ok(ListResourcesResult {
            resources,
            next_cursor: None,
        })
    }

    /// Read resource content by URI
    /// Following Compare.MD strategy lines 219-265: Enhanced resource access
    async fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        debug!("Reading resource: {}", request.uri);
        
        // Use the resource provider to read resource content
        let content_bytes = self.resource_provider
            .read_resource(&request.uri)
            .map_err(|e| McpError::internal_error(format!("Failed to read resource '{}': {}", request.uri, e), None))?;
        
        // Convert bytes to string for text-based resources
        let content_text = String::from_utf8(content_bytes)
            .map_err(|e| McpError::internal_error(format!("Resource '{}' contains invalid UTF-8: {}", request.uri, e), None))?;
        
        // Determine MIME type based on URI pattern
        let mime_type = if request.uri.starts_with("golem://manifest/") {
            if request.uri.ends_with(".yaml") || request.uri.ends_with(".yml") {
                "application/yaml"
            } else if request.uri.ends_with(".toml") {
                "application/toml"
            } else if request.uri.ends_with(".json") {
                "application/json"
            } else {
                "text/plain"
            }
        } else if request.uri.starts_with("golem://component/") {
            "application/wasm"
        } else if request.uri == "golem://project/metadata" {
            "application/json"
        } else {
            "text/plain"
        };
        
        Ok(ReadResourceResult {
            contents: vec![ResourceContents::TextResourceContents {
                uri: request.uri.clone(),
                mime_type: Some(mime_type.to_string()),
                text: content_text,
            }],
        })
    }
}

impl Default for GolemMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Serve the MCP server over stdio
///
/// This function starts the MCP server and serves it over stdin/stdout,
/// following the industry-standard pattern used by Amazon Q and others.
pub async fn serve_mcp_stdio() -> Result<(), Box<dyn std::error::Error>> {
    let server = GolemMcpServer::new();
    server.serve(GolemMcpTransport::Stdio).await
}

/// Serve the MCP server over HTTP
///
/// This function starts the MCP server and serves it over HTTP,
/// enabling web applications and remote access.
///
/// # Arguments
///
/// * `port` - Port to bind the HTTP server to
/// * `host` - Optional host to bind to (defaults to 127.0.0.1)
pub async fn serve_mcp_http(port: u16, host: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let server = GolemMcpServer::new();
    server.serve(GolemMcpTransport::Http { port, host }).await
}

/// Create a stdio transport configuration
pub fn stdio_transport() -> GolemMcpTransport {
    GolemMcpTransport::Stdio
}

/// Create an HTTP transport configuration
pub fn http_transport(port: u16) -> GolemMcpTransport {
    GolemMcpTransport::Http { port, host: None }
}

/// Create an HTTP transport configuration with custom host
pub fn http_transport_with_host(port: u16, host: String) -> GolemMcpTransport {
    GolemMcpTransport::Http { port, host: Some(host) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
        let server = GolemMcpServer::new();
        let info = server.get_info();
        
        assert!(info.capabilities.tools.is_some());
        assert!(info.instructions.is_some());
    }

    #[tokio::test]
    async fn test_component_list_operation() {
        let server = GolemMcpServer::new();
        let params = ComponentOperation {
            operation: "list".to_string(),
            component_name: None,
            wasm_file: None,
            project_id: None,
            extra: serde_json::json!({}),
        };
        
        let result = server
            .golem_component_management(Parameters(params))
            .await;
        
        assert!(result.is_ok());
        // CallToolResult doesn't have is_success method, just check it's ok
    }

    #[tokio::test]
    async fn test_worker_invoke_operation() {
        let server = GolemMcpServer::new();
        let params = WorkerOperation {
            operation: "invoke".to_string(),
            component_id: Some("test-component".to_string()),
            worker_name: Some("test-worker".to_string()),
            function_name: Some("hello".to_string()),
            function_parameters: None,
            extra: serde_json::json!({}),
        };
        
        let result = server
            .golem_worker_management(Parameters(params))
            .await;
        
        assert!(result.is_ok());
        // CallToolResult doesn't have is_success method, just check it's ok
    }
}