#!/usr/bin/env python3
"""REAL comparison: MCP vs CLI - same commands, same output"""

import json
import subprocess
import os
import time

def run_cli(cmd):
    """Run actual golem-cli command"""
    result = subprocess.run(
        f"cd /home/chethan/dev/golem && ./golem-cli {cmd}",
        shell=True,
        capture_output=True,
        text=True
    )
    return result.stdout.strip(), result.stderr.strip(), result.returncode

def run_mcp(proc, tool_name, args):
    """Run MCP tool"""
    req = {
        "jsonrpc": "2.0",
        "id": int(time.time() * 1000),
        "method": "tools/call",
        "params": {
            "name": tool_name,
            "arguments": args
        }
    }
    proc.stdin.write((json.dumps(req) + "\n").encode())
    proc.stdin.flush()
    resp = json.loads(proc.stdout.readline().decode())
    
    if 'error' in resp:
        return None, resp['error']['message']
    else:
        content = resp['result']['content'][0]['text']
        return content, None

print("=" * 80)
print("MCP vs CLI - REAL COMPARISON")
print("=" * 80)

# Start MCP server
proc = subprocess.Popen(
    ["/home/chethan/dev/golem/target/debug/golem-mcp-server"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    bufsize=0,
    env={"RUST_LOG": "error"}
)

# Initialize MCP
init = {"jsonrpc": "2.0", "id": 1, "method": "initialize", 
        "params": {"protocolVersion": "2024-11-05", "capabilities": {}, 
                  "clientInfo": {"name": "test", "version": "1.0"}}}
proc.stdin.write((json.dumps(init) + "\n").encode())
proc.stdin.flush()
proc.stdout.readline()
proc.stdin.write(json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}).encode() + b"\n")
proc.stdin.flush()
time.sleep(0.5)

# TEST 1: List profiles
print("\n1. LIST PROFILES")
print("-" * 40)

cli_out, cli_err, cli_code = run_cli("profile list")
print("CLI output:")
for line in cli_out.split('\n')[:10]:
    print(f"  {line}")

mcp_out, mcp_err = run_mcp(proc, "list_profiles", {})
print("\nMCP output:")
if mcp_out:
    try:
        profiles = json.loads(mcp_out)
        print(f"  Profiles: {profiles}")
    except:
        print(f"  {mcp_out[:200]}")
else:
    print(f"  Error: {mcp_err[:100]}")

# TEST 2: Current profile
print("\n2. CURRENT PROFILE") 
print("-" * 40)

cli_out, cli_err, cli_code = run_cli("profile get")
print("CLI output:")
for line in cli_out.split('\n')[:5]:
    print(f"  {line}")

# TEST 3: List components
print("\n3. LIST COMPONENTS")
print("-" * 40)

cli_out, cli_err, cli_code = run_cli("component list")
print("CLI output:")
if cli_code == 0:
    for line in cli_out.split('\n')[:10]:
        print(f"  {line}")
else:
    print(f"  Exit code: {cli_code}")
    print(f"  Error: {cli_err[:100]}")

mcp_out, mcp_err = run_mcp(proc, "list_components", {})
print("\nMCP output:")
if mcp_out:
    print(f"  {mcp_out[:200]}")
else:
    print(f"  Error: {mcp_err[:100]}")

# TEST 4: Test with auth token if available
print("\n4. WITH AUTH TOKEN")
print("-" * 40)

# Try with the admin token from local-run script
admin_token = "5c832d93-ff85-4a8f-9803-513950fdfdb1"
os.environ["GOLEM_API_TOKEN"] = admin_token

cli_out, cli_err, cli_code = run_cli("component list")
print("CLI with token:")
if cli_code == 0:
    print(f"  Success: {cli_out[:100]}")
else:
    print(f"  Failed: {cli_err[:100]}")

# TEST 5: Direct API test
print("\n5. DIRECT API TEST")
print("-" * 40)

api_result = subprocess.run(
    f'curl -s -H "Authorization: Bearer {admin_token}" http://localhost:9881/v1/components',
    shell=True,
    capture_output=True,
    text=True
)
print("Direct API call:")
print(f"  Response: {api_result.stdout[:200]}")

print("\n" + "=" * 80)
print("ANALYSIS:")
print("-" * 40)

# Check if CLI works
if "Selected profile" in cli_out:
    print("✓ CLI is running")
else:
    print("✗ CLI has issues")

# Check if MCP returns mock data
if mcp_out and "production" in str(mcp_out):
    print("✗ MCP is returning MOCK data - not connected to real API!")
elif mcp_err and "connection" in mcp_err.lower():
    print("✓ MCP is trying to connect to real API (same as CLI)")
else:
    print("? MCP status unclear")

# Check API
if "error" in api_result.stdout:
    print("✓ API is responding (may need auth)")
else:
    print("✗ API not responding properly")

proc.terminate()