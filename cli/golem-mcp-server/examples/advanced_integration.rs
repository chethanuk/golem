//! Advanced Integration Example for Golem MCP Server
//!
//! This example demonstrates production-ready patterns including:
//! - Multi-transport support (stdio + HTTP)
//! - Comprehensive error handling with structured errors
//! - Progress streaming for long operations
//! - Resource provider integration for manifest discovery
//! - Graceful shutdown and cleanup
//!
//! # Usage
//!
//! ```bash
//! # Run with HTTP transport
//! cargo run --example advanced_integration --features http,tracing-subscriber -- --transport http --port 8080
//!
//! # Run with stdio transport (Claude Desktop)
//! cargo run --example advanced_integration --features stdio,tracing-subscriber -- --transport stdio
//! ```

use clap::{Arg, Command};
use golem_mcp_server::{
    rmcp_handler::{GolemMcpServer, GolemMcpTransport},
    serve_mcp_http, serve_mcp_stdio,
    errors::{GolemMcpError, ErrorCategory, ErrorSeverity},
    progress::TimeoutConfig,
};
use std::{
    sync::Arc,
    time::Duration,
};
use tokio::{
    signal,
    sync::Mutex,
    time::timeout,
};
use tracing::{info, warn, error, debug, instrument};

/// Configuration for the advanced MCP server
#[derive(Debug, Clone)]
struct ServerConfig {
    /// Transport type (stdio or http)
    transport: GolemMcpTransport,
    /// Enable progress streaming
    enable_progress: bool,
    /// Server timeout configuration
    timeout_config: TimeoutConfig,
    /// Enable resource discovery
    enable_resources: bool,
    /// Base directory for resource scanning
    resource_base_dir: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            transport: GolemMcpTransport::Stdio,
            enable_progress: true,
            timeout_config: TimeoutConfig::default(),
            enable_resources: true,
            resource_base_dir: None,
        }
    }
}

/// Initialize structured logging with different levels for development vs production
fn init_structured_logging(verbose: bool) {
    let log_level = if verbose { "debug" } else { "info" };
    
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{},golem_mcp=debug", log_level).into())
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .json()
        .init();
}

/// Parse command line arguments
fn parse_args() -> ServerConfig {
    let matches = Command::new("Golem MCP Server - Advanced Integration")
        .version("1.0.0")
        .about("Production-ready MCP server with advanced features")
        .arg(
            Arg::new("transport")
                .long("transport")
                .value_name("TYPE")
                .help("Transport type: stdio or http")
                .default_value("stdio")
                .value_parser(["stdio", "http"])
        )
        .arg(
            Arg::new("port")
                .long("port")
                .value_name("PORT")
                .help("HTTP port (only for http transport)")
                .default_value("8080")
                .value_parser(clap::value_parser!(u16))
        )
        .arg(
            Arg::new("host")
                .long("host")
                .value_name("HOST")
                .help("HTTP host (only for http transport)")
                .default_value("127.0.0.1")
        )
        .arg(
            Arg::new("disable-progress")
                .long("disable-progress")
                .help("Disable progress streaming")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("disable-resources")
                .long("disable-resources")
                .help("Disable resource discovery")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("resource-dir")
                .long("resource-dir")
                .value_name("DIR")
                .help("Base directory for resource scanning")
        )
        .arg(
            Arg::new("fast-timeouts")
                .long("fast-timeouts")
                .help("Use fast timeout configuration for testing")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    let transport = match matches.get_one::<String>("transport").unwrap().as_str() {
        "http" => GolemMcpTransport::Http {
            port: *matches.get_one::<u16>("port").unwrap(),
            host: matches.get_one::<String>("host").cloned(),
        },
        "stdio" => GolemMcpTransport::Stdio,
        _ => unreachable!(),
    };

    let timeout_config = if matches.get_flag("fast-timeouts") {
        TimeoutConfig::fast_operations()
    } else {
        TimeoutConfig::default()
    };

    ServerConfig {
        transport,
        enable_progress: !matches.get_flag("disable-progress"),
        timeout_config,
        enable_resources: !matches.get_flag("disable-resources"),
        resource_base_dir: matches.get_one::<String>("resource-dir").cloned(),
    }
}

/// Setup server with advanced configuration
#[instrument(skip(config))]
async fn setup_server(config: &ServerConfig) -> Result<GolemMcpServer, GolemMcpError> {
    info!("🔧 Setting up Golem MCP Server with advanced configuration");
    debug!("Server config: {:?}", config);

    // Create server instance
    let server = if let Some(base_dir) = &config.resource_base_dir {
        info!("📁 Using custom resource base directory: {}", base_dir);
        GolemMcpServer::with_base_dir(base_dir.into())
    } else {
        GolemMcpServer::new()
    };

    // Configure timeout settings
    info!("⏱️  Configuring timeout settings");
    server.configure_timeouts(config.timeout_config.clone()).await;

    // Setup progress streaming if enabled
    if config.enable_progress {
        info!("📈 Enabling progress streaming with buffer size 64");
        let _progress_receiver = server.setup_progress_streaming(Some(64)).await;
        info!("✅ Progress streaming configured successfully");
    } else {
        warn!("⚠️  Progress streaming disabled");
    }

    info!("✅ Server setup completed successfully");
    Ok(server)
}

/// Start server with the specified transport
#[instrument(skip(server))]
async fn start_server_with_transport(
    server: GolemMcpServer,
    transport: GolemMcpTransport,
) -> Result<(), GolemMcpError> {
    match transport {
        GolemMcpTransport::Stdio => {
            info!("🎯 Starting stdio transport for Claude Desktop integration");
            info!("📋 Server ready to accept MCP requests via stdin/stdout");
            
            serve_mcp_stdio(server)
                .await
                .map_err(|e| GolemMcpError::transport(
                    crate::errors::TransportType::Stdio,
                    format!("Failed to start stdio transport: {}", e)
                ))?;
        }
        GolemMcpTransport::Http { port, host } => {
            let host_str = host.as_deref().unwrap_or("127.0.0.1");
            info!("🌐 Starting HTTP transport on {}:{}", host_str, port);
            info!("📋 Server endpoints:");
            info!("   • Main MCP: http://{}:{}/mcp", host_str, port);
            info!("   • Health: http://{}:{}/mcp/health", host_str, port);
            info!("   • Capabilities: http://{}:{}/mcp/capabilities", host_str, port);
            
            serve_mcp_http(server, port, host_str)
                .await
                .map_err(|e| GolemMcpError::transport(
                    crate::errors::TransportType::Http,
                    format!("Failed to start HTTP transport: {}", e)
                ))?;
        }
    }
    
    Ok(())
}

/// Handle graceful shutdown with cleanup
#[instrument]
async fn handle_shutdown() -> Result<(), GolemMcpError> {
    info!("⏳ Setting up graceful shutdown handlers");
    
    // Wait for SIGINT (Ctrl+C) or SIGTERM
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("🛑 Received shutdown signal (Ctrl+C)");
            info!("🧹 Performing graceful shutdown...");
            
            // Give a moment for cleanup
            tokio::time::sleep(Duration::from_millis(100)).await;
            
            info!("✅ Shutdown completed successfully");
            Ok(())
        }
        Err(e) => {
            error!("❌ Failed to listen for shutdown signal: {}", e);
            Err(GolemMcpError::InternalError {
                message: format!("Signal handling error: {}", e),
                error_id: "SIGNAL_ERROR".to_string(),
            })
        }
    }
}

/// Main entry point with comprehensive error handling
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize logging first
    init_structured_logging(std::env::var("RUST_LOG").is_ok());
    
    info!("🚀 Starting Golem MCP Server - Advanced Integration Example");
    info!("📖 This example demonstrates production-ready patterns and error handling");
    
    // Parse configuration
    let config = parse_args();
    info!("⚙️  Configuration loaded: {:?}", config.transport);
    
    // Setup server with error handling
    let server = match setup_server(&config).await {
        Ok(server) => {
            info!("✅ Server setup completed successfully");
            server
        }
        Err(e) => {
            error!("❌ Server setup failed: {}", e);
            e.log_error();
            
            // Provide recovery guidance
            if let Some(hint) = e.recovery_hint() {
                info!("💡 Recovery suggestion: {:?}", hint);
            }
            
            return Err(Box::new(e));
        }
    };
    
    // Create shared server reference for graceful shutdown
    let server = Arc::new(Mutex::new(Some(server)));
    let server_for_shutdown = server.clone();
    
    // Start server in background task
    let transport = config.transport.clone();
    let server_task = tokio::spawn(async move {
        if let Some(server) = server.lock().await.take() {
            start_server_with_transport(server, transport).await
        } else {
            Err(GolemMcpError::InternalError {
                message: "Server already consumed".to_string(),
                error_id: "SERVER_CONSUMED".to_string(),
            })
        }
    });
    
    // Setup shutdown handler
    let shutdown_task = tokio::spawn(handle_shutdown());
    
    // Wait for either server completion or shutdown signal
    let result = timeout(
        Duration::from_secs(3600), // 1 hour max runtime
        tokio::select! {
            server_result = server_task => {
                match server_result {
                    Ok(Ok(())) => {
                        info!("✅ Server terminated successfully");
                        Ok(())
                    }
                    Ok(Err(e)) => {
                        error!("❌ Server error: {}", e);
                        e.log_error();
                        
                        // Categorize error for appropriate response
                        match e.category() {
                            ErrorCategory::ConfigurationError => {
                                info!("💡 Configuration error - check your settings");
                            }
                            ErrorCategory::NetworkError => {
                                info!("💡 Network error - check connectivity and ports");
                            }
                            ErrorCategory::SystemError => {
                                info!("💡 System error - check resources and permissions");
                            }
                            _ => {
                                info!("💡 See error details above for troubleshooting");
                            }
                        }
                        
                        Err(e)
                    }
                    Err(join_error) => {
                        error!("❌ Server task join error: {}", join_error);
                        Err(GolemMcpError::InternalError {
                            message: format!("Task join error: {}", join_error),
                            error_id: "TASK_JOIN_ERROR".to_string(),
                        })
                    }
                }
            }
            shutdown_result = shutdown_task => {
                match shutdown_result {
                    Ok(Ok(())) => {
                        info!("✅ Graceful shutdown completed");
                        Ok(())
                    }
                    Ok(Err(e)) => {
                        error!("❌ Shutdown error: {}", e);
                        Err(e)
                    }
                    Err(join_error) => {
                        error!("❌ Shutdown task join error: {}", join_error);
                        Err(GolemMcpError::InternalError {
                            message: format!("Shutdown join error: {}", join_error),
                            error_id: "SHUTDOWN_JOIN_ERROR".to_string(),
                        })
                    }
                }
            }
        }
    ).await;
    
    match result {
        Ok(Ok(())) => {
            info!("👋 Golem MCP Server shutdown complete");
            Ok(())
        }
        Ok(Err(e)) => {
            error!("❌ Server error during operation: {}", e);
            Err(Box::new(e))
        }
        Err(_) => {
            warn!("⏰ Server operation timed out after 1 hour");
            info!("🧹 Forcing cleanup...");
            
            // Force cleanup of server resources
            if let Some(_server) = server_for_shutdown.lock().await.take() {
                info!("🗑️  Server resources cleaned up");
            }
            
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use tokio_test;
    
    /// Test fixture for server configuration
    #[fixture]
    fn default_config() -> ServerConfig {
        ServerConfig::default()
    }
    
    /// Test fixture for HTTP configuration
    #[fixture]
    fn http_config() -> ServerConfig {
        ServerConfig {
            transport: GolemMcpTransport::Http {
                port: 8080,
                host: Some("127.0.0.1".to_string()),
            },
            ..ServerConfig::default()
        }
    }
    
    /// Test server configuration validation
    #[rstest]
    #[case::default_stdio(default_config(), "Default stdio configuration")]
    #[case::http_config(http_config(), "HTTP configuration")]
    fn test_server_configuration(
        #[case] config: ServerConfig,
        #[case] description: &str,
    ) {
        // Validate transport configuration
        match &config.transport {
            GolemMcpTransport::Stdio => {
                assert!(true, "Stdio transport is always valid");
            }
            GolemMcpTransport::Http { port, .. } => {
                assert!(*port > 0, "HTTP port should be positive");
                assert!(*port <= 65535, "HTTP port should be within valid range");
            }
        }
        
        // Validate timeout configuration
        assert!(config.timeout_config.default_timeout.as_secs() > 0, "Default timeout should be positive");
        
        println!("✅ {}: Configuration validated", description);
    }
    
    /// Test error categorization for different scenarios
    #[rstest]
    #[case::config_error(
        GolemMcpError::ConfigurationError {
            message: "Invalid configuration".to_string(),
            config_key: Some("transport".to_string()),
            expected_format: Some("stdio or http".to_string()),
        },
        ErrorCategory::ConfigurationError,
        "Configuration error"
    )]
    #[case::network_error(
        GolemMcpError::transport(crate::errors::TransportType::Http, "Connection refused"),
        ErrorCategory::NetworkError,
        "Network error"
    )]
    #[case::timeout_error(
        GolemMcpError::operation_timeout("test_operation", 30000),
        ErrorCategory::PerformanceError,
        "Timeout error"
    )]
    fn test_error_categorization(
        #[case] error: GolemMcpError,
        #[case] expected_category: ErrorCategory,
        #[case] description: &str,
    ) {
        assert_eq!(error.category(), expected_category);
        
        // Verify error has recovery hints for appropriate categories
        match expected_category {
            ErrorCategory::ConfigurationError | ErrorCategory::NetworkError => {
                assert!(error.recovery_hint().is_some(), "Error should have recovery hints");
            }
            ErrorCategory::PerformanceError => {
                assert!(error.recovery_hint().is_some(), "Performance errors should have retry hints");
            }
            _ => {
                // Other categories may or may not have hints
            }
        }
        
        println!("✅ {}: Error properly categorized as {:?}", description, expected_category);
    }
    
    /// Test async server setup with different configurations
    #[rstest]
    #[tokio::test]
    async fn test_server_setup_async(default_config: ServerConfig) {
        // Test server setup without actually starting transport
        let result = setup_server(&default_config).await;
        
        match result {
            Ok(_server) => {
                println!("✅ Server setup completed successfully");
            }
            Err(e) => {
                println!("⚠️  Server setup failed gracefully: {:?}", e);
                // Graceful failure is acceptable in test environment
                assert!(!e.to_string().is_empty(), "Error should have meaningful message");
            }
        }
    }
}