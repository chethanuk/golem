//! Server configuration and startup parameters
//!
//! Provides comprehensive configuration management for the MCP server,
//! supporting environment variables, configuration files, and CLI arguments.

use figment::{Figment, providers::{Format, Toml, Env, Serialized}};
use golem_mcp_types::{
    error::{McpError, McpResult},
    transport::HttpTransportConfig,
    protocol::{McpServerInfo, McpImplementation, LogLevel},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use which;

/// MCP server configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Server information
    pub server_info: McpServerInfo,
    /// HTTP transport configuration
    pub http: HttpTransportConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Security configuration
    pub security: SecurityConfig,
    /// Golem CLI integration settings
    pub golem: GolemConfig,
    /// Additional server settings
    #[serde(flatten)]
    pub settings: HashMap<String, serde_json::Value>,
}

/// Logging configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: LogLevel,
    /// Enable JSON formatted logs
    pub json: bool,
    /// Log file path (optional)
    pub file: Option<PathBuf>,
    /// Enable request/response logging
    pub trace_requests: bool,
    /// Maximum log file size in bytes
    pub max_file_size: Option<u64>,
    /// Number of log files to retain
    pub max_files: Option<u32>,
}

/// Security configuration  
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable authentication
    pub enable_auth: bool,
    /// Authentication tokens (if auth enabled)
    pub auth_tokens: Vec<String>,
    /// CORS allowed origins
    pub cors_origins: Vec<String>,
    /// Maximum request size in bytes
    pub max_request_size: usize,
    /// Request rate limiting (requests per minute)
    pub rate_limit: Option<u32>,
    /// Enable TLS
    pub enable_tls: bool,
    /// TLS certificate file path
    pub tls_cert_file: Option<PathBuf>,
    /// TLS private key file path
    pub tls_key_file: Option<PathBuf>,
}

/// Golem CLI integration configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GolemConfig {
    /// Path to golem CLI binary
    pub cli_path: Option<PathBuf>,
    /// Default golem CLI profile to use
    pub default_profile: Option<String>,
    /// Command timeout in seconds
    pub command_timeout_secs: u64,
    /// Maximum concurrent CLI commands
    pub max_concurrent_commands: u32,
    /// Working directory for CLI commands
    pub working_directory: Option<PathBuf>,
    /// Environment variables for CLI commands
    pub env_vars: HashMap<String, String>,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            server_info: McpServerInfo {
                name: "golem-mcp-server".to_owned(),
                version: env!("CARGO_PKG_VERSION").to_owned(),
                implementation: Some(McpImplementation {
                    name: "golem-mcp-server".to_owned(),
                    version: env!("CARGO_PKG_VERSION").to_owned(),
                    metadata: {
                        let mut map = HashMap::new();
                        map.insert("repository".to_owned(), serde_json::Value::String("https://github.com/golemcloud/golem".to_owned()));
                        map.insert("homepage".to_owned(), serde_json::Value::String("https://golem.cloud".to_owned()));
                        map
                    },
                }),
            },
            http: HttpTransportConfig::default(),
            logging: LoggingConfig::default(),
            security: SecurityConfig::default(),
            golem: GolemConfig::default(),
            settings: HashMap::new(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            json: false,
            file: None,
            trace_requests: false,
            max_file_size: Some(100 * 1024 * 1024), // 100MB
            max_files: Some(5),
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_auth: false,
            auth_tokens: Vec::new(),
            cors_origins: vec!["*".to_owned()],
            max_request_size: 1024 * 1024, // 1MB
            rate_limit: None,
            enable_tls: false,
            tls_cert_file: None,
            tls_key_file: None,
        }
    }
}

impl Default for GolemConfig {
    fn default() -> Self {
        Self {
            cli_path: None, // Will auto-detect golem binary
            default_profile: None,
            command_timeout_secs: 300, // 5 minutes
            max_concurrent_commands: 10,
            working_directory: None,
            env_vars: HashMap::new(),
        }
    }
}

impl McpServerConfig {
    /// Create configuration from multiple sources
    ///
    /// Priority order (highest to lowest):
    /// 1. Environment variables (prefixed with GOLEM_MCP_)
    /// 2. Configuration file (if provided)
    /// 3. Default values
    ///
    /// # Errors
    ///
    /// Returns `McpError::Configuration` if configuration cannot be loaded or is invalid.
    pub fn from_sources(config_file: Option<&PathBuf>) -> McpResult<Self> {
        let mut figment = Figment::new();

        // Start with defaults
        figment = figment.merge(Serialized::defaults(Self::default()));

        // Add config file if provided
        if let Some(path) = config_file {
            if path.exists() {
                figment = figment.merge(Toml::file(path));
            } else {
                return Err(McpError::configuration(
                    "config_file".to_owned(),
                    format!("Configuration file not found: {}", path.display()),
                ));
            }
        }

        // Add environment variables
        figment = figment.merge(Env::prefixed("GOLEM_MCP_").split("_"));

        // Extract configuration
        figment.extract().map_err(|e| {
            McpError::configuration(
                "figment_extract".to_owned(),
                format!("Failed to extract configuration: {e}"),
            )
        })
    }

    /// Validate the configuration
    ///
    /// # Errors
    ///
    /// Returns `McpError::Configuration` if the configuration is invalid.
    pub fn validate(&self) -> McpResult<()> {
        // Validate HTTP configuration
        self.http.validate()?;

        // Validate security configuration
        if self.security.enable_auth && self.security.auth_tokens.is_empty() {
            return Err(McpError::configuration(
                "security.auth_tokens".to_owned(),
                "Authentication is enabled but no auth tokens are configured".to_owned(),
            ));
        }

        if self.security.enable_tls {
            if self.security.tls_cert_file.is_none() {
                return Err(McpError::configuration(
                    "security.tls_cert_file".to_owned(),
                    "TLS is enabled but no certificate file is configured".to_owned(),
                ));
            }
            if self.security.tls_key_file.is_none() {
                return Err(McpError::configuration(
                    "security.tls_key_file".to_owned(),
                    "TLS is enabled but no private key file is configured".to_owned(),
                ));
            }
        }

        // Validate Golem configuration
        if self.golem.command_timeout_secs == 0 {
            return Err(McpError::configuration(
                "golem.command_timeout_secs".to_owned(),
                "Command timeout must be greater than 0".to_owned(),
            ));
        }

        if self.golem.max_concurrent_commands == 0 {
            return Err(McpError::configuration(
                "golem.max_concurrent_commands".to_owned(),
                "Max concurrent commands must be greater than 0".to_owned(),
            ));
        }

        // Validate logging configuration
        if let Some(max_size) = self.logging.max_file_size {
            if max_size == 0 {
                return Err(McpError::configuration(
                    "logging.max_file_size".to_owned(),
                    "Max file size must be greater than 0".to_owned(),
                ));
            }
        }

        if let Some(max_files) = self.logging.max_files {
            if max_files == 0 {
                return Err(McpError::configuration(
                    "logging.max_files".to_owned(),
                    "Max files must be greater than 0".to_owned(),
                ));
            }
        }

        Ok(())
    }

    /// Get the effective Golem CLI binary path
    ///
    /// If not explicitly configured, attempts to find 'golem' in PATH.
    ///
    /// # Errors
    ///
    /// Returns `McpError::Configuration` if the Golem CLI binary cannot be found.
    pub fn golem_cli_path(&self) -> McpResult<PathBuf> {
        if let Some(ref path) = self.golem.cli_path {
            if path.exists() {
                Ok(path.clone())
            } else {
                Err(McpError::configuration(
                    "golem.cli_path".to_owned(),
                    format!("Configured Golem CLI path does not exist: {}", path.display()),
                ))
            }
        } else {
            // Try to find golem in PATH
            which::which("golem").map_err(|e| {
                McpError::configuration(
                    "golem.cli_path".to_owned(),
                    format!("Golem CLI not found in PATH: {e}"),
                )
            })
        }
    }

    /// Get the server bind address
    #[must_use]
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.http.host, self.http.port)
    }

    /// Check if authentication is required
    #[must_use]
    pub const fn requires_auth(&self) -> bool {
        self.security.enable_auth
    }

    /// Get configured auth tokens
    #[must_use]
    pub fn auth_tokens(&self) -> &[String] {
        &self.security.auth_tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_default_config_validation() {
        let config = McpServerConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_with_auth_but_no_tokens() {
        let mut config = McpServerConfig::default();
        config.security.enable_auth = true;
        config.security.auth_tokens.clear();
        
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_from_toml_file() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        writeln!(temp_file, r#"
[server_info]
name = "test-server"
version = "1.0.0"

[http]
host = "127.0.0.1"
port = 8080

[logging]
level = "Debug"
json = true
        "#).expect("Failed to write config");

        let config = McpServerConfig::from_sources(Some(temp_file.path()))
            .expect("Should load config");
        
        assert_eq!(config.server_info.name, "test-server");
        assert_eq!(config.http.port, 8080);
        assert_eq!(config.logging.level, LogLevel::Debug);
        assert!(config.logging.json);
    }

    #[test]
    fn test_golem_cli_path_detection() {
        let config = McpServerConfig::default();
        // This test will pass if golem is in PATH, otherwise it should fail gracefully
        let result = config.golem_cli_path();
        // We can't make strong assertions here since it depends on the system
        // but we can verify the method doesn't panic
        match result {
            Ok(path) => assert!(!path.as_os_str().is_empty()),
            Err(e) => assert!(e.to_string().contains("not found")),
        }
    }

    #[test]
    fn test_bind_address_formatting() {
        let mut config = McpServerConfig::default();
        config.http.host = "0.0.0.0".to_owned();
        config.http.port = 3000;
        
        assert_eq!(config.bind_address(), "0.0.0.0:3000");
    }
}