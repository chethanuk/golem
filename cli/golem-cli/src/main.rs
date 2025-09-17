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
    #[cfg(feature = "server-commands")]
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
                ServerSubcommand::Run { args } => {
                    // Import necessary components from golem crate for server run
                    use golem::launch::{LaunchArgs, launch_golem_services};
                    use std::path::PathBuf;
                    
                    // Get the data directory
                    let data_dir = match &args.data_dir {
                        Some(data_dir) => data_dir.to_path_buf(),
                        None => dirs::data_local_dir()
                            .ok_or_else(|| anyhow::anyhow!("Failed to get data local dir"))?
                            .join("golem"),
                    };
                    
                    // Clean the data directory if requested
                    if args.clean && tokio::fs::metadata(&data_dir).await.is_ok() {
                        tokio::fs::remove_dir_all(&data_dir)
                            .await
                            .map_err(|err| anyhow::anyhow!("Failed cleaning data dir ({}): {}", data_dir.display(), err))?;
                    }
                    
                    println!("Starting Golem server with data directory: {}", data_dir.display());
                    
                    // Launch the Golem services
                    let mut join_set = launch_golem_services(&LaunchArgs {
                        router_addr: args.router_addr().to_string(),
                        router_port: args.router_port(),
                        custom_request_port: args.custom_request_port(),
                        data_dir,
                    }).await?;
                    
                    // Wait for all services to complete
                    while let Some(res) = join_set.join_next().await {
                        res??;
                    }
                    
                    Ok(())
                }
                ServerSubcommand::Clean => {
                    let data_dir = dirs::data_local_dir()
                        .ok_or_else(|| anyhow::anyhow!("Failed to get data local dir"))?
                        .join("golem");
                    
                    tokio::fs::remove_dir_all(&data_dir)
                        .await
                        .map_err(|err| anyhow::anyhow!("Failed cleaning data dir ({}): {}", data_dir.display(), err))
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
