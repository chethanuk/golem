#!/usr/bin/env python3
"""Test MCP server with ADMIN TOKEN - compare with CLI"""

import json
import subprocess
import time
import os

ADMIN_TOKEN = "5c832d93-ff85-4a8f-9803-513950fdfdb1"

def run_cli_command(cmd):
    """Run golem-cli command and return output"""
    result = subprocess.run(
        f"cd /home/chethan/dev/golem && ./golem-cli {cmd}",
        shell=True,
        capture_output=True,
        text=True,
        env={**os.environ, "GOLEM_API_TOKEN": ADMIN_TOKEN}
    )
    return result.stdout, result.stderr

def run_mcp_tool(proc, tool_name, arguments):
    """Call MCP tool and return response"""
    req = {
        "jsonrpc": "2.0",
        "id": int(time.time() * 1000),
        "method": "tools/call",
        "params": {
            "name": tool_name,
            "arguments": arguments
        }
    }
    proc.stdin.write((json.dumps(req) + "\n").encode())
    proc.stdin.flush()
    response = json.loads(proc.stdout.readline().decode())
    return response

print("=" * 70)
print("MCP vs CLI COMPARISON TEST WITH ADMIN TOKEN")
print("=" * 70)

# Start MCP server with admin token
proc = subprocess.Popen(
    ["/home/chethan/dev/golem/target/debug/golem-mcp-server"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    bufsize=0,
    env={
        **os.environ,
        "GOLEM_API_URL": "http://localhost:9881",
        "GOLEM_API_TOKEN": ADMIN_TOKEN,
        "RUST_LOG": "error"
    }
)

# Initialize MCP
init_req = {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": {"name": "auth-test", "version": "1.0"}
    }
}
proc.stdin.write((json.dumps(init_req) + "\n").encode())
proc.stdin.flush()
init_resp = json.loads(proc.stdout.readline().decode())
print(f"\n✓ MCP Server initialized: {init_resp['result']['serverInfo']['name']}")

# Send initialized
proc.stdin.write(json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}).encode() + b"\n")
proc.stdin.flush()
time.sleep(0.5)

print("\n" + "=" * 70)
print("TESTING COMPONENT OPERATIONS")
print("=" * 70)

# Test 1: List Components
print("\n1. LIST COMPONENTS:")
print("-" * 50)

cli_out, cli_err = run_cli_command("component list")
print("CLI Output:")
if "401" in cli_err or "authorization" in cli_err.lower():
    print("  ERROR: CLI also getting auth error!")
    print(f"  {cli_err[:200]}")
else:
    print(f"  {cli_out[:300]}")

mcp_resp = run_mcp_tool(proc, "list_components", {})
print("\nMCP Output:")
if 'error' in mcp_resp:
    print(f"  ERROR: {mcp_resp['error']['message'][:200]}")
else:
    content = mcp_resp['result']['content'][0]['text']
    print(f"  {content[:300]}")

# Test 2: Create a test component
print("\n2. DEPLOY COMPONENT:")
print("-" * 50)

wasm_path = "/home/chethan/dev/golem/wasm-ast/wasm/shopping-cart.wasm"
if os.path.exists(wasm_path):
    # CLI deploy
    cli_out, cli_err = run_cli_command(f"component deploy {wasm_path}")
    print("CLI Output:")
    print(f"  {cli_out[:200] if cli_out else cli_err[:200]}")
    
    # MCP deploy
    mcp_resp = run_mcp_tool(proc, "deploy_component", {
        "name": "test-shopping-cart",
        "wasm_path": wasm_path
    })
    print("\nMCP Output:")
    if 'error' in mcp_resp:
        print(f"  ERROR: {mcp_resp['error']['message'][:200]}")
    else:
        content = mcp_resp['result']['content'][0]['text']
        print(f"  SUCCESS: {content[:200]}")

print("\n" + "=" * 70)
print("TESTING WORKER OPERATIONS")
print("=" * 70)

# Test 3: List Workers
print("\n3. LIST WORKERS:")
print("-" * 50)

# First we need a component ID
# For now, use a dummy UUID to show both fail the same way
test_component_id = "12345678-1234-1234-1234-123456789abc"

cli_out, cli_err = run_cli_command(f"worker list {test_component_id}")
print("CLI Output:")
print(f"  {cli_out[:200] if cli_out else cli_err[:200]}")

mcp_resp = run_mcp_tool(proc, "list_workers", {
    "component_id": test_component_id
})
print("\nMCP Output:")
if 'error' in mcp_resp:
    print(f"  ERROR: {mcp_resp['error']['message'][:200]}")
else:
    content = mcp_resp['result']['content'][0]['text']
    print(f"  {content[:200]}")

print("\n" + "=" * 70)
print("RESULTS:")
print("=" * 70)
print("With admin token, both MCP and CLI should work the same way.")
print("If both fail with auth, the token might not be valid for this setup.")
print("If MCP fails but CLI works, there's an issue with MCP auth handling.")

proc.terminate()