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

//! Golem MCP Server - V2 Implementation
//!
//! This is the main entry point for the Golem MCP Server using V2 implementation.
//! V2 provides:
//! - Industry-standard MCP patterns
//! - Direct API calls (no subprocess)
//! - JSON responses
//! - Simple tool names

use golem_mcp_server::GolemMcpServerV2;
use rmcp::{ServiceExt, transport::stdio};
use tracing::info;
use tracing_subscriber::{filter::LevelFilter, EnvFilter, FmtSubscriber};

#[tokio::main]
async fn main() {
    // Initialize tracing
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy()
        .add_directive("golem_mcp_server=debug".parse().unwrap());
    
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(filter)
        .with_target(false)
        .with_file(false)
        .with_line_number(false)
        .with_writer(std::io::stderr)  // Write logs to stderr, not stdout
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");
    
    info!("Starting Golem MCP Server V2");
    
    // Use V2 implementation with direct API calls
    let server = GolemMcpServerV2::new().expect("Failed to create V2 server");
    
    // Run the server using stdio transport - directly use Service trait's serve method
    info!("Serving MCP over stdio transport");
    
    let service = server.serve(stdio()).await.inspect_err(|e| {
        eprintln!("Server initialization error: {:?}", e);
    });
    
    match service {
        Ok(running_service) => {
            info!("Server initialized successfully, waiting for requests...");
            if let Err(e) = running_service.waiting().await {
                eprintln!("Server runtime error: {}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to start server: {:?}", e);
            std::process::exit(1);
        }
    }
}
