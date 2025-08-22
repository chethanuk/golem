//! Progress streaming implementation for long-running Golem operations
//!
//! Implements Amazon Q style progress streaming with timeout handling,
//! following the pattern specified in Compare.MD lines 313-366.
//!
//! # Architecture
//!
//! - Type-safe progress tokens using newtypes
//! - Timeout configuration (120s default for tool execution)
//! - Progress notifications via rmcp::model::ProgressNotificationParam
//! - Zero-cost abstractions for short operations
//! - Comprehensive error handling

use rmcp::model::{ProgressNotificationParam, ProgressToken, NumberOrString};
use std::future::Future;
use std::time::Duration;
use tokio::sync::mpsc;
use thiserror::Error;
use tracing::{debug, warn, instrument};

/// Progress streaming errors following staff engineer error handling patterns
#[derive(Error, Debug)]
pub enum ProgressStreamError {
    /// Operation timed out during execution
    #[error("Operation '{operation_name}' timed out after {timeout_secs} seconds")]
    Timeout {
        operation_name: String,
        timeout_secs: u64,
    },
    
    /// Progress notification channel closed unexpectedly
    #[error("Progress notification channel closed for operation '{operation_name}'")]
    ChannelClosed {
        operation_name: String,
    },
    
    /// Failed to send progress notification
    #[error("Failed to send progress notification for operation '{operation_name}': {source}")]
    NotificationFailed {
        operation_name: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    /// Operation execution failed
    #[error("Operation '{operation_name}' failed: {source}")]
    OperationFailed {
        operation_name: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl ProgressStreamError {
    /// Create a timeout error
    pub fn timeout(operation_name: impl Into<String>, timeout: Duration) -> Self {
        Self::Timeout {
            operation_name: operation_name.into(),
            timeout_secs: timeout.as_secs(),
        }
    }
    
    /// Create a channel closed error
    pub fn channel_closed(operation_name: impl Into<String>) -> Self {
        Self::ChannelClosed {
            operation_name: operation_name.into(),
        }
    }
    
    /// Create a notification failed error
    pub fn notification_failed(
        operation_name: impl Into<String>,
        source: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::NotificationFailed {
            operation_name: operation_name.into(),
            source: source.into(),
        }
    }
    
    /// Create an operation failed error
    pub fn operation_failed(
        operation_name: impl Into<String>,
        source: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::OperationFailed {
            operation_name: operation_name.into(),
            source: source.into(),
        }
    }
}

/// Type alias for progress streaming results
pub type ProgressResult<T> = Result<T, ProgressStreamError>;

/// Timeout configuration following Amazon Q pattern from Compare.MD lines 297-305
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    /// Initialization timeout (5-10 seconds)
    pub init_timeout: Duration,
    /// Interactive timeout (30-60 seconds)
    pub interactive_timeout: Duration,
    /// Non-interactive timeout (10-15 seconds)
    pub non_interactive_timeout: Duration,
    /// Tool execution timeout (120 seconds)
    pub tool_execution_timeout: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            init_timeout: Duration::from_secs(10),
            interactive_timeout: Duration::from_secs(60),
            non_interactive_timeout: Duration::from_secs(15),
            tool_execution_timeout: Duration::from_secs(120),
        }
    }
}

impl TimeoutConfig {
    /// Create configuration for fast operations
    pub fn fast_operations() -> Self {
        Self {
            init_timeout: Duration::from_secs(5),
            interactive_timeout: Duration::from_secs(30),
            non_interactive_timeout: Duration::from_secs(10),
            tool_execution_timeout: Duration::from_secs(60),
        }
    }
    
    /// Create configuration for slow operations
    pub fn slow_operations() -> Self {
        Self {
            init_timeout: Duration::from_secs(15),
            interactive_timeout: Duration::from_secs(120),
            non_interactive_timeout: Duration::from_secs(30),
            tool_execution_timeout: Duration::from_secs(300), // 5 minutes for builds
        }
    }
}

/// Type-safe progress token newtype
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationToken(String);

impl OperationToken {
    /// Create a new operation token
    pub fn new(operation_name: impl Into<String>) -> Self {
        Self(operation_name.into())
    }
    
    /// Get the operation name
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<OperationToken> for ProgressToken {
    fn from(token: OperationToken) -> Self {
        ProgressToken(NumberOrString::String(token.0.into()))
    }
}

impl From<&str> for OperationToken {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for OperationToken {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Progress update data
#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    /// Current progress (0-100)
    pub progress: f64,
    /// Total items/steps if known
    pub total: Option<f64>,
    /// Optional message for this update
    pub message: Option<String>,
}

impl ProgressUpdate {
    /// Create a new progress update
    pub fn new(progress: f64) -> Self {
        Self {
            progress,
            total: Some(100.0),
            message: None,
        }
    }
    
    /// Set the total number of steps
    pub fn with_total(mut self, total: f64) -> Self {
        self.total = Some(total);
        self
    }
    
    /// Set a message for this update
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }
    
    /// Create a starting progress update (0%)
    pub fn starting() -> Self {
        Self::new(0.0).with_message("Starting operation...")
    }
    
    /// Create a completion progress update (100%)
    pub fn completed() -> Self {
        Self::new(100.0).with_message("Operation completed")
    }
}

/// Progress notification sender
/// Wraps mpsc::Sender to provide type-safe progress updates
#[derive(Debug, Clone)]
pub struct ProgressSender {
    sender: mpsc::Sender<ProgressNotificationParam>,
    token: OperationToken,
}

impl ProgressSender {
    /// Create a new progress sender
    pub fn new(sender: mpsc::Sender<ProgressNotificationParam>, token: OperationToken) -> Self {
        Self { sender, token }
    }
    
    /// Send a progress update
    #[instrument(skip(self), fields(operation = %self.token.as_str()))]
    pub async fn send_update(&self, update: ProgressUpdate) -> ProgressResult<()> {
        let notification = ProgressNotificationParam {
            progress_token: self.token.clone().into(),
            progress: update.progress,
            total: update.total,
            message: update.message.clone(),
        };
        
        debug!(
            progress = update.progress,
            total = ?update.total,
            message = ?update.message,
            "Sending progress update"
        );
        
        self.sender
            .send(notification)
            .await
            .map_err(|_| ProgressStreamError::channel_closed(self.token.as_str()))
    }
    
    /// Send a starting notification
    pub async fn send_starting(&self) -> ProgressResult<()> {
        self.send_update(ProgressUpdate::starting()).await
    }
    
    /// Send a completion notification
    pub async fn send_completed(&self) -> ProgressResult<()> {
        self.send_update(ProgressUpdate::completed()).await
    }
    
    /// Send a progress update with a specific percentage
    pub async fn send_progress(&self, progress: f64) -> ProgressResult<()> {
        self.send_update(ProgressUpdate::new(progress)).await
    }
    
    /// Send a progress update with a message
    pub async fn send_progress_with_message(
        &self,
        progress: f64,
        message: impl Into<String>,
    ) -> ProgressResult<()> {
        self.send_update(ProgressUpdate::new(progress).with_message(message)).await
    }
}

/// Execute a long-running operation with progress streaming
/// Following Amazon Q pattern from Compare.MD lines 318-348
pub async fn execute_with_progress<F, T, E>(
    operation: F,
    progress_sender: ProgressSender,
    timeout: Duration,
) -> ProgressResult<T>
where
    F: Future<Output = Result<T, E>>,
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    let operation_name = progress_sender.token.as_str();
    
    debug!(
        operation = operation_name,
        timeout_secs = timeout.as_secs(),
        "Starting operation with progress tracking"
    );
    
    // Send initial progress notification
    if let Err(e) = progress_sender.send_starting().await {
        warn!(
            operation = operation_name,
            error = %e,
            "Failed to send starting progress notification, continuing..."
        );
    }
    
    // Execute operation with timeout using tokio::select!
    let result = tokio::select! {
        result = operation => {
            match result {
                Ok(value) => {
                    debug!(operation = operation_name, "Operation completed successfully");
                    
                    // Send completion notification
                    if let Err(e) = progress_sender.send_completed().await {
                        warn!(
                            operation = operation_name,
                            error = %e,
                            "Failed to send completion progress notification"
                        );
                    }
                    
                    Ok(value)
                }
                Err(e) => {
                    let error = ProgressStreamError::operation_failed(operation_name, e.into());
                    warn!(
                        operation = operation_name,
                        error = %error,
                        "Operation failed"
                    );
                    Err(error)
                }
            }
        }
        _ = tokio::time::sleep(timeout) => {
            let error = ProgressStreamError::timeout(operation_name, timeout);
            warn!(
                operation = operation_name,
                timeout_secs = timeout.as_secs(),
                "Operation timed out"
            );
            Err(error)
        }
    };
    
    result
}

/// Create a progress streaming channel
/// Returns (sender, receiver) pair for progress notifications
pub fn create_progress_channel(
    buffer_size: usize,
) -> (mpsc::Sender<ProgressNotificationParam>, mpsc::Receiver<ProgressNotificationParam>) {
    mpsc::channel(buffer_size)
}

/// Create a progress sender for a specific operation
pub fn create_progress_sender(
    sender: mpsc::Sender<ProgressNotificationParam>,
    operation_name: impl Into<OperationToken>,
) -> ProgressSender {
    ProgressSender::new(sender, operation_name.into())
}

/// Execute operation with progress tracking for component builds
/// Specialized version for 2-5 minute component operations
pub async fn execute_component_build_with_progress<F, T, E>(
    operation: F,
    progress_sender: ProgressSender,
) -> ProgressResult<T>
where
    F: Future<Output = Result<T, E>>,
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    let config = TimeoutConfig::slow_operations();
    execute_with_progress(operation, progress_sender, config.tool_execution_timeout).await
}

/// Execute operation with progress tracking for worker operations
/// Specialized version for 30-120 second worker operations
pub async fn execute_worker_operation_with_progress<F, T, E>(
    operation: F,
    progress_sender: ProgressSender,
) -> ProgressResult<T>
where
    F: Future<Output = Result<T, E>>,
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    let config = TimeoutConfig::default();
    execute_with_progress(operation, progress_sender, config.tool_execution_timeout).await
}

/// Execute operation with progress tracking for template downloads
/// Specialized version for 10-60 second download operations
pub async fn execute_template_download_with_progress<F, T, E>(
    operation: F,
    progress_sender: ProgressSender,
) -> ProgressResult<T>
where
    F: Future<Output = Result<T, E>>,
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    let config = TimeoutConfig::default();
    execute_with_progress(operation, progress_sender, config.interactive_timeout).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_progress_token_creation() {
        let token = OperationToken::new("test_operation");
        assert_eq!(token.as_str(), "test_operation");
        
        let progress_token: ProgressToken = token.into();
        if let ProgressToken(NumberOrString::String(s)) = progress_token {
            assert_eq!(s.as_ref(), "test_operation");
        } else {
            panic!("Expected string token");
        }
    }

    #[tokio::test]
    async fn test_progress_update_creation() {
        let update = ProgressUpdate::new(50.0)
            .with_total(100.0)
            .with_message("Processing...");
        
        assert_eq!(update.progress, 50.0);
        assert_eq!(update.total, Some(100.0));
        assert_eq!(update.message.as_deref(), Some("Processing..."));
    }

    #[tokio::test]
    async fn test_timeout_config() {
        let config = TimeoutConfig::default();
        assert_eq!(config.tool_execution_timeout, Duration::from_secs(120));
        
        let fast_config = TimeoutConfig::fast_operations();
        assert_eq!(fast_config.tool_execution_timeout, Duration::from_secs(60));
        
        let slow_config = TimeoutConfig::slow_operations();
        assert_eq!(slow_config.tool_execution_timeout, Duration::from_secs(300));
    }

    #[tokio::test]
    async fn test_execute_with_progress_success() {
        let (sender, mut receiver) = create_progress_channel(10);
        let progress_sender = create_progress_sender(sender, "test_operation");
        
        let operation = async {
            sleep(Duration::from_millis(100)).await;
            Ok::<_, std::io::Error>("success")
        };
        
        let handle = tokio::spawn(async move {
            let mut notifications = Vec::new();
            while let Some(notification) = receiver.recv().await {
                notifications.push(notification);
                if notifications.len() >= 2 {
                    break; // Starting + completion notifications
                }
            }
            notifications
        });
        
        let result = execute_with_progress(
            operation,
            progress_sender,
            Duration::from_secs(1),
        ).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        
        let notifications = handle.await.unwrap();
        assert_eq!(notifications.len(), 2);
        
        // Check starting notification
        assert_eq!(notifications[0].progress, 0.0);
        
        // Check completion notification
        assert_eq!(notifications[1].progress, 100.0);
    }

    #[tokio::test]
    async fn test_execute_with_progress_timeout() {
        let (sender, _receiver) = create_progress_channel(10);
        let progress_sender = create_progress_sender(sender, "test_timeout");
        
        let operation = async {
            sleep(Duration::from_secs(2)).await;
            Ok::<_, std::io::Error>("should_timeout")
        };
        
        let result = execute_with_progress(
            operation,
            progress_sender,
            Duration::from_millis(100),
        ).await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            ProgressStreamError::Timeout { operation_name, .. } => {
                assert_eq!(operation_name, "test_timeout");
            }
            _ => panic!("Expected timeout error"),
        }
    }

    #[tokio::test]
    async fn test_execute_with_progress_operation_failure() {
        let (sender, _receiver) = create_progress_channel(10);
        let progress_sender = create_progress_sender(sender, "test_failure");
        
        let operation = async {
            Err::<String, _>(std::io::Error::new(
                std::io::ErrorKind::Other,
                "test error"
            ))
        };
        
        let result = execute_with_progress(
            operation,
            progress_sender,
            Duration::from_secs(1),
        ).await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            ProgressStreamError::OperationFailed { operation_name, .. } => {
                assert_eq!(operation_name, "test_failure");
            }
            _ => panic!("Expected operation failure error"),
        }
    }

    #[tokio::test]
    async fn test_progress_sender_manual_updates() {
        let (sender, mut receiver) = create_progress_channel(10);
        let progress_sender = create_progress_sender(sender, "manual_test");
        
        // Send manual progress updates
        progress_sender.send_progress(25.0).await.unwrap();
        progress_sender.send_progress_with_message(50.0, "Half way there").await.unwrap();
        progress_sender.send_progress(75.0).await.unwrap();
        progress_sender.send_completed().await.unwrap();
        
        // Collect notifications
        let mut notifications = Vec::new();
        while let Ok(notification) = receiver.try_recv() {
            notifications.push(notification);
        }
        
        assert_eq!(notifications.len(), 4);
        assert_eq!(notifications[0].progress, 25.0);
        assert_eq!(notifications[1].progress, 50.0);
        assert_eq!(notifications[2].progress, 75.0);
        assert_eq!(notifications[3].progress, 100.0);
    }
}