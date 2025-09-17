#!/usr/bin/env python3
"""
Test script to verify Golem MCP Server V2 tool functionality
Acts as a proper staff engineer verification
"""

import json
import subprocess
import sys
from typing import Dict, Any

def send_request(request: Dict[str, Any]) -> Dict[str, Any]:
    """Send a JSON-RPC request to the MCP server via stdio"""
    proc = subprocess.Popen(
        ['cargo', 'run', '--bin', 'golem-mcp-server'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True
    )
    
    # Send request
    proc.stdin.write(json.dumps(request) + '\n')
    proc.stdin.flush()
    
    # Read response
    response_line = proc.stdout.readline()
    proc.terminate()
    
    if response_line:
        return json.loads(response_line)
    return {}

def test_initialize():
    """Test server initialization"""
    request = {
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {
            "protocolVersion": "0.1.0",
            "capabilities": {"tools": {}}
        },
        "id": 1
    }
    
    response = send_request(request)
    
    if 'result' in response:
        server_info = response['result'].get('serverInfo', {})
        print(f"✅ Initialize: Server name = {server_info.get('name')}")
        print(f"   Version: {server_info.get('version')}")
        capabilities = response['result'].get('capabilities', {})
        print(f"   Has tools: {capabilities.get('tools') is not None}")
        return True
    else:
        print(f"❌ Initialize failed: {response}")
        return False

def test_list_tools():
    """Test listing available tools"""
    # First initialize
    init_request = {
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {
            "protocolVersion": "0.1.0",
            "capabilities": {"tools": {}}
        },
        "id": 1
    }
    
    # Then list tools
    list_request = {
        "jsonrpc": "2.0",
        "method": "tools/list",
        "params": {},
        "id": 2
    }
    
    # Send both requests
    proc = subprocess.Popen(
        ['cargo', 'run', '--bin', 'golem-mcp-server'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True
    )
    
    # Initialize first
    proc.stdin.write(json.dumps(init_request) + '\n')
    proc.stdin.flush()
    init_response = proc.stdout.readline()
    
    # Then list tools
    proc.stdin.write(json.dumps(list_request) + '\n')
    proc.stdin.flush()
    tools_response = proc.stdout.readline()
    proc.terminate()
    
    if tools_response:
        response = json.loads(tools_response)
        if 'result' in response:
            tools = response['result'].get('tools', [])
            print(f"✅ List tools: Found {len(tools)} tools")
            if tools:
                print(f"   Sample tools: {[t['name'] for t in tools[:5]]}")
            return len(tools) == 46
        else:
            print(f"❌ List tools failed: {response}")
    return False

def test_call_tool():
    """Test calling a specific tool"""
    # Use a three-step process: initialize, list, then call
    proc = subprocess.Popen(
        ['cargo', 'run', '--bin', 'golem-mcp-server'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True
    )
    
    # Initialize
    init_request = {
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {
            "protocolVersion": "0.1.0",
            "capabilities": {"tools": {}}
        },
        "id": 1
    }
    proc.stdin.write(json.dumps(init_request) + '\n')
    proc.stdin.flush()
    init_response = proc.stdout.readline()
    
    # Call list_components tool
    call_request = {
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "list_components",
            "arguments": {}
        },
        "id": 3
    }
    proc.stdin.write(json.dumps(call_request) + '\n')
    proc.stdin.flush()
    call_response = proc.stdout.readline()
    proc.terminate()
    
    if call_response:
        response = json.loads(call_response)
        if 'result' in response:
            content = response['result'].get('content', [])
            print(f"✅ Call tool (list_components): Got response with {len(content)} content items")
            return True
        else:
            error = response.get('error', {})
            print(f"⚠️  Call tool: {error.get('message', 'Unknown error')}")
    return False

def main():
    """Run all tests"""
    print("=" * 60)
    print("Golem MCP Server V2 - Tool Functionality Verification")
    print("=" * 60)
    
    tests = [
        ("Initialize", test_initialize),
        ("List Tools", test_list_tools),
        ("Call Tool", test_call_tool),
    ]
    
    passed = 0
    failed = 0
    
    for name, test_func in tests:
        print(f"\nTesting: {name}")
        print("-" * 40)
        try:
            if test_func():
                passed += 1
            else:
                failed += 1
        except Exception as e:
            print(f"❌ {name} exception: {e}")
            failed += 1
    
    print("\n" + "=" * 60)
    print(f"Results: {passed} passed, {failed} failed")
    
    if failed == 0:
        print("✅ All tool functionality tests passed!")
        return 0
    else:
        print("❌ Some tests failed - tools may not be fully functional")
        return 1

if __name__ == "__main__":
    sys.exit(main())