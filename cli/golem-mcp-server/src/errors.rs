//! Comprehensive error handling system for Golem MCP Server
//!
//! Implements structured error hierarchy with detailed categorization,
//! observability support, and recovery mechanisms for robust error handling.

use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt};
use thiserror::Error;
use tracing::{error, warn};

/// Primary error type for all Golem MCP operations
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum GolemMcpError {
    /// CLI command execution failed
    #[error("CLI command failed: {operation} - {message}")]
    CliCommandFailed {
        operation: String,
        message: String,
        exit_code: Option<i32>,
        stderr: Option<String>,
        #[serde(skip)]
        recovery_hint: Option<RecoveryHint>,
    },

    /// Invalid parameters provided to operation
    #[error("Invalid parameters for {operation}: {message}")]
    InvalidParameters {
        operation: String,
        message: String,
        missing_fields: Vec<String>,
        invalid_fields: Vec<String>,
    },

    /// Resource not found (component, worker, etc.)
    #[error("Resource not found: {resource_type} '{resource_id}'")]
    ResourceNotFound {
        resource_type: String,
        resource_id: String,
        #[serde(skip)]
        recovery_hint: Option<RecoveryHint>,
    },

    /// Operation timeout
    #[error("Operation timeout after {duration_ms}ms: {operation}")]
    OperationTimeout {
        operation: String,
        duration_ms: u64,
        #[serde(skip)]
        recovery_hint: Option<RecoveryHint>,
    },

    /// Configuration error
    #[error("Configuration error: {message}")]
    ConfigurationError {
        message: String,
        config_key: Option<String>,
        expected_format: Option<String>,
    },

    /// Resource provider error
    #[error("Resource provider error: {message}")]
    ResourceProvider {
        message: String,
        resource_uri: Option<String>,
    },

    /// Progress streaming error
    #[error("Progress streaming error: {message}")]
    ProgressStreaming {
        message: String,
        stage: ProgressStage,
        #[serde(skip)]
        recovery_hint: Option<RecoveryHint>,
    },

    /// Transport layer error
    #[error("Transport error ({transport_type}): {message}")]
    Transport {
        transport_type: TransportType,
        message: String,
        #[serde(skip)]
        recovery_hint: Option<RecoveryHint>,
    },

    /// Internal server error
    #[error("Internal server error: {message}")]
    InternalError {
        message: String,
        error_id: String,
    },
}

/// Error categories for observability and metrics
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ErrorCategory {
    /// User input or parameter errors
    UserError,
    /// System or infrastructure errors
    SystemError,
    /// Configuration or setup errors
    ConfigurationError,
    /// Network or transport errors
    NetworkError,
    /// Resource availability errors
    ResourceError,
    /// Performance or timeout errors
    PerformanceError,
    /// Internal logic or state errors
    InternalError,
}

/// Recovery hints for error resolution
#[derive(Debug, Clone)]
pub enum RecoveryHint {
    /// Retry the operation with backoff
    RetryWithBackoff {
        max_retries: u32,
        initial_delay_ms: u64,
        backoff_multiplier: f64,
    },
    /// Check configuration or environment
    CheckConfiguration {
        config_keys: Vec<String>,
        documentation_url: Option<String>,
    },
    /// Verify resource availability
    VerifyResource {
        resource_type: String,
        suggested_commands: Vec<String>,
    },
    /// Contact support or check logs
    ContactSupport {
        error_id: String,
        log_context: Vec<String>,
    },
}

/// Progress streaming stages for error context
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProgressStage {
    Initialization,
    Preparation,
    Execution,
    Completion,
    Cleanup,
}

/// Transport types for error categorization
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransportType {
    Stdio,
    Http,
    Unknown,
}

impl fmt::Display for TransportType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransportType::Stdio => write!(f, "stdio"),
            TransportType::Http => write!(f, "http"),
            TransportType::Unknown => write!(f, "unknown"),
        }
    }
}

impl GolemMcpError {
    /// Create a CLI command failure error with recovery hints
    pub fn cli_command_failed<S: Into<String>>(
        operation: S,
        message: S,
        exit_code: Option<i32>,
        stderr: Option<String>,
    ) -> Self {
        let op = operation.into();
        let msg = message.into();
        
        let recovery_hint = match exit_code {
            Some(127) => Some(RecoveryHint::CheckConfiguration {
                config_keys: vec!["PATH".to_string(), "GOLEM_CLI_PATH".to_string()],
                documentation_url: Some("https://github.com/golemcloud/golem".to_string()),
            }),
            Some(1..=126) => Some(RecoveryHint::VerifyResource {
                resource_type: "command parameters".to_string(),
                suggested_commands: vec![format!("golem {op} --help")],
            }),
            _ => Some(RecoveryHint::RetryWithBackoff {
                max_retries: 3,
                initial_delay_ms: 1000,
                backoff_multiplier: 2.0,
            }),
        };

        Self::CliCommandFailed {
            operation: op,
            message: msg,
            exit_code,
            stderr,
            recovery_hint,
        }
    }

    /// Create an invalid parameters error
    pub fn invalid_parameters<S: Into<String>>(
        operation: S,
        message: S,
        missing_fields: Vec<String>,
        invalid_fields: Vec<String>,
    ) -> Self {
        Self::InvalidParameters {
            operation: operation.into(),
            message: message.into(),
            missing_fields,
            invalid_fields,
        }
    }

    /// Create a resource not found error with suggestions
    pub fn resource_not_found<S: Into<String>>(
        resource_type: S,
        resource_id: S,
        list_command: Option<String>,
    ) -> Self {
        let rt = resource_type.into();
        let rid = resource_id.into();
        
        let recovery_hint = list_command.map(|cmd| RecoveryHint::VerifyResource {
            resource_type: rt.clone(),
            suggested_commands: vec![cmd],
        });

        Self::ResourceNotFound {
            resource_type: rt,
            resource_id: rid,
            recovery_hint,
        }
    }

    /// Create an operation timeout error
    pub fn operation_timeout<S: Into<String>>(
        operation: S,
        duration_ms: u64,
    ) -> Self {
        Self::OperationTimeout {
            operation: operation.into(),
            duration_ms,
            recovery_hint: Some(RecoveryHint::RetryWithBackoff {
                max_retries: 2,
                initial_delay_ms: 5000,
                backoff_multiplier: 1.5,
            }),
        }
    }

    /// Create a progress streaming error
    pub fn progress_streaming<S: Into<String>>(
        message: S,
        stage: ProgressStage,
    ) -> Self {
        Self::ProgressStreaming {
            message: message.into(),
            stage,
            recovery_hint: Some(RecoveryHint::RetryWithBackoff {
                max_retries: 1,
                initial_delay_ms: 2000,
                backoff_multiplier: 1.0,
            }),
        }
    }

    /// Create a transport error
    pub fn transport<S: Into<String>>(
        transport_type: TransportType,
        message: S,
    ) -> Self {
        let msg = message.into();
        let recovery_hint = match transport_type {
            TransportType::Http => Some(RecoveryHint::CheckConfiguration {
                config_keys: vec!["HTTP_PORT".to_string(), "HTTP_HOST".to_string()],
                documentation_url: Some("https://modelcontextprotocol.io/".to_string()),
            }),
            TransportType::Stdio => Some(RecoveryHint::CheckConfiguration {
                config_keys: vec!["STDIN".to_string(), "STDOUT".to_string()],
                documentation_url: Some("https://modelcontextprotocol.io/".to_string()),
            }),
            TransportType::Unknown => None,
        };

        Self::Transport {
            transport_type,
            message: msg,
            recovery_hint,
        }
    }

    /// Get error category for metrics and observability
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::InvalidParameters { .. } => ErrorCategory::UserError,
            Self::ResourceNotFound { .. } => ErrorCategory::ResourceError,
            Self::OperationTimeout { .. } => ErrorCategory::PerformanceError,
            Self::ConfigurationError { .. } => ErrorCategory::ConfigurationError,
            Self::Transport { .. } => ErrorCategory::NetworkError,
            Self::CliCommandFailed { exit_code, .. } => {
                match exit_code {
                    Some(127) => ErrorCategory::ConfigurationError,
                    Some(1..=126) => ErrorCategory::UserError,
                    _ => ErrorCategory::SystemError,
                }
            }
            Self::ResourceProvider { .. } => ErrorCategory::ResourceError,
            Self::ProgressStreaming { .. } => ErrorCategory::SystemError,
            Self::InternalError { .. } => ErrorCategory::InternalError,
        }
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::InvalidParameters { .. } => ErrorSeverity::Warning,
            Self::ResourceNotFound { .. } => ErrorSeverity::Warning,
            Self::ConfigurationError { .. } => ErrorSeverity::Error,
            Self::OperationTimeout { .. } => ErrorSeverity::Warning,
            Self::CliCommandFailed { exit_code, .. } => {
                match exit_code {
                    Some(0) => ErrorSeverity::Info,
                    Some(1..=126) => ErrorSeverity::Warning,
                    _ => ErrorSeverity::Error,
                }
            }
            Self::Transport { .. } => ErrorSeverity::Error,
            Self::ResourceProvider { .. } => ErrorSeverity::Warning,
            Self::ProgressStreaming { .. } => ErrorSeverity::Warning,
            Self::InternalError { .. } => ErrorSeverity::Critical,
        }
    }

    /// Get recovery hint if available
    pub fn recovery_hint(&self) -> Option<&RecoveryHint> {
        match self {
            Self::CliCommandFailed { recovery_hint, .. } => recovery_hint.as_ref(),
            Self::ResourceNotFound { recovery_hint, .. } => recovery_hint.as_ref(),
            Self::OperationTimeout { recovery_hint, .. } => recovery_hint.as_ref(),
            Self::ProgressStreaming { recovery_hint, .. } => recovery_hint.as_ref(),
            Self::Transport { recovery_hint, .. } => recovery_hint.as_ref(),
            _ => None,
        }
    }

    /// Log error with appropriate level and context
    pub fn log_error(&self) {
        let context = self.get_log_context();
        
        match self.severity() {
            ErrorSeverity::Critical => {
                error!(
                    error = %self,
                    category = ?self.category(),
                    context = ?context,
                    "Critical error occurred"
                );
            }
            ErrorSeverity::Error => {
                error!(
                    error = %self,
                    category = ?self.category(),
                    context = ?context,
                    "Error occurred"
                );
            }
            ErrorSeverity::Warning => {
                warn!(
                    error = %self,
                    category = ?self.category(),
                    context = ?context,
                    "Warning occurred"
                );
            }
            ErrorSeverity::Info => {
                tracing::info!(
                    error = %self,
                    category = ?self.category(),
                    context = ?context,
                    "Info occurred"
                );
            }
        }
    }

    /// Get structured context for logging
    fn get_log_context(&self) -> LogContext {
        match self {
            Self::CliCommandFailed { operation, exit_code, .. } => LogContext {
                operation: Some(operation.clone()),
                exit_code: *exit_code,
                ..Default::default()
            },
            Self::InvalidParameters { operation, missing_fields, invalid_fields, .. } => LogContext {
                operation: Some(operation.clone()),
                missing_fields: missing_fields.clone(),
                invalid_fields: invalid_fields.clone(),
                ..Default::default()
            },
            Self::ResourceNotFound { resource_type, resource_id, .. } => LogContext {
                resource_type: Some(resource_type.clone()),
                resource_id: Some(resource_id.clone()),
                ..Default::default()
            },
            Self::OperationTimeout { operation, duration_ms, .. } => LogContext {
                operation: Some(operation.clone()),
                timeout_ms: Some(*duration_ms),
                ..Default::default()
            },
            Self::ProgressStreaming { stage, .. } => LogContext {
                progress_stage: Some(*stage),
                ..Default::default()
            },
            Self::Transport { transport_type, .. } => LogContext {
                transport_type: Some(*transport_type),
                ..Default::default()
            },
            _ => LogContext::default(),
        }
    }
}

/// Error severity levels for logging and alerting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Structured logging context
#[derive(Debug, Default, Clone)]
struct LogContext {
    operation: Option<String>,
    exit_code: Option<i32>,
    missing_fields: Vec<String>,
    invalid_fields: Vec<String>,
    resource_type: Option<String>,
    resource_id: Option<String>,
    timeout_ms: Option<u64>,
    progress_stage: Option<ProgressStage>,
    transport_type: Option<TransportType>,
}

/// Convert GolemMcpError to rmcp ErrorData (McpError)
impl From<GolemMcpError> for rmcp::ErrorData {
    fn from(error: GolemMcpError) -> Self {
        // Log the error for observability
        error.log_error();

        rmcp::ErrorData {
            code: rmcp::model::ErrorCode(error.get_mcp_error_code()),
            message: Cow::Owned(error.to_string()),
            data: Some(serde_json::to_value(&error).unwrap_or_default()),
        }
    }
}

impl GolemMcpError {
    /// Get appropriate MCP error code
    fn get_mcp_error_code(&self) -> i32 {
        match self {
            Self::InvalidParameters { .. } => -32602, // Invalid params
            Self::ResourceNotFound { .. } => -32601,  // Method not found (closest to resource not found)
            Self::OperationTimeout { .. } => -32603,  // Internal error
            Self::ConfigurationError { .. } => -32603, // Internal error
            Self::Transport { .. } => -32603,          // Internal error
            Self::CliCommandFailed { .. } => -32603,  // Internal error
            Self::ResourceProvider { .. } => -32603,  // Internal error
            Self::ProgressStreaming { .. } => -32603, // Internal error
            Self::InternalError { .. } => -32603,     // Internal error
        }
    }
}

/// Result type alias for Golem MCP operations
pub type GolemMcpResult<T> = Result<T, GolemMcpError>;

/// Error builder for convenient error construction
pub struct ErrorBuilder {
    operation: Option<String>,
    message: Option<String>,
    category: Option<ErrorCategory>,
}

impl ErrorBuilder {
    pub fn new() -> Self {
        Self {
            operation: None,
            message: None,
            category: None,
        }
    }

    pub fn operation<S: Into<String>>(mut self, operation: S) -> Self {
        self.operation = Some(operation.into());
        self
    }

    pub fn message<S: Into<String>>(mut self, message: S) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn category(mut self, category: ErrorCategory) -> Self {
        self.category = Some(category);
        self
    }

    pub fn cli_failed(
        self,
        exit_code: Option<i32>,
        stderr: Option<String>,
    ) -> GolemMcpError {
        GolemMcpError::cli_command_failed(
            self.operation.unwrap_or_default(),
            self.message.unwrap_or_default(),
            exit_code,
            stderr,
        )
    }

    pub fn invalid_params(
        self,
        missing: Vec<String>,
        invalid: Vec<String>,
    ) -> GolemMcpError {
        GolemMcpError::invalid_parameters(
            self.operation.unwrap_or_default(),
            self.message.unwrap_or_default(),
            missing,
            invalid,
        )
    }

    pub fn resource_not_found(
        self,
        resource_type: String,
        resource_id: String,
    ) -> GolemMcpError {
        GolemMcpError::resource_not_found(resource_type, resource_id, None)
    }
}

impl Default for ErrorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_category_classification() {
        let cli_error = GolemMcpError::cli_command_failed(
            "test", "command failed", Some(127), None
        );
        assert_eq!(cli_error.category(), ErrorCategory::ConfigurationError);

        let invalid_params = GolemMcpError::invalid_parameters(
            "test", "missing params", vec!["name".to_string()], vec![]
        );
        assert_eq!(invalid_params.category(), ErrorCategory::UserError);
    }

    #[test]
    fn test_error_severity_levels() {
        let timeout_error = GolemMcpError::operation_timeout("test", 5000);
        assert_eq!(timeout_error.severity(), ErrorSeverity::Warning);

        let internal_error = GolemMcpError::InternalError {
            message: "test".to_string(),
            error_id: "test-id".to_string(),
        };
        assert_eq!(internal_error.severity(), ErrorSeverity::Critical);
    }

    #[test]
    fn test_error_builder() {
        let error = ErrorBuilder::new()
            .operation("test-op")
            .message("test message")
            .cli_failed(Some(1), Some("stderr output".to_string()));

        match error {
            GolemMcpError::CliCommandFailed { operation, exit_code, stderr, .. } => {
                assert_eq!(operation, "test-op");
                assert_eq!(exit_code, Some(1));
                assert_eq!(stderr.as_deref(), Some("stderr output"));
            }
            _ => panic!("Expected CliCommandFailed error"),
        }
    }

    #[test]
    fn test_mcp_error_conversion() {
        let golem_error = GolemMcpError::invalid_parameters(
            "test", "invalid params", vec!["name".to_string()], vec![]
        );
        
        let mcp_error: rmcp::ErrorData = golem_error.into();
        assert_eq!(mcp_error.code.0, -32602);
        assert!(!mcp_error.message.is_empty());
        assert!(mcp_error.data.is_some());
    }
}