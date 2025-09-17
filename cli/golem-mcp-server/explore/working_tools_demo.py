#!/usr/bin/env python3
"""Demo of WORKING MCP tools - showing actual output"""

import json
import subprocess
import time

ADMIN_TOKEN = "5c832d93-ff85-4a8f-9803-513950fdfdb1"

proc = subprocess.Popen(
    ["/home/chethan/dev/golem/target/debug/golem-mcp-server"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    bufsize=0,
    env={"GOLEM_API_TOKEN": ADMIN_TOKEN, "RUST_LOG": "error"}
)

def call(method, params=None, id=None):
    req = {"jsonrpc": "2.0", "method": method}
    if params:
        req["params"] = params
    if id is not None:
        req["id"] = id
    proc.stdin.write((json.dumps(req) + "\n").encode())
    proc.stdin.flush()
    if id is not None:
        return json.loads(proc.stdout.readline().decode())

# Initialize
call("initialize", {
    "protocolVersion": "2024-11-05",
    "capabilities": {},
    "clientInfo": {"name": "demo", "version": "1.0"}
}, 1)
call("notifications/initialized")
time.sleep(0.5)

print("=" * 70)
print("MCP TOOLS THAT WORK WITHOUT GOLEM API")
print("=" * 70)

# 1. List profiles
print("\n1. LIST PROFILES:")
resp = call("tools/call", {"name": "list_profiles", "arguments": {}}, 2)
if 'result' in resp:
    content = json.loads(resp['result']['content'][0]['text'])
    print(f"   Found {len(content)} profiles:")
    for profile in content:
        print(f"   - {profile['name']} (active: {profile.get('active', False)})")

# 2. Get account info
print("\n2. GET ACCOUNT:")
resp = call("tools/call", {"name": "get_account", "arguments": {}}, 3)
if 'result' in resp:
    content = json.loads(resp['result']['content'][0]['text'])
    print(f"   Account: {content}")

# 3. List plugins
print("\n3. LIST PLUGINS:")
resp = call("tools/call", {"name": "list_plugins", "arguments": {}}, 4)
if 'result' in resp:
    content = json.loads(resp['result']['content'][0]['text'])
    print(f"   Plugins: {content}")

# 4. List tokens
print("\n4. LIST TOKENS:")
resp = call("tools/call", {"name": "list_tokens", "arguments": {}}, 5)
if 'result' in resp:
    content = json.loads(resp['result']['content'][0]['text'])
    print(f"   Tokens: {content}")

# 5. Switch profile
print("\n5. SWITCH PROFILE (to local):")
resp = call("tools/call", {"name": "switch_profile", "arguments": {"profile_name": "local"}}, 6)
if 'result' in resp:
    content = resp['result']['content'][0]['text']
    print(f"   Result: {content}")


# 6. List projects
print("\n6. LIST PROJECTS:")
resp = call("tools/call", {"name": "list_projects", "arguments": {}}, 7)
if 'result' in resp:
    content = json.loads(resp['result']['content'][0]['text'])
    print(f"   Projects: {content}")


# Switch to cloud
print("\n5. SWITCH PROFILE (to cloud):")
resp = call("tools/call", {"name": "switch_profile", "arguments": {"profile_name": "cloud"}}, 6)
if 'result' in resp:
    content = resp['result']['content'][0]['text']
    print(f"   Result: {content}")



# 6. List projects
print("\n6. LIST PROJECTS:")
resp = call("tools/call", {"name": "list_projects", "arguments": {}}, 7)
if 'result' in resp:
    content = json.loads(resp['result']['content'][0]['text'])
    print(f"   Projects: {content}")

proc.terminate()
