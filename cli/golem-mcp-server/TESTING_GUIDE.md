# MCP Server Testing Guide

## Overview

This guide provides comprehensive testing patterns for MCP (Model Context Protocol) servers using the rmcp Rust SDK v0.6.0, with specific focus on STDIO transport and tool registration.

## Key Testing Areas

### 1. Tool Registration Testing

Tools must be properly registered through the `#[tool_router]` macro and exposed via the `ServerHandler` implementation.

**Common Issues:**
- Using `#[tool_handler]` macro incorrectly
- Not delegating to `tool_router` in `ServerHandler` implementation
- Missing tool metadata (descriptions, schemas)

**Solution Pattern:**
```rust
// Correct implementation
impl ServerHandler for YourServer {
    async fn list_tools(&self, _req: ListToolsRequest, _ctx: RequestContext<RoleServer>) 
        -> Result<ListToolsResult, McpError> {
        // Get tools from the router
        let tools = self.tool_router.map
            .values()
            .map(|route| route.attr.clone())
            .collect();
        
        Ok(ListToolsResult { tools, _meta: Default::default() })
    }
    
    async fn call_tool(&self, req: CallToolRequest, ctx: RequestContext<RoleServer>) 
        -> Result<CallToolResult, McpError> {
        // Delegate to tool_router
        if let Some(route) = self.tool_router.map.get(&req.params.name) {
            let tool_ctx = ToolCallContext {
                params: req.params,
                context: ctx,
                state: self,
                meta: Default::default(),
            };
            (route.call)(tool_ctx).await
        } else {
            Err(McpError::method_not_found("Tool not found", None))
        }
    }
}
```

### 2. STDIO Transport Testing

STDIO transport uses standard input/output for communication. Testing requires simulating these streams.

**Test Patterns:**

```rust
// Create mock STDIO using DuplexStream
let (client_stream, server_stream) = tokio::io::duplex(4096);

// Start server with mock transport
let server = YourServer::new();
tokio::spawn(async move {
    rmcp::serve_server(server, server_stream).await
});

// Send JSON-RPC messages
let request = json!({
    "jsonrpc": "2.0",
    "method": "tools/list",
    "id": 1
});
```

### 3. E2E Testing Flow

Complete MCP session testing should follow this flow:

1. **Initialize** - Send initialize request with capabilities
2. **Initialized** - Send initialized notification
3. **List Tools** - Request available tools
4. **Call Tools** - Invoke specific tools
5. **Verify Results** - Check responses for correctness

### 4. Integration Test Structure

```
tests/
├── test_tool_registration.rs  # Tool registration and discovery
├── test_e2e_stdio.rs          # E2E STDIO transport tests
├── test_tool_invocation.rs    # Individual tool testing
└── test_with_mcp_inspector.sh # MCP Inspector integration
```

## Testing Tools

### MCP Inspector

The official MCP Inspector tool provides visual testing:

```bash
npx @modelcontextprotocol/inspector cargo run --bin your-mcp-server
```

Features:
- Visual tool listing
- Interactive tool invocation
- Request/response monitoring
- Performance metrics

### Cargo Test

Run all integration tests:

```bash
# Run all tests
cargo test

# Run specific test module
cargo test test_tool_registration

# Run with logging
RUST_LOG=debug cargo test -- --nocapture
```

## Common Testing Scenarios

### 1. Verify Tool Registration

```rust
#[tokio::test]
async fn test_tools_are_registered() {
    let server = YourServer::new();
    assert!(!server.tool_router.map.is_empty());
    assert!(server.tool_router.map.contains_key("your_tool_name"));
}
```

### 2. Test Tool Parameters

```rust
#[tokio::test]
async fn test_tool_with_parameters() {
    let params = YourToolParams {
        field1: "value".to_string(),
        field2: 42,
    };
    
    let result = server.your_tool(Parameters(params)).await;
    assert!(result.is_ok());
}
```

### 3. Test Error Handling

```rust
#[tokio::test]
async fn test_invalid_tool_call() {
    let request = CallToolRequest {
        params: ToolCall {
            name: "non_existent".to_string(),
            parameters: None,
        },
        _meta: Default::default(),
    };
    
    let result = server.call_tool(request, context).await;
    assert!(result.is_err());
}
```

## Performance Testing

### Concurrent Tool Calls

```rust
#[tokio::test]
async fn test_concurrent_calls() {
    let server = Arc::new(YourServer::new());
    let mut handles = vec![];
    
    for _ in 0..100 {
        let server = Arc::clone(&server);
        handles.push(tokio::spawn(async move {
            server.list_tools(Default::default(), Default::default()).await
        }));
    }
    
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }
}
```

### Large Message Handling

Test STDIO transport with large payloads to ensure proper buffering:

```rust
#[tokio::test]  
async fn test_large_payload() {
    let large_data = vec!["item"; 10000];
    let message = json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "process_data",
            "parameters": { "data": large_data }
        },
        "id": 1
    });
    // Test handling...
}
```

## Debugging Tips

1. **Enable Tracing**: Use `RUST_LOG=debug` for detailed logs
2. **Check Tool Router**: Verify `tool_router.map` contains expected tools
3. **Validate JSON Schema**: Ensure tool parameters have valid schemas
4. **Test Incrementally**: Start with initialization, then tools/list, then individual tools
5. **Use MCP Inspector**: Visual debugging with the official inspector tool

## CI/CD Integration

```yaml
# GitHub Actions example
- name: Run MCP Tests
  run: |
    cargo test --test test_tool_registration
    cargo test --test test_e2e_stdio
    ./tests/test_with_mcp_inspector.sh
```

## References

- [rmcp Documentation](https://docs.rs/rmcp/latest/rmcp/)
- [MCP Specification](https://spec.modelcontextprotocol.io)
- [MCP Inspector](https://github.com/modelcontextprotocol/inspector)
- [Example Implementations](https://github.com/modelcontextprotocol/rust-sdk/tree/main/examples)