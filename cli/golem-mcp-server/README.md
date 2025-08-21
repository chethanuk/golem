# Golem CLI MCP Server

A production-ready Model Context Protocol (MCP) server implementation that exposes Golem CLI functionality to AI agents like Claude Code.

## Overview

The Golem MCP Server enables AI agents to interact with Golem Cloud infrastructure through the standardized MCP protocol. It exposes all CLI commands as MCP tools and provides access to project manifests and configurations as MCP resources.

## Features

- ✅ **Complete MCP 2025-06-18 specification compliance**
- ✅ **HTTP transport** with RESTful endpoints
- ✅ **All Golem CLI commands** exposed as MCP tools
- ✅ **Manifest and resource discovery**
- ✅ **Real-time progress feedback**
- ✅ **Comprehensive error handling**
- ✅ **Production-ready security**

## Quick Start

### Prerequisites

- Rust 1.75+ with 2021 edition
- Golem CLI installed and accessible in PATH
- Port 1232 available (default, configurable)

### Build and Run

```bash
# Build with MCP server support
cargo build --features server-commands

# Start the MCP server
cargo run --features server-commands -- server mcp --serve-port 1232 --verbose

# Or using the built binary
./target/debug/golem-cli server mcp --serve-port 1232 --verbose
```

Expected output:
```
🚀 Golem CLI MCP Server running at http://localhost:1232
📋 Available at: http://localhost:1232/mcp
🏥 Health check: http://localhost:1232/mcp/health
⚡ Capabilities: http://localhost:1232/mcp/capabilities
🔧 All Golem CLI commands are available via MCP tools
📄 Manifest resources are discoverable via MCP resources
```

### Command Options

```bash
golem-cli server mcp [OPTIONS]

OPTIONS:
    -p, --serve-port <PORT>     Port to serve the MCP API on [default: 1232]
        --serve-host <HOST>     Address to bind the MCP server to [default: localhost]
        --verbose               Enable verbose logging for MCP server
    -h, --help                  Print help
```

## Testing Guide

### 1. Health Check Test

Verify the server is running:

```bash
# Test server health
curl -X GET http://localhost:1232/mcp/health

# Expected response:
{
  "status": "healthy",
  "timestamp": "2024-01-01T00:00:00Z"
}
```

### 2. Capabilities Test

Check server capabilities:

```bash
# Get server capabilities
curl -X GET http://localhost:1232/mcp/capabilities

# Expected response:
{
  "capabilities": {
    "tools": {
      "listChanged": true
    },
    "resources": {
      "subscribe": false,
      "listChanged": true
    },
    "logging": {}
  },
  "serverInfo": {
    "name": "Golem CLI MCP Server",
    "version": "0.0.0"
  }
}
```

### 3. MCP Protocol Test

Test the main MCP endpoint with JSON-RPC:

```bash
# Initialize MCP session
curl -X POST http://localhost:1232/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": "1",
    "method": "initialize",
    "params": {
      "protocolVersion": "2025-06-18",
      "capabilities": {
        "tools": {}
      },
      "clientInfo": {
        "name": "test-client",
        "version": "1.0.0"
      }
    }
  }'

# Expected response:
{
  "jsonrpc": "2.0",
  "id": "1",
  "result": {
    "protocolVersion": "2025-06-18",
    "capabilities": {
      "tools": {
        "listChanged": true
      },
      "resources": {
        "listChanged": true
      },
      "logging": {}
    },
    "serverInfo": {
      "name": "Golem CLI MCP Server",
      "version": "0.0.0"
    }
  }
}
```

### 4. Tools List Test

List available CLI tools:

```bash
# List all available tools
curl -X POST http://localhost:1232/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": "2",
    "method": "tools/list",
    "params": {}
  }'

# Expected response includes Golem CLI commands as MCP tools
{
  "jsonrpc": "2.0",
  "id": "2",
  "result": {
    "tools": [
      {
        "name": "app_new",
        "description": "Create a new Golem application",
        "inputSchema": {
          "type": "object",
          "properties": {
            "app-name": {
              "type": "string",
              "description": "Name of the new application"
            }
          },
          "required": ["app-name"]
        }
      }
      // ... more tools
    ]
  }
}
```

### 5. Resources Test

List available resources (manifests):

```bash
# List available resources
curl -X POST http://localhost:1232/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": "3",
    "method": "resources/list",
    "params": {}
  }'

# Expected response:
{
  "jsonrpc": "2.0",
  "id": "3",
  "result": {
    "resources": [
      {
        "uri": "file:///path/to/golem.yaml",
        "name": "Application Manifest",
        "description": "Golem application configuration",
        "mimeType": "application/yaml"
      }
      // ... more resources
    ]
  }
}
```

### 6. Tool Execution Test

Execute a Golem CLI command via MCP:

```bash
# Execute app list command
curl -X POST http://localhost:1232/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": "4",
    "method": "tools/call",
    "params": {
      "name": "app_list",
      "arguments": {}
    }
  }'

# Expected response:
{
  "jsonrpc": "2.0",
  "id": "4",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Listed applications output here..."
      }
    ],
    "isError": false
  }
}
```

## Integration with Claude Code

### Add to Claude Code Configuration

Add this to your Claude Code MCP settings:

```json
{
  "mcpServers": {
    "golem-cli": {
      "command": "golem-cli",
      "args": ["server", "mcp", "--serve-port", "1232", "--verbose"],
      "transport": "http",
      "url": "http://localhost:1232/mcp"
    }
  }
}
```

### Test with Claude Code

1. Start the MCP server:
   ```bash
   golem-cli server mcp --verbose
   ```

2. Open Claude Code and verify connection:
   - Check that "golem-cli" appears in available MCP servers
   - Try using commands like: "List my Golem applications"
   - Verify Claude Code can access manifest files

## Advanced Testing

### Performance Test

```bash
# Test concurrent requests
for i in {1..10}; do
  curl -X POST http://localhost:1232/mcp \
    -H "Content-Type: application/json" \
    -d "{\"jsonrpc\":\"2.0\",\"id\":\"$i\",\"method\":\"tools/list\",\"params\":{}}" &
done
wait
```

### Error Handling Test

```bash
# Test invalid method
curl -X POST http://localhost:1232/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": "err1",
    "method": "invalid/method",
    "params": {}
  }'

# Expected error response:
{
  "jsonrpc": "2.0",
  "id": "err1",
  "error": {
    "code": -32601,
    "message": "Method not found"
  }
}
```

### Timeout Test

```bash
# Test long-running command (if available)
curl -X POST http://localhost:1232/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": "timeout1",
    "method": "tools/call",
    "params": {
      "name": "component_build",
      "arguments": {
        "component-name": "example"
      }
    }
  }' \
  --max-time 30
```

## Development Testing

### Run Unit Tests

```bash
# Test all MCP crates
cargo test -p golem-mcp-types
cargo test -p golem-mcp-server  
cargo test -p golem-mcp-tools
cargo test -p golem-mcp-resources

# Run with verbose output
cargo test --verbose
```

### Run Integration Tests

```bash
# Full integration test suite
cargo test --features server-commands --test integration

# Test specific scenarios
cargo test --features server-commands test_mcp_protocol_compliance
cargo test --features server-commands test_cli_tool_execution
```

### Linting and Code Quality

```bash
# Check code formatting
cargo fmt --check

# Run Clippy with pedantic lints
cargo clippy --features server-commands -- -D warnings

# Check for security issues
cargo audit
```

## Troubleshooting

### Common Issues

1. **Port Already in Use**
   ```bash
   # Check what's using the port
   lsof -i :1232
   # Use different port
   golem-cli server mcp --serve-port 1233
   ```

2. **Golem CLI Not Found**
   ```bash
   # Ensure golem-cli is in PATH
   which golem-cli
   # Or specify full path in configuration
   ```

3. **Permission Denied**
   ```bash
   # Check file permissions for manifest access
   ls -la golem.yaml
   ```

### Debug Mode

Enable detailed logging:

```bash
# Maximum verbosity
RUST_LOG=debug golem-cli server mcp --verbose

# MCP-specific logs only
RUST_LOG=golem_mcp=debug golem-cli server mcp --verbose
```

### Logs Location

Server logs are written to stderr. Redirect for analysis:

```bash
# Save logs to file
golem-cli server mcp --verbose 2> mcp-server.log

# Monitor logs in real-time
tail -f mcp-server.log
```

## API Reference

### HTTP Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/mcp` | POST | Main MCP JSON-RPC endpoint |
| `/mcp/health` | GET | Server health check |
| `/mcp/capabilities` | GET | Server capabilities |

### MCP Methods

| Method | Description | Parameters |
|--------|-------------|------------|
| `initialize` | Initialize MCP session | `protocolVersion`, `capabilities`, `clientInfo` |
| `tools/list` | List available tools | None |
| `tools/call` | Execute a tool | `name`, `arguments` |
| `resources/list` | List available resources | None |
| `resources/read` | Read resource content | `uri` |

## Contributing

1. Follow Rust 2024+ patterns and Staff Engineer best practices
2. Maintain zero-tolerance for anti-patterns
3. Add comprehensive tests for new functionality
4. Update documentation for API changes
5. Ensure Clippy pedantic compliance

## License

Licensed under the Golem Source License v1.0. See LICENSE file for details.