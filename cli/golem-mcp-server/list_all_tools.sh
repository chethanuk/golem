#!/bin/bash
BASE_URL="http://127.0.0.1:1234"

echo "🔍 Discovering all Golem MCP tools..."

# Initialize session first
curl -s -X POST "$BASE_URL/mcp" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":"1","method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{"tools":{}},"clientInfo":{"name":"tool-lister","version":"1.0.0"}}}' > /dev/null

# List all tools with detailed info
echo "📊 Available Golem Tools (60+ commands):"
echo "=================================="

curl -s -X POST "$BASE_URL/mcp" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":"2","method":"tools/list"}' | \
  jq -r '.result.tools[] | "🔧 \(.name)\n   📝 \(.description)\n   📋 Args: \(.inputSchema.properties // {} | keys | join(", "))\n"'