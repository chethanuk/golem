#!/usr/bin/env python3
"""THE TRUTH: Which MCP tools ACTUALLY call the real API vs return FAKE data"""

import json
import subprocess
import time

proc = subprocess.Popen(
    ["/home/chethan/dev/golem/target/debug/golem-mcp-server"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    bufsize=0,
    env={"RUST_LOG": "error"}
)

def call(tool_name, args={}):
    req = {
        "jsonrpc": "2.0",
        "id": int(time.time() * 1000),
        "method": "tools/call",
        "params": {"name": tool_name, "arguments": args}
    }
    proc.stdin.write((json.dumps(req) + "\n").encode())
    proc.stdin.flush()
    return json.loads(proc.stdout.readline().decode())

# Initialize
init = {"jsonrpc": "2.0", "id": 1, "method": "initialize", 
        "params": {"protocolVersion": "2024-11-05", "capabilities": {}, 
                  "clientInfo": {"name": "truth", "version": "1.0"}}}
proc.stdin.write((json.dumps(init) + "\n").encode())
proc.stdin.flush()
proc.stdout.readline()
proc.stdin.write(json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}).encode() + b"\n")
proc.stdin.flush()
time.sleep(0.5)

print("=" * 80)
print("MCP TOOLS: REAL API vs FAKE DATA")
print("=" * 80)

# Tools to test
test_cases = [
    ("list_profiles", {}, "FAKE - hardcoded at line 1325-1338"),
    ("get_account", {}, "FAKE - hardcoded at line 1477-1496"),
    ("list_plugins", {}, "CHECK"),
    ("list_tokens", {}, "CHECK"),
    ("list_projects", {}, "CHECK"),
    ("list_components", {}, "REAL API - calls api_client.list_components()"),
    ("get_component", {"component_id": "test"}, "REAL API - calls api_client.list_components()"),
    ("create_worker", {"component_id": "test", "name": "w1", "env": [], "args": []}, "REAL API - calls api_client.create_worker()"),
]

for tool, args, expected in test_cases:
    resp = call(tool, args)
    
    print(f"\n{tool}:")
    print(f"  Expected: {expected}")
    
    if 'error' in resp:
        error = resp['error']['message']
        if 'connection' in error.lower() or 'unavailable' in error.lower():
            print(f"  Result: REAL API (got connection error)")
        elif 'unauthorized' in error.lower():
            print(f"  Result: REAL API (got auth error)")
        elif 'invalid' in error.lower():
            print(f"  Result: REAL API (validation error)")
        else:
            print(f"  Result: UNKNOWN ({error[:50]})")
    else:
        content = resp['result']['content'][0]['text']
        # Check for telltale signs of fake data
        if '"email": "user@example.com"' in content:
            print(f"  Result: FAKE DATA (example.com email)")
        elif '"Professional"' in content and '"Golem User"' in content:
            print(f"  Result: FAKE DATA (hardcoded account)")
        elif 'production' in content and 'staging' in content:
            print(f"  Result: LIKELY FAKE (too convenient)")
        else:
            try:
                data = json.loads(content)
                if isinstance(data, list) and len(data) == 2:
                    if any('local' in str(item) and 'cloud' in str(item) for item in [data]):
                        print(f"  Result: FAKE DATA (hardcoded profiles)")
                    else:
                        print(f"  Result: Could be real")
                else:
                    print(f"  Result: DATA returned (verify manually)")
                    print(f"    {str(data)[:100]}...")
            except:
                print(f"  Result: {content[:100]}")

print("\n" + "=" * 80)
print("CONCLUSION:")
print("-" * 80)
print("Most 'working' tools return FAKE/MOCK data!")
print("Only tools that call api_client methods use real API.")
print("The MCP server needs REAL implementations, not mocks!")

proc.terminate()