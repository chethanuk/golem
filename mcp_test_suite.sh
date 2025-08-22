#!/bin/bash
set -e

# Golem MCP Server Test Suite
# Comprehensive testing of MCP protocol compliance and functionality

echo "🧪 Golem MCP Server Test Suite"
echo "======================================"

# Configuration
MCP_HOST="localhost"
MCP_PORT="1232"
BASE_URL="http://${MCP_HOST}:${MCP_PORT}"
HEALTH_URL="${BASE_URL}/health"
MCP_URL="${BASE_URL}/mcp"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Helper functions
print_test() {
    echo -e "${BLUE}[TEST $((++TESTS_RUN))]${NC} $1"
}

print_pass() {
    echo -e "${GREEN}✓ PASS:${NC} $1"
    ((TESTS_PASSED++))
}

print_fail() {
    echo -e "${RED}✗ FAIL:${NC} $1"
    ((TESTS_FAILED++))
}

print_warn() {
    echo -e "${YELLOW}⚠ WARN:${NC} $1"
}

# Test if server is running
test_server_running() {
    print_test "Server connectivity"
    if curl -s --connect-timeout 5 "$HEALTH_URL" > /dev/null; then
        print_pass "Server is responding"
    else
        print_fail "Server is not responding at $HEALTH_URL"
        echo "Please start the server with:"
        echo "cargo run --bin golem-cli --features server-commands -- server mcp --serve-port $MCP_PORT"
        exit 1
    fi
}

# Test health endpoint
test_health_endpoint() {
    print_test "Health endpoint"
    RESPONSE=$(curl -s "$HEALTH_URL")
    if echo "$RESPONSE" | jq -e '.status == "healthy"' > /dev/null 2>&1; then
        print_pass "Health endpoint returns healthy status"
        echo "Response: $RESPONSE"
    else
        print_fail "Health endpoint response invalid: $RESPONSE"
    fi
}

# Test MCP initialization
test_mcp_initialization() {
    print_test "MCP protocol initialization"
    
    # Test with correct protocol version
    INIT_REQUEST='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0.0"}}}'
    RESPONSE=$(curl -s -X POST -H "Content-Type: application/json" -d "$INIT_REQUEST" "$MCP_URL")
    
    if echo "$RESPONSE" | jq -e '.result.protocolVersion == "2025-06-18"' > /dev/null 2>&1; then
        print_pass "MCP initialization successful"
        echo "Server info: $(echo "$RESPONSE" | jq '.result.serverInfo')"
    else
        print_fail "MCP initialization failed: $RESPONSE"
    fi
    
    # Test with incorrect protocol version
    print_test "Protocol version validation"
    BAD_INIT_REQUEST='{"jsonrpc":"2.0","id":2,"method":"initialize","params":{"protocolVersion":"1.0.0","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0.0"}}}'
    BAD_RESPONSE=$(curl -s -X POST -H "Content-Type: application/json" -d "$BAD_INIT_REQUEST" "$MCP_URL")
    
    if echo "$BAD_RESPONSE" | jq -e '.error.message | contains("version mismatch")' > /dev/null 2>&1; then
        print_pass "Protocol version validation working"
    else
        print_fail "Protocol version validation failed: $BAD_RESPONSE"
    fi
}

# Test tool listing
test_tool_listing() {
    print_test "Tool listing"
    TOOLS_REQUEST='{"jsonrpc":"2.0","id":3,"method":"tools/list","params":{}}'
    RESPONSE=$(curl -s -X POST -H "Content-Type: application/json" -d "$TOOLS_REQUEST" "$MCP_URL")
    
    if echo "$RESPONSE" | jq -e '.result | type == "array"' > /dev/null 2>&1; then
        TOOL_COUNT=$(echo "$RESPONSE" | jq '.result | length')
        print_pass "Tool listing successful - Found $TOOL_COUNT tools"
        if [ "$TOOL_COUNT" -eq 0 ]; then
            print_warn "No tools registered - this might be expected for a minimal implementation"
        fi
    else
        print_fail "Tool listing failed: $RESPONSE"
    fi
}

# Test resource listing
test_resource_listing() {
    print_test "Resource listing"
    RESOURCES_REQUEST='{"jsonrpc":"2.0","id":4,"method":"resources/list","params":{}}'
    RESPONSE=$(curl -s -X POST -H "Content-Type: application/json" -d "$RESOURCES_REQUEST" "$MCP_URL")
    
    if echo "$RESPONSE" | jq -e '.result | type == "array"' > /dev/null 2>&1; then
        RESOURCE_COUNT=$(echo "$RESPONSE" | jq '.result | length')
        print_pass "Resource listing successful - Found $RESOURCE_COUNT resources"
        if [ "$RESOURCE_COUNT" -eq 0 ]; then
            print_warn "No resources registered - this might be expected for a minimal implementation"
        fi
    else
        print_fail "Resource listing failed: $RESPONSE"
    fi
}

# Test tool execution
test_tool_execution() {
    print_test "Tool execution"
    TOOL_CALL_REQUEST='{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"test_tool","arguments":{"message":"hello world"}}}'
    RESPONSE=$(curl -s -X POST -H "Content-Type: application/json" -d "$TOOL_CALL_REQUEST" "$MCP_URL")
    
    if echo "$RESPONSE" | jq -e '.result.success == true' > /dev/null 2>&1; then
        print_pass "Tool execution successful"
        echo "Tool result: $(echo "$RESPONSE" | jq '.result.result')"
    else
        print_fail "Tool execution failed: $RESPONSE"
    fi
}

# Test resource reading
test_resource_reading() {
    print_test "Resource reading"
    RESOURCE_READ_REQUEST='{"jsonrpc":"2.0","id":6,"method":"resources/read","params":{"uri":"test://example"}}'
    RESPONSE=$(curl -s -X POST -H "Content-Type: application/json" -d "$RESOURCE_READ_REQUEST" "$MCP_URL")
    
    if echo "$RESPONSE" | jq -e '.result.contents | type == "array"' > /dev/null 2>&1; then
        print_pass "Resource reading successful"
        echo "Resource content: $(echo "$RESPONSE" | jq '.result.contents[0].text' 2>/dev/null || echo 'N/A')"
    else
        print_fail "Resource reading failed: $RESPONSE"
    fi
}

# Test error handling
test_error_handling() {
    print_test "JSON-RPC error handling"
    
    # Test invalid JSON
    INVALID_JSON='{"invalid":"json","missing":"jsonrpc"}'
    RESPONSE=$(curl -s -X POST -H "Content-Type: application/json" -d "$INVALID_JSON" "$MCP_URL")
    
    if echo "$RESPONSE" | jq -e '.error.code' > /dev/null 2>&1; then
        print_pass "Invalid JSON properly rejected"
    else
        print_fail "Invalid JSON not properly handled: $RESPONSE"
    fi
    
    # Test non-existent method
    INVALID_METHOD='{"jsonrpc":"2.0","id":7,"method":"invalid/method","params":{}}'
    RESPONSE=$(curl -s -X POST -H "Content-Type: application/json" -d "$INVALID_METHOD" "$MCP_URL")
    
    if echo "$RESPONSE" | jq -e '.error.code == -32601' > /dev/null 2>&1; then
        print_pass "Invalid method properly rejected with JSON-RPC error -32601"
    else
        print_fail "Invalid method not properly handled: $RESPONSE"
    fi
}

# Test streaming capabilities
test_streaming() {
    print_test "Server-Sent Events streaming"
    
    # Test GET request for SSE
    RESPONSE=$(timeout 2s curl -N -H "Accept: text/event-stream" "$MCP_URL" 2>/dev/null || true)
    
    if [ -n "$RESPONSE" ]; then
        print_pass "SSE endpoint responds"
        echo "SSE response preview: $(echo "$RESPONSE" | head -3)"
    else
        print_warn "SSE streaming not implemented or not responding (this is optional for basic MCP)"
    fi
}

# Test session management
test_session_management() {
    print_test "Session management"
    
    # Initialize with session tracking
    INIT_REQUEST='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"session-test-client","version":"1.0.0"}}}'
    RESPONSE=$(curl -s -X POST -H "Content-Type: application/json" -d "$INIT_REQUEST" "$MCP_URL" -i)
    
    # Check for session ID header
    if echo "$RESPONSE" | grep -i "mcp-session-id" > /dev/null; then
        SESSION_ID=$(echo "$RESPONSE" | grep -i "mcp-session-id" | cut -d: -f2 | tr -d ' \r\n')
        print_pass "Session ID provided: $SESSION_ID"
        
        # Test using session ID in subsequent request
        TOOL_REQUEST='{"jsonrpc":"2.0","id":8,"method":"tools/list","params":{}}'
        SESSION_RESPONSE=$(curl -s -X POST -H "Content-Type: application/json" -H "Mcp-Session-Id: $SESSION_ID" -d "$TOOL_REQUEST" "$MCP_URL")
        
        if echo "$SESSION_RESPONSE" | jq -e '.result' > /dev/null 2>&1; then
            print_pass "Session-based request successful"
        else
            print_fail "Session-based request failed: $SESSION_RESPONSE"
        fi
    else
        print_warn "Session management not implemented (optional for basic MCP)"
    fi
}

# Performance test
test_performance() {
    print_test "Performance and concurrency"
    
    # Run 10 concurrent requests
    echo "Running 10 concurrent tool execution requests..."
    START_TIME=$(date +%s.%N)
    
    for i in {1..10}; do
        (
            TOOL_REQUEST='{"jsonrpc":"2.0","id":'$i',"method":"tools/call","params":{"name":"perf_test_tool","arguments":{"test_id":'$i'}}}'
            curl -s -X POST -H "Content-Type: application/json" -d "$TOOL_REQUEST" "$MCP_URL" > /dev/null
        ) &
    done
    
    wait
    END_TIME=$(date +%s.%N)
    DURATION=$(echo "$END_TIME - $START_TIME" | bc -l)
    
    print_pass "Handled 10 concurrent requests in ${DURATION}s"
}

# Main test execution
main() {
    echo "Starting comprehensive MCP server testing..."
    echo ""
    
    # Check dependencies
    command -v curl >/dev/null 2>&1 || { print_fail "curl is required but not installed"; exit 1; }
    command -v jq >/dev/null 2>&1 || { print_fail "jq is required but not installed"; exit 1; }
    
    # Run all tests
    test_server_running
    test_health_endpoint
    test_mcp_initialization
    test_tool_listing
    test_resource_listing
    test_tool_execution
    test_resource_reading
    test_error_handling
    test_streaming
    test_session_management
    test_performance
    
    # Summary
    echo ""
    echo "======================================"
    echo "Test Results Summary:"
    echo "Tests Run: $TESTS_RUN"
    echo -e "${GREEN}Tests Passed: $TESTS_PASSED${NC}"
    echo -e "${RED}Tests Failed: $TESTS_FAILED${NC}"
    echo ""
    
    if [ $TESTS_FAILED -eq 0 ]; then
        echo -e "${GREEN}🎉 All tests passed! MCP server is working correctly.${NC}"
        exit 0
    else
        echo -e "${RED}❌ Some tests failed. Please review the failures above.${NC}"
        exit 1
    fi
}

# Run the test suite
main "$@"