//! Golem MCP Server - Production Binary
//!
//! A complete Model Context Protocol server that exposes all Golem CLI
//! functionality as MCP tools for AI agents like Claude Code.
//!
//! # Features
//!
//! - All 60+ Golem CLI commands exposed as MCP tools
//! - Multiple transport support (stdio, HTTP)
//! - Real-time progress streaming
//! - Resource discovery (golem.yaml, components, etc.)
//! - Production-ready error handling and logging

use clap::{Arg, Command, ValueEnum};
use golem_mcp_server::{serve_mcp_stdio, serve_mcp_http};
use std::net::{IpAddr, Ipv4Addr};
use tracing::{info, error};

/// Transport types supported by the MCP server
#[derive(Debug, Clone, ValueEnum)]
enum Transport {
    /// Stdio transport for Claude Desktop integration
    Stdio,
    /// HTTP transport for web applications and remote access
    Http,
}

/// Server configuration parsed from command line arguments
#[derive(Debug)]
struct ServerConfig {
    /// Transport type to use
    transport: Transport,
    /// Port for HTTP transport (ignored for stdio)
    port: u16,
    /// Host for HTTP transport (ignored for stdio)
    host: IpAddr,
    /// Verbose logging
    verbose: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            transport: Transport::Stdio,
            port: 1232,
            host: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            verbose: false,
        }
    }
}

/// Parse command line arguments
fn parse_args() -> ServerConfig {
    let matches = Command::new("golem-mcp-server")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Golem Model Context Protocol Server - Expose all Golem CLI tools to AI agents")
        .long_about(
            "The Golem MCP Server provides AI agents like Claude Code with access to all \
            Golem CLI functionality through the standardized Model Context Protocol. \
            \n\n\
            Features:\n\
            • All 60+ Golem CLI commands as MCP tools\n\
            • Component, worker, application, and project management\n\
            • Real-time progress streaming for long operations\n\
            • Resource discovery (golem.yaml, manifests, etc.)\n\
            • Production-ready error handling\n\
            \n\n\
            For Claude Desktop integration, use stdio transport (default).\n\
            For web applications or remote access, use HTTP transport."
        )
        .arg(
            Arg::new("transport")
                .short('t')
                .long("transport")
                .value_name("TYPE")
                .help("Transport type")
                .long_help(
                    "Transport mechanism for MCP communication:\n\
                    • stdio: Standard input/output (default) - for Claude Desktop\n\
                    • http: HTTP server - for web applications and remote access"
                )
                .value_parser(clap::value_parser!(Transport))
                .default_value("stdio")
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .help("HTTP server port (HTTP transport only)")
                .value_parser(clap::value_parser!(u16).range(1..=65535))
                .default_value("1232")
        )
        .arg(
            Arg::new("host")
                .short('H')
                .long("host")
                .value_name("HOST")
                .help("HTTP server host (HTTP transport only)")
                .value_parser(clap::value_parser!(IpAddr))
                .default_value("127.0.0.1")
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose logging")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    ServerConfig {
        transport: matches.get_one::<Transport>("transport").unwrap().clone(),
        port: *matches.get_one::<u16>("port").unwrap(),
        host: *matches.get_one::<IpAddr>("host").unwrap(),
        verbose: matches.get_flag("verbose"),
    }
}

/// Initialize structured logging
fn init_logging(verbose: bool) {
    let log_level = if verbose { "debug" } else { "info" };
    
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{},golem_mcp_server=debug", log_level).into())
        )
        .with_target(false)
        .with_thread_ids(verbose)
        .with_file(verbose)
        .with_line_number(verbose)
        .init();
}

/// Main entry point for the Golem MCP Server
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = parse_args();
    init_logging(config.verbose);
    
    info!("🚀 Starting Golem MCP Server v{}", env!("CARGO_PKG_VERSION"));
    info!("📋 Transport: {:?}", config.transport);
    
    match config.transport {
        Transport::Stdio => {
            info!("🎯 Starting stdio transport for Claude Desktop integration");
            info!("📄 All Golem CLI commands available as MCP tools");
            info!("🔧 Manifest resources will be discovered automatically");
            
            serve_mcp_stdio().await?;
        }
        Transport::Http => {
            let host_str = config.host.to_string();
            info!("🌐 Starting HTTP transport on {}:{}", host_str, config.port);
            info!("📋 Server endpoints:");
            info!("   • Main MCP: http://{}:{}/mcp", host_str, config.port);
            info!("   • Health: http://{}:{}/health", host_str, config.port);
            info!("📄 All Golem CLI commands available as MCP tools");
            info!("🔧 Manifest resources will be discovered automatically");
            
            serve_mcp_http(config.port, Some(host_str)).await?;
        }
    }
    
    info!("👋 Golem MCP Server shutdown complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert_eq!(config.port, 1232);
        assert!(matches!(config.transport, Transport::Stdio));
    }
    
    #[test]
    fn test_transport_parsing() {
        use clap::ValueEnum;
        let variants = Transport::value_variants();
        assert_eq!(variants.len(), 2);
        assert!(variants.contains(&Transport::Stdio));
        assert!(variants.contains(&Transport::Http));
    }
}