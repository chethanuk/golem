// Test MCP protocol without requiring Golem server
// This tests basic MCP functionality: initialization, tool listing, etc.

use serde_json::json;
use std::process::{Command, Stdio};
use std::io::{Write, BufRead, BufReader};

#[test]
fn test_mcp_server_starts() {
    // Build the server
    let output = Command::new("cargo")
        .args(["build", "--bin", "golem-mcp-server"])
        .output()
        .expect("Failed to build");
    
    assert!(output.status.success(), "Build failed: {}", 
        String::from_utf8_lossy(&output.stderr));
}

#[test] 
fn test_mcp_initialization() {
    // Build first
    Command::new("cargo")
        .args(["build", "--bin", "golem-mcp-server", "--quiet"])
        .output()
        .expect("Failed to build");
    
    // Start MCP server
    let mut child = Command::new("target/debug/golem-mcp-server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start MCP server");
    
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    
    // Send initialize request
    let init_request = json!({
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {
            "protocolVersion": "0.1.0",
            "capabilities": {},
            "clientInfo": {
                "name": "test",
                "version": "1.0"
            }
        },
        "id": 1
    });
    
    writeln!(stdin, "{}", init_request).expect("Failed to write");
    stdin.flush().expect("Failed to flush");
    
    // Read response
    let mut response = String::new();
    stdout.read_line(&mut response).expect("Failed to read response");
    
    // Parse and validate
    let json: serde_json::Value = serde_json::from_str(&response)
        .expect("Invalid JSON response");
    
    assert_eq!(json["jsonrpc"], "2.0");
    assert_eq!(json["id"], 1);
    assert!(json["result"].is_object(), "Should have result");
    assert!(json["result"]["serverInfo"].is_object(), "Should have serverInfo");
    
    // Clean up
    let _ = child.kill();
}

#[test]
fn test_mcp_list_tools() {
    // Build first
    Command::new("cargo")
        .args(["build", "--bin", "golem-mcp-server", "--quiet"])
        .output()
        .expect("Failed to build");
    
    // Start MCP server
    let mut child = Command::new("target/debug/golem-mcp-server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start MCP server");
    
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    
    // Initialize first
    let init_request = json!({
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {
            "protocolVersion": "0.1.0",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        },
        "id": 1
    });
    
    writeln!(stdin, "{}", init_request).expect("Failed to write");
    stdin.flush().expect("Failed to flush");
    
    let mut response = String::new();
    stdout.read_line(&mut response).expect("Failed to read");
    response.clear();
    
    // Now list tools
    let list_request = json!({
        "jsonrpc": "2.0",
        "method": "tools/list",
        "params": {},
        "id": 2
    });
    
    writeln!(stdin, "{}", list_request).expect("Failed to write");
    stdin.flush().expect("Failed to flush");
    
    stdout.read_line(&mut response).expect("Failed to read");
    
    let json: serde_json::Value = serde_json::from_str(&response)
        .expect("Invalid JSON response");
    
    assert_eq!(json["jsonrpc"], "2.0");
    assert_eq!(json["id"], 2);
    
    let tools = json["result"]["tools"].as_array()
        .expect("Should have tools array");
    
    // Verify we have the expected number of tools (46 as per registration tests)
    assert_eq!(tools.len(), 46, "Should have 46 tools registered");
    
    // Check for some expected tools
    let tool_names: Vec<String> = tools.iter()
        .filter_map(|t| t["name"].as_str())
        .map(|s| s.to_string())
        .collect();
    
    assert!(tool_names.contains(&"component_list".to_string()));
    assert!(tool_names.contains(&"component_deploy".to_string()));
    assert!(tool_names.contains(&"worker_create".to_string()));
    assert!(tool_names.contains(&"worker_invoke".to_string()));
    
    // Clean up
    let _ = child.kill();
}

#[test]
fn test_mcp_call_tool_without_golem() {
    // Build first
    Command::new("cargo")
        .args(["build", "--bin", "golem-mcp-server", "--quiet"])
        .output()
        .expect("Failed to build");
    
    // Start MCP server
    let mut child = Command::new("target/debug/golem-mcp-server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start MCP server");
    
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    
    // Initialize
    let init_request = json!({
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {
            "protocolVersion": "0.1.0",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        },
        "id": 1
    });
    
    writeln!(stdin, "{}", init_request).unwrap();
    stdin.flush().unwrap();
    
    let mut response = String::new();
    stdout.read_line(&mut response).unwrap();
    response.clear();
    
    // Try to call component_list (will fail without Golem but should respond)
    let call_request = json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "component_list",
            "arguments": {}
        },
        "id": 2
    });
    
    writeln!(stdin, "{}", call_request).unwrap();
    stdin.flush().unwrap();
    
    stdout.read_line(&mut response).unwrap();
    
    let json: serde_json::Value = serde_json::from_str(&response)
        .expect("Should get valid JSON response");
    
    // Should get an error since Golem isn't running
    assert_eq!(json["jsonrpc"], "2.0");
    assert_eq!(json["id"], 2);
    
    // The tool should fail gracefully when Golem isn't available
    // Either error or result with failure message
    assert!(json["error"].is_object() || json["result"].is_object(),
        "Should have error or result");
    
    // Clean up
    let _ = child.kill();
}