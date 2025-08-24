#!/bin/bash
set -e

echo "🧪 Testing Golem MCP Server"
BASE_URL="http://127.0.0.1:8080"

# Test 1: Health check
echo "1. Testing health endpoint..."
curl -s "$BASE_URL/health" | jq '.'

# Test 2: Initialize MCP session
echo -e "\n2. Initializing MCP session..."
curl -s -X POST "$BASE_URL/mcp" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": "1",
    "method": "initialize",
    "params": {
      "protocolVersion": "2025-06-18",
      "capabilities": {"tools": {}},
      "clientInfo": {"name": "test-client", "version": "1.0.0"}
    }
  }' | jq '.'

# Test 3: List all available tools
echo -e "\n3. Listing all available Golem tools..."
curl -s -X POST "$BASE_URL/mcp" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": "2",
    "method": "tools/list"
  }' | jq '.result.tools[] | {name: .name, description: .description}'

# Test 4: List component templates (real Golem CLI call)
echo -e "\n4. Testing real Golem CLI integration - list templates..."
curl -s -X POST "$BASE_URL/mcp" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": "3",
    "method": "tools/call",
    "params": {
      "name": "golem_project_management",
      "arguments": {"operation": "list_templates"}
    }
  }' | jq '.'

echo -e "\n✅ MCP Server testing complete!"