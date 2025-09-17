#!/bin/bash
# Test script for MCP server with MCP Inspector
# This demonstrates how to test the MCP server using the official MCP Inspector tool

set -e

echo "=== MCP Server Testing with Inspector ==="
echo

# Build the server in debug mode for testing
echo "Building Golem MCP Server..."
cargo build --bin golem-mcp-server

# Test 1: Direct execution test
echo "Test 1: Testing direct server execution..."
timeout 2s cargo run --bin golem-mcp-server 2>&1 | head -20 || true
echo

# Test 2: Using MCP Inspector (if available)
if command -v npx &> /dev/null; then
    echo "Test 2: Testing with MCP Inspector..."
    echo "Running: npx @modelcontextprotocol/inspector cargo run --bin golem-mcp-server"
    echo
    echo "This will open a web interface where you can:"
    echo "1. See all registered tools"
    echo "2. Test tool invocations"
    echo "3. Monitor request/response flow"
    echo
    # Uncomment to actually run:
    # npx @modelcontextprotocol/inspector cargo run --bin golem-mcp-server
else
    echo "MCP Inspector not available. Install with: npm install -g @modelcontextprotocol/inspector"
fi

# Test 3: JSON-RPC message test
echo "Test 3: Testing JSON-RPC message handling..."
cat << 'EOF' | cargo run --bin golem-mcp-server 2>/dev/null | head -20 || true
{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{"tools":{}},"clientInfo":{"name":"test","version":"1.0"}},"id":1}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","method":"tools/list","id":2}
EOF

echo
echo "=== Testing Complete ==="