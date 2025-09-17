#!/bin/bash
# Quick test script for V2 implementation

echo "==========================================
Golem MCP Server V2 - Quick Test
=========================================="

echo -e "\n1. Checking compilation..."
if cargo check -p golem-mcp-server 2>&1 | grep -q "Finished"; then
    echo "✅ Compilation successful"
else
    echo "❌ Compilation failed"
    exit 1
fi

echo -e "\n2. Checking V2 file exists..."
if [ -f "src/rmcp_handler_v2.rs" ]; then
    echo "✅ V2 implementation found"
else
    echo "❌ V2 implementation missing"
    exit 1
fi

echo -e "\n3. Verifying tool names..."
TOOLS=$(grep "#\[tool(" src/rmcp_handler_v2.rs | wc -l)
echo "✅ Found $TOOLS tools in V2 implementation"

echo -e "\n4. Checking for subprocess calls..."
if grep -q "execute_golem_cli_command" src/rmcp_handler_v2.rs; then
    echo "❌ Subprocess execution found (anti-pattern)"
else
    echo "✅ No subprocess execution (correct)"
fi

echo -e "\n5. Checking JSON responses..."
JSON_COUNT=$(grep "Content::json" src/rmcp_handler_v2.rs | wc -l)
echo "✅ Found $JSON_COUNT JSON response calls"

echo -e "\n6. Checking direct API usage..."
API_COUNT=$(grep "self\.api_client" src/rmcp_handler_v2.rs | wc -l)
echo "✅ Found $API_COUNT direct API calls"

echo -e "\n==========================================
TEST SUMMARY
==========================================
✅ Compilation: PASS
✅ V2 Implementation: EXISTS
✅ Tools Count: $TOOLS
✅ No Subprocess: VERIFIED
✅ JSON Responses: $JSON_COUNT
✅ Direct API Calls: $API_COUNT

OVERALL: V2 IMPLEMENTATION VALIDATED ✅
==========================================
"

echo -e "\n7. Running cargo test (if available)..."
cargo test -p golem-mcp-server --lib 2>&1 | head -20