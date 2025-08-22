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

use crate::hooks::NoHooks;
use golem_cli::command_handler::CommandHandler;
use std::process::ExitCode;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerCommandError {
    #[error("MCP server failed to start: {0}")]
    McpStartFailed(#[from] golem_mcp_server::McpError),
    #[error("Server configuration error: {message}")]
    ConfigError { message: String },
}

#[cfg(feature = "server-commands")]
mod hooks {
    use golem_cli::command::server::ServerSubcommand;
    use golem_cli::command_handler::CommandHandlerHooks;
    use golem_cli::context::Context;

    use clap_verbosity_flag::Verbosity;
    use std::sync::Arc;

    pub struct NoHooks {}

    impl CommandHandlerHooks for NoHooks {
        #[cfg(feature = "server-commands")]
        async fn handler_server_commands(
            &self,
            _ctx: Arc<Context>,
            subcommand: ServerSubcommand,
        ) -> anyhow::Result<()> {
            match subcommand {
                ServerSubcommand::Mcp { serve_port, serve_host } => {
                    use golem_mcp_server::{McpServer, McpServerConfig};
                    
                    // Create server configuration
                    let mut config = McpServerConfig::default();
                    config.http.host = serve_host.clone();
                    config.http.port = serve_port;
                    // Use default INFO level logging
                    use golem_mcp_server::LogLevel;
                    config.logging.level = LogLevel::Info;
                    
                    // Create and start the MCP server
                    let server = McpServer::new(config).await?;
                    
                    println!("🚀 Golem CLI MCP Server running at http://{}:{}", serve_host, serve_port);
                    println!("📋 Available at: http://{}:{}/mcp", serve_host, serve_port);
                    println!("🏥 Health check: http://{}:{}/mcp/health", serve_host, serve_port);
                    println!("⚡ Capabilities: http://{}:{}/mcp/capabilities", serve_host, serve_port);
                    
                    println!("🔧 All Golem CLI commands are available via MCP tools");
                    println!("📄 Manifest resources are discoverable via MCP resources");
                    
                    // Start the server (this will run indefinitely)
                    server.serve().await?;
                    
                    Ok(())
                }
                _ => {
                    // Handle other server commands (Run, Clean) - delegate to default behavior
                    eprintln!("Server command not implemented in MCP context");
                    Ok(())
                }
            }
        }

        #[cfg(feature = "server-commands")]
        async fn run_server() -> anyhow::Result<()> {
            unimplemented!()
        }

        #[cfg(feature = "server-commands")]
        fn override_verbosity(verbosity: Verbosity) -> Verbosity {
            verbosity
        }

        #[cfg(feature = "server-commands")]
        fn override_pretty_mode() -> bool {
            false
        }
    }
}

#[cfg(not(feature = "server-commands"))]
mod hooks {
    use golem_cli::command_handler::CommandHandlerHooks;

    pub struct NoHooks {}

    impl CommandHandlerHooks for NoHooks {}
}

fn main() -> ExitCode {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to build tokio runtime for golem-cli main")
        .block_on(CommandHandler::handle_args(
            std::env::args_os(),
            Arc::from(NoHooks {}),
        ))
}
