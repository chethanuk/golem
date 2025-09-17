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

//! Demonstration of "Bypassing Clap" Pattern in Golem MCP Server
//! 
//! This example shows how the MCP server uses direct API clients
//! instead of parsing CLI commands through Clap.

use golem_mcp_server::client_manager::{ClientManager, ClientMode};
use golem_mcp_server::GolemMcpServerV2;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("=== Demonstrating Bypassing Clap Pattern ===");
    
    // Traditional approach (NOT what we do):
    // 1. Parse CLI arguments with Clap
    // 2. Execute CLI command as subprocess
    // 3. Parse text output
    // 4. Convert to JSON
    info!("Traditional CLI approach would:");
    info!("  - Parse: golem component list --format json");
    info!("  - Execute subprocess");
    info!("  - Parse stdout text");
    info!("  - Convert to structured data");
    
    // Our approach: Bypassing Clap
    info!("\nBypassing Clap approach (what we actually do):");
    
    // 1. Direct API client initialization
    let mut client_manager = ClientManager::new();
    
    // 2. Initialize for OSS mode (localhost)
    let oss_url = "http://localhost:9881";
    client_manager.init_oss(oss_url)?;
    info!("  ✓ Initialized OSS clients directly at {}", oss_url);
    
    // 3. Direct API call without CLI parsing
    let oss_clients = client_manager.oss()?;
    info!("  ✓ Got component client without Clap parsing");
    
    // Example: List components directly
    // In real usage:
    // let components = oss_clients.component.get_components(
    //     None,  // component_name_filter
    //     None,  // cursor  
    //     None,  // limit
    //     None,  // precise
    // ).await?;
    info!("  ✓ Can call component.get_components() directly");
    
    // 4. Cloud mode example
    info!("\nCloud mode example:");
    let cloud_url = "https://release.api.golem.cloud";
    let token = Some("your-api-token".to_string());
    client_manager.init_cloud(cloud_url, token)?;
    info!("  ✓ Initialized cloud clients with authentication");
    
    let cloud_clients = client_manager.cloud()?;
    info!("  ✓ Got full cloud client suite:");
    info!("    - account, token, project clients");
    info!("    - api_definition, api_deployment clients");
    info!("    - component, worker clients");
    info!("    - All without parsing CLI commands!");
    
    // Benefits demonstrated
    info!("\n=== Benefits of Bypassing Clap ===");
    info!("1. Performance: No subprocess overhead");
    info!("2. Type safety: Strongly typed API clients");
    info!("3. Error handling: Structured errors, not text parsing");
    info!("4. Flexibility: Switch between cloud/OSS dynamically");
    info!("5. Direct responses: JSON objects, not CLI text");
    
    // MCP Server integration
    info!("\n=== MCP Server Integration ===");
    let server = GolemMcpServerV2::new()?;
    info!("  ✓ Created MCP server with {} tools", server.tool_count());
    info!("  ✓ Each tool uses direct API calls");
    info!("  ✓ No CLI subprocess invocations");
    
    // Example tool check
    if server.has_tool("list_components") {
        info!("  ✓ list_components tool available");
        info!("    - Uses api_client or client_manager directly");
        info!("    - Returns JSON, not CLI output");
    }
    
    info!("\n=== Bypassing Clap Pattern Complete ===");
    info!("This is how we achieve efficient MCP server implementation!");
    
    Ok(())
}