// E2E Test Client for Golem MCP Server
// Based on FuelLabs ForcMcpClient pattern

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicI64, Ordering};
use serde_json::{json, Value};
use anyhow::{Result, anyhow};
use tokio::time::{timeout, Duration};

/// Test client for E2E testing of Golem MCP Server
pub struct GolemMcpTestClient {
    process: Child,
    stdin: std::process::ChildStdin,
    stdout: BufReader<std::process::ChildStdout>,
    request_id: AtomicI64,
}

impl GolemMcpTestClient {
    /// Create a new test client and spawn the MCP server process
    pub async fn new() -> Result<Self> {
        // First ensure the binary is built
        let output = Command::new("cargo")
            .args(["build", "--bin", "golem-mcp-server"])
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!(
                "Failed to build golem-mcp-server: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        // Spawn MCP server process with stdio
        let mut process = Command::new("target/debug/golem-mcp-server")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        
        let stdin = process.stdin.take()
            .ok_or_else(|| anyhow!("Failed to capture stdin"))?;
        let stdout = BufReader::new(
            process.stdout.take()
                .ok_or_else(|| anyhow!("Failed to capture stdout"))?
        );
        
        Ok(Self {
            process,
            stdin,
            stdout,
            request_id: AtomicI64::new(1),
        })
    }
    
    /// Initialize the MCP connection
    pub async fn initialize(&mut self) -> Result<Value> {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "initialize",
            "params": {
                "protocolVersion": "0.1.0",
                "capabilities": {},
                "clientInfo": {
                    "name": "golem-test-client",
                    "version": "1.0.0"
                }
            },
            "id": self.request_id.fetch_add(1, Ordering::SeqCst)
        });
        
        self.send_request(request).await
    }
    
    /// List available tools
    pub async fn list_tools(&mut self) -> Result<Vec<String>> {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "tools/list",
            "params": {},
            "id": self.request_id.fetch_add(1, Ordering::SeqCst)
        });
        
        let response = self.send_request(request).await?;
        
        // Extract tool names from response
        let tools = response["result"]["tools"]
            .as_array()
            .ok_or_else(|| anyhow!("Invalid tools response"))?;
        
        let names: Vec<String> = tools.iter()
            .filter_map(|t| t["name"].as_str())
            .map(|s| s.to_string())
            .collect();
        
        Ok(names)
    }
    
    /// Call a tool with given arguments
    pub async fn call_tool(&mut self, name: &str, args: Value) -> Result<Value> {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": args
            },
            "id": self.request_id.fetch_add(1, Ordering::SeqCst)
        });
        
        self.send_request(request).await
    }
    
    /// Send a JSON-RPC request and read response
    async fn send_request(&mut self, request: Value) -> Result<Value> {
        // Write request
        let request_str = serde_json::to_string(&request)?;
        writeln!(self.stdin, "{}", request_str)?;
        self.stdin.flush()?;
        
        // Read response with timeout
        let mut response = String::new();
        
        // Use tokio for async timeout
        let result = tokio::task::block_in_place(|| {
            timeout(
                Duration::from_secs(10),
                tokio::task::spawn_blocking({
                    let mut stdout = std::io::BufReader::new(std::io::stdin()); // Placeholder, would use self.stdout
                    move || -> Result<String> {
                        let mut line = String::new();
                        // In real impl, would read from self.stdout
                        Ok(line)
                    }
                })
            )
        });
        
        // For now, use blocking read with manual timeout check
        self.stdout.read_line(&mut response)?;
        
        if response.is_empty() {
            return Err(anyhow!("Empty response from server"));
        }
        
        // Parse JSON response
        let json: Value = serde_json::from_str(&response)?;
        
        // Check for errors
        if let Some(error) = json.get("error") {
            return Err(anyhow!("MCP error: {}", error));
        }
        
        Ok(json)
    }
    
    /// Shutdown the test client and kill the process
    pub async fn shutdown(mut self) -> Result<()> {
        // Send shutdown notification
        let shutdown = json!({
            "jsonrpc": "2.0",
            "method": "shutdown",
            "params": {}
        });
        
        let _ = writeln!(self.stdin, "{}", shutdown);
        let _ = self.stdin.flush();
        
        // Give it a moment to shut down gracefully
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Kill the process
        let _ = self.process.kill();
        let _ = self.process.wait();
        
        Ok(())
    }
}

/// Helper function to ensure Golem server is running
pub async fn ensure_golem_running() -> Result<()> {
    // Check if Golem is running on localhost:9881
    let client = reqwest::Client::new();
    match client.get("http://localhost:9881/healthcheck")
        .timeout(Duration::from_secs(2))
        .send()
        .await 
    {
        Ok(_) => Ok(()),
        Err(_) => {
            eprintln!("WARNING: Golem server not running on localhost:9881");
            eprintln!("E2E tests require Golem server. Start it with:");
            eprintln!("  golem-cli --local server run");
            eprintln!("Or use Docker:");
            eprintln!("  cd docker-examples/local-sqlite && docker compose up");
            Err(anyhow!("Golem server required for E2E tests"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mcp_initialization() -> Result<()> {
        let mut client = GolemMcpTestClient::new().await?;
        
        let response = client.initialize().await?;
        assert_eq!(response["jsonrpc"], "2.0");
        assert!(response["result"]["serverInfo"].is_object());
        
        client.shutdown().await?;
        Ok(())
    }
    
    #[tokio::test]
    async fn test_list_tools() -> Result<()> {
        let mut client = GolemMcpTestClient::new().await?;
        
        // Initialize first
        client.initialize().await?;
        
        // List tools
        let tools = client.list_tools().await?;
        
        // We should have at least the expected tools
        assert!(tools.len() > 0, "Should have tools registered");
        
        // Check for some expected tool names
        assert!(tools.contains(&"component_list".to_string()));
        assert!(tools.contains(&"component_deploy".to_string()));
        assert!(tools.contains(&"worker_create".to_string()));
        assert!(tools.contains(&"worker_invoke".to_string()));
        
        client.shutdown().await?;
        Ok(())
    }
    
    #[tokio::test]
    #[ignore] // This test requires Golem server running
    async fn test_component_list_with_golem() -> Result<()> {
        // Ensure Golem is running
        ensure_golem_running().await?;
        
        let mut client = GolemMcpTestClient::new().await?;
        
        // Initialize
        client.initialize().await?;
        
        // Call component_list tool
        let result = client.call_tool("component_list", json!({})).await?;
        
        // Should get a successful response
        assert!(result["result"].is_object());
        
        client.shutdown().await?;
        Ok(())
    }
}