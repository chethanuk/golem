//! Direct execution of Golem CLI commands
//!
//! This module provides simplified tool execution for MCP protocol.
//! Following industry patterns (Amazon Q, Codex, rmcp), we avoid trait abstractions
//! and implement direct execution. The actual CLI handler integration will happen
//! in the golem-mcp-server crate using rmcp's #[tool_router] macro.

use golem_mcp_types::{
    error::{McpError, McpResult},
    protocol::JsonRpcId,
};
use serde_json::Value;
use std::time::Duration;
use tracing::{debug, instrument};
use uuid;

/// Direct executor for Golem CLI commands
/// 
/// This is a temporary placeholder that will be replaced by the MCP server
/// implementation using rmcp. Following industry patterns, we don't use
/// trait abstractions or dependency injection.
#[derive(Debug)]
pub struct DirectExecutor {
    /// Default timeout for command execution
    default_timeout: Duration,
}

impl Default for DirectExecutor {
    fn default() -> Self {
        Self {
            default_timeout: Duration::from_secs(300),
        }
    }
}

/// Execution context for direct command execution
#[derive(Debug, Clone)]
pub struct DirectExecutionContext {
    /// Request ID for tracing
    pub request_id: JsonRpcId,
    /// Tool name being executed
    pub tool_name: String,
    /// Tool input parameters
    pub parameters: Value,
    /// Custom timeout for this execution
    pub timeout: Option<Duration>,
}

/// Direct execution result
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectExecutionResult {
    /// Success indicator
    pub success: bool,
    /// Result data (JSON)
    pub result: Value,
    /// Error message if failed
    pub error: Option<String>,
    /// Execution duration
    pub duration: Duration,
}

impl DirectExecutor {
    /// Create a new direct executor
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Execute a tool with the given context using direct handler integration
    ///
    /// # Errors
    ///
    /// Returns `McpError::ToolExecution` if the tool execution fails.
    #[instrument(skip(self), fields(tool = %context.tool_name, request_id = %context.request_id))]
    pub async fn execute(&self, context: DirectExecutionContext) -> McpResult<DirectExecutionResult> {
        debug!("Starting direct tool execution: {}", context.tool_name);
        let start_time = std::time::Instant::now();

        let result = match context.tool_name.as_str() {
            // Domain-based tools following Amazon Q pattern
            "golem_component_management" => self.execute_component_management(&context.parameters).await,
            "golem_worker_management" => self.execute_worker_management(&context.parameters).await,
            "golem_application_management" => self.execute_application_management(&context.parameters).await,
            "golem_project_management" => self.execute_project_management(&context.parameters).await,
            
            // Legacy individual tools (still supported for backward compatibility)
            "golem_component_list" => self.execute_component_list(&context.parameters),
            "golem_component_add" => self.execute_component_add(&context.parameters),
            "golem_worker_list" => self.execute_worker_list(&context.parameters),
            "golem_worker_invoke" => self.execute_worker_invoke(&context.parameters),
            "golem_template_list" => self.execute_template_list(&context.parameters),
            
            _ => Err(McpError::invalid_request(format!(
                "Unknown tool: {}",
                context.tool_name
            ))),
        };

        let duration = start_time.elapsed();

        match result {
            Ok(result) => Ok(DirectExecutionResult {
                success: true,
                result,
                error: None,
                duration,
            }),
            Err(e) => Ok(DirectExecutionResult {
                success: false,
                result: serde_json::json!({}),
                error: Some(e.to_string()),
                duration,
            }),
        }
    }

    /// Execute component management domain tool
    /// 
    /// NOTE: This is a placeholder implementation. The real implementation
    /// will be in golem-mcp-server using rmcp's #[tool] macro.
    async fn execute_component_management(&self, parameters: &Value) -> McpResult<Value> {
        debug!("Executing component management with parameters: {}", parameters);
        
        let operation = parameters
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_request("Missing 'operation' parameter".to_string()))?;
        
        // Placeholder response - real implementation will be in MCP server
        Ok(serde_json::json!({
            "tool": "golem_component_management",
            "status": "executed",
            "parameters": parameters,
            "result": {
                "action": format!("component_{}", operation),
                "message": format!("Component {} operation would be executed", operation),
                "domain": "component_management"
            }
        }))
    }
    
    /// Execute worker management domain tool
    /// 
    /// NOTE: This is a placeholder implementation. The real implementation
    /// will be in golem-mcp-server using rmcp's #[tool] macro.
    async fn execute_worker_management(&self, parameters: &Value) -> McpResult<Value> {
        debug!("Executing worker management with parameters: {}", parameters);
        
        let operation = parameters
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_request("Missing 'operation' parameter".to_string()))?;
        
        // Placeholder response - real implementation will be in MCP server
        Ok(serde_json::json!({
            "tool": "golem_worker_management",
            "status": "executed",
            "parameters": parameters,
            "result": {
                "action": format!("worker_{}", operation),
                "message": format!("Worker {} operation would be executed", operation),
                "domain": "worker_management"
            }
        }))
    }
    
    /// Execute application management domain tool
    /// 
    /// NOTE: This is a placeholder implementation. The real implementation
    /// will be in golem-mcp-server using rmcp's #[tool] macro.
    async fn execute_application_management(&self, parameters: &Value) -> McpResult<Value> {
        debug!("Executing application management with parameters: {}", parameters);
        
        let operation = parameters
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_request("Missing 'operation' parameter".to_string()))?;
        
        // Placeholder response - real implementation will be in MCP server
        Ok(serde_json::json!({
            "tool": "golem_application_management",
            "status": "executed",
            "parameters": parameters,
            "result": {
                "action": format!("application_{}", operation),
                "message": format!("Application {} operation would be executed", operation),
                "domain": "application_management"
            }
        }))
    }
    
    /// Execute project management domain tool
    /// 
    /// NOTE: This is a placeholder implementation. The real implementation
    /// will be in golem-mcp-server using rmcp's #[tool] macro.
    async fn execute_project_management(&self, parameters: &Value) -> McpResult<Value> {
        debug!("Executing project management with parameters: {}", parameters);
        
        let operation = parameters
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_request("Missing 'operation' parameter".to_string()))?;
        
        // Placeholder response - real implementation will be in MCP server
        Ok(serde_json::json!({
            "tool": "golem_project_management",
            "status": "executed",
            "parameters": parameters,
            "result": {
                "action": format!("project_{}", operation),
                "message": format!("Project {} operation would be executed", operation),
                "domain": "project_management"
            }
        }))
    }

    /// Execute component list command
    fn execute_component_list(&self, parameters: &Value) -> McpResult<Value> {
        debug!("Executing component list with parameters: {}", parameters);
        
        let project_id = parameters
            .get("project_id")
            .and_then(|v| v.as_str());

        // Return structured response with parsed parameters
        Ok(serde_json::json!({
            "tool": "golem_component_list",
            "status": "executed",
            "parameters": {
                "project_id": project_id
            },
            "result": {
                "action": "list_components",
                "message": "Component listing would be executed",
                "project_filter": project_id
            }
        }))
    }

    /// Execute component add command
    fn execute_component_add(&self, parameters: &Value) -> McpResult<Value> {
        debug!("Executing component add with parameters: {}", parameters);
        
        let component_name = parameters
            .get("component_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_request("Missing required parameter: component_name".to_owned()))?;
        
        let wasm_file = parameters
            .get("wasm_file")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_request("Missing required parameter: wasm_file".to_owned()))?;

        let project_id = parameters
            .get("project_id")
            .and_then(|v| v.as_str());

        Ok(serde_json::json!({
            "tool": "golem_component_add",
            "status": "executed",
            "parameters": {
                "component_name": component_name,
                "wasm_file": wasm_file,
                "project_id": project_id
            },
            "result": {
                "action": "add_component",
                "message": format!("Component '{}' would be added from '{}'", component_name, wasm_file),
                "component_name": component_name,
                "wasm_path": wasm_file
            }
        }))
    }

    /// Execute worker list command
    fn execute_worker_list(&self, parameters: &Value) -> McpResult<Value> {
        debug!("Executing worker list with parameters: {}", parameters);
        
        let component_id = parameters
            .get("component_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_request("Missing required parameter: component_id".to_owned()))?;

        // Validate component_id format (should be UUID)
        if uuid::Uuid::parse_str(component_id).is_err() {
            return Err(McpError::invalid_request(format!(
                "Invalid component_id format: '{}' (expected UUID)",
                component_id
            )));
        }

        Ok(serde_json::json!({
            "tool": "golem_worker_list",
            "status": "executed",
            "parameters": {
                "component_id": component_id
            },
            "result": {
                "action": "list_workers",
                "message": format!("Workers for component '{}' would be listed", component_id),
                "component_id": component_id
            }
        }))
    }

    /// Execute worker invoke command
    fn execute_worker_invoke(&self, parameters: &Value) -> McpResult<Value> {
        debug!("Executing worker invoke with parameters: {}", parameters);
        
        let component_id = parameters
            .get("component_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_request("Missing required parameter: component_id".to_owned()))?;
        
        let worker_name = parameters
            .get("worker_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_request("Missing required parameter: worker_name".to_owned()))?;
        
        let function_name = parameters
            .get("function_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_request("Missing required parameter: function_name".to_owned()))?;
        
        let function_parameters = parameters
            .get("function_parameters")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(str::to_owned).collect::<Vec<_>>())
            .unwrap_or_default();

        // Validate component_id format (should be UUID)
        if uuid::Uuid::parse_str(component_id).is_err() {
            return Err(McpError::invalid_request(format!(
                "Invalid component_id format: '{}' (expected UUID)",
                component_id
            )));
        }

        Ok(serde_json::json!({
            "tool": "golem_worker_invoke",
            "status": "executed",
            "parameters": {
                "component_id": component_id,
                "worker_name": worker_name,
                "function_name": function_name,
                "function_parameters": function_parameters
            },
            "result": {
                "action": "invoke_function",
                "message": format!("Function '{}' would be invoked on worker '{}'", function_name, worker_name),
                "component_id": component_id,
                "worker_name": worker_name,
                "function_name": function_name,
                "function_parameters": function_parameters
            }
        }))
    }

    /// Execute template list command
    fn execute_template_list(&self, parameters: &Value) -> McpResult<Value> {
        debug!("Executing template list with parameters: {}", parameters);
        
        let language = parameters
            .get("language")
            .and_then(|v| v.as_str());

        // Validate language if provided
        if let Some(lang) = language {
            let valid_languages = ["rust", "python", "typescript", "javascript", "go"];
            if !valid_languages.contains(&lang) {
                return Err(McpError::invalid_request(format!(
                    "Invalid language '{}'. Valid languages are: {}",
                    lang,
                    valid_languages.join(", ")
                )));
            }
        }

        Ok(serde_json::json!({
            "tool": "golem_template_list",
            "status": "executed",
            "parameters": {
                "language": language
            },
            "result": {
                "action": "list_templates",
                "message": "Available templates would be listed",
                "language_filter": language,
                "available_languages": ["rust", "python", "typescript", "javascript", "go"]
            }
        }))
    }

    /// Get current execution statistics
    #[must_use]
    pub fn execution_stats(&self) -> DirectExecutionStats {
        DirectExecutionStats {
            default_timeout: self.default_timeout,
        }
    }
}

/// Direct execution statistics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirectExecutionStats {
    /// Default timeout duration
    pub default_timeout: Duration,
}

impl DirectExecutionResult {
    /// Check if execution was successful
    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.success
    }

    /// Get error message if execution failed
    #[must_use]
    pub fn error_message(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direct_executor_creation() {
        let _executor = DirectExecutor::new();
        // If we get here without panicking, creation succeeded
    }

    #[test]
    fn test_direct_execution_result_success() {
        let result = DirectExecutionResult {
            success: true,
            result: serde_json::json!({"status": "ok"}),
            error: None,
            duration: Duration::from_secs(1),
        };

        assert!(result.is_success());
        assert!(result.error_message().is_none());
    }

    #[test]
    fn test_direct_execution_result_failure() {
        let result = DirectExecutionResult {
            success: false,
            result: serde_json::json!({}),
            error: Some("Test error".to_owned()),
            duration: Duration::from_secs(1),
        };

        assert!(!result.is_success());
        assert_eq!(result.error_message(), Some("Test error"));
    }
}