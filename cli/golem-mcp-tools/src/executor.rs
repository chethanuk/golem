//! Tool execution and process management
//!
//! Provides secure and efficient execution of Golem CLI commands as MCP tools,
//! with proper timeout handling, resource management, and error context preservation.

use golem_mcp_types::{
    error::{McpError, McpResult},
    protocol::JsonRpcId,
};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{debug, error, instrument};

/// Tool executor for running Golem CLI commands
#[derive(Debug, Clone)]
pub struct ToolExecutor {
    /// Path to golem CLI binary
    cli_path: PathBuf,
    /// Default timeout for commands
    default_timeout: Duration,
    /// Working directory for CLI commands
    working_directory: Option<PathBuf>,
    /// Environment variables for CLI commands
    env_vars: HashMap<String, String>,
    /// Maximum concurrent executions
    max_concurrent: u32,
    /// Currently running executions
    running_count: std::sync::Arc<std::sync::atomic::AtomicU32>,
}

/// Tool execution context
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Request ID for tracing
    pub request_id: JsonRpcId,
    /// Tool name being executed
    pub tool_name: String,
    /// Tool input parameters
    pub parameters: Value,
    /// Custom timeout for this execution
    pub timeout: Option<Duration>,
    /// Working directory override
    pub working_directory: Option<PathBuf>,
}

/// Tool execution result
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionResult {
    /// Exit code of the command
    pub exit_code: i32,
    /// Standard output
    pub stdout: String,
    /// Standard error output
    pub stderr: String,
    /// Execution duration
    pub duration: Duration,
    /// Success indicator
    pub success: bool,
}

impl ToolExecutor {
    /// Create a new tool executor
    #[must_use]
    pub fn new(cli_path: PathBuf) -> Self {
        Self {
            cli_path,
            default_timeout: Duration::from_secs(300), // 5 minutes
            working_directory: None,
            env_vars: HashMap::new(),
            max_concurrent: 10,
            running_count: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0)),
        }
    }

    /// Set default timeout for command execution
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    /// Set working directory for CLI commands
    #[must_use]
    pub fn with_working_directory(mut self, dir: PathBuf) -> Self {
        self.working_directory = Some(dir);
        self
    }

    /// Add environment variable
    #[must_use]
    pub fn with_env_var(mut self, key: String, value: String) -> Self {
        self.env_vars.insert(key, value);
        self
    }

    /// Set maximum concurrent executions
    #[must_use]
    pub fn with_max_concurrent(mut self, max: u32) -> Self {
        self.max_concurrent = max;
        self
    }

    /// Execute a tool with the given context
    ///
    /// # Errors
    ///
    /// Returns `McpError::ToolExecution` if the tool execution fails or times out.
    /// Returns `McpError::Internal` if resource limits are exceeded.
    #[instrument(skip(self), fields(tool = %context.tool_name, request_id = %context.request_id))]
    pub async fn execute(&self, context: ExecutionContext) -> McpResult<ExecutionResult> {
        // Check concurrent execution limit
        let current_count = self
            .running_count
            .load(std::sync::atomic::Ordering::Acquire);
        if current_count >= self.max_concurrent {
            return Err(McpError::internal(format!(
                "Maximum concurrent executions reached: {current_count}/{}",
                self.max_concurrent
            )));
        }

        // Increment running count
        self.running_count
            .fetch_add(1, std::sync::atomic::Ordering::AcqRel);

        // Ensure we decrement on exit
        let _guard = ExecutionGuard {
            counter: std::sync::Arc::clone(&self.running_count),
        };

        debug!("Starting tool execution: {}", context.tool_name);
        let start_time = std::time::Instant::now();

        // Build command arguments from parameters
        let args = self.build_command_args(&context)?;

        // Prepare command
        let mut command = Command::new(&self.cli_path);
        command
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());

        // Set working directory
        if let Some(ref wd) = context
            .working_directory
            .or_else(|| self.working_directory.clone())
        {
            command.current_dir(wd);
        }

        // Set environment variables
        for (key, value) in &self.env_vars {
            command.env(key, value);
        }

        // Execute with timeout - spawn the command first, then apply timeout to waiting
        let execution_timeout = context.timeout.unwrap_or(self.default_timeout);

        // Spawn command synchronously (this is fast and doesn't need timeout)
        let child = match command.spawn() {
            Ok(child) => child,
            Err(e) => {
                error!("Failed to spawn command: {}", e);
                return Err(McpError::tool_execution(
                    context.tool_name,
                    format!("Failed to spawn command: {e}"),
                ));
            }
        };

        // Apply timeout to the actual execution (waiting for completion)
        let result = match timeout(
            execution_timeout,
            async move { child.wait_with_output().await },
        )
        .await
        {
            Ok(Ok(output)) => {
                let duration = start_time.elapsed();
                let exit_code = output.status.code().unwrap_or(-1);
                let success = output.status.success();

                ExecutionResult {
                    exit_code,
                    stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                    stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                    duration,
                    success,
                }
            }
            Ok(Err(e)) => {
                error!("Failed to wait for command output: {}", e);
                return Err(McpError::tool_execution(
                    context.tool_name,
                    format!("Failed to wait for command: {e}"),
                ));
            }
            Err(_) => {
                error!("Command execution timed out after {:?}", execution_timeout);
                return Err(McpError::tool_execution(
                    context.tool_name,
                    format!("Command timed out after {:?}", execution_timeout),
                ));
            }
        };

        debug!(
            "Tool execution completed: {} (exit_code: {}, duration: {:?})",
            context.tool_name, result.exit_code, result.duration
        );

        Ok(result)
    }

    /// Build command arguments from tool parameters
    fn build_command_args(&self, context: &ExecutionContext) -> McpResult<Vec<String>> {
        // This is a placeholder implementation
        // In a real implementation, this would parse the tool parameters
        // and build the appropriate golem CLI arguments

        match context.tool_name.as_str() {
            "golem_component_list" => Ok(vec!["component".to_owned(), "list".to_owned()]),
            "golem_worker_list" => Ok(vec!["worker".to_owned(), "list".to_owned()]),
            _ => Err(McpError::invalid_request(format!(
                "Unknown tool: {}",
                context.tool_name
            ))),
        }
    }

    /// Get current execution statistics
    #[must_use]
    pub fn execution_stats(&self) -> ExecutionStats {
        ExecutionStats {
            running_count: self
                .running_count
                .load(std::sync::atomic::Ordering::Acquire),
            max_concurrent: self.max_concurrent,
            default_timeout: self.default_timeout,
        }
    }
}

/// Execution statistics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionStats {
    /// Currently running executions
    pub running_count: u32,
    /// Maximum allowed concurrent executions
    pub max_concurrent: u32,
    /// Default timeout duration
    pub default_timeout: Duration,
}

/// Guard to ensure running count is decremented
struct ExecutionGuard {
    counter: std::sync::Arc<std::sync::atomic::AtomicU32>,
}

impl Drop for ExecutionGuard {
    fn drop(&mut self) {
        self.counter
            .fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
    }
}

impl ExecutionResult {
    /// Check if execution was successful
    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.success
    }

    /// Get combined output (stdout and stderr)
    #[must_use]
    pub fn combined_output(&self) -> String {
        if self.stderr.is_empty() {
            self.stdout.clone()
        } else if self.stdout.is_empty() {
            self.stderr.clone()
        } else {
            format!("{}\n{}", self.stdout, self.stderr)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_executor_creation() {
        let executor = ToolExecutor::new(PathBuf::from("/usr/bin/golem"));
        assert_eq!(executor.cli_path, PathBuf::from("/usr/bin/golem"));
        assert_eq!(executor.default_timeout, Duration::from_secs(300));
    }

    #[test]
    fn test_tool_executor_with_timeout() {
        let executor = ToolExecutor::new(PathBuf::from("/usr/bin/golem"))
            .with_timeout(Duration::from_secs(120));
        assert_eq!(executor.default_timeout, Duration::from_secs(120));
    }

    #[test]
    fn test_execution_result_success() {
        let result = ExecutionResult {
            exit_code: 0,
            stdout: "success".to_owned(),
            stderr: String::new(),
            duration: Duration::from_secs(1),
            success: true,
        };

        assert!(result.is_success());
        assert_eq!(result.combined_output(), "success");
    }

    #[test]
    fn test_execution_result_with_stderr() {
        let result = ExecutionResult {
            exit_code: 1,
            stdout: "output".to_owned(),
            stderr: "error".to_owned(),
            duration: Duration::from_secs(1),
            success: false,
        };

        assert!(!result.is_success());
        assert_eq!(result.combined_output(), "output\nerror");
    }
}
