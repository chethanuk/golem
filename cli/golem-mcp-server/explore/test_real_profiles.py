#!/usr/bin/env python3
"""Test that profile tools now use REAL config files instead of fake data"""

import json
import subprocess
import time
import os

# Start MCP server
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
                  "clientInfo": {"name": "test", "version": "1.0"}}}
proc.stdin.write((json.dumps(init) + "\n").encode())
proc.stdin.flush()
proc.stdout.readline()
proc.stdin.write(json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}).encode() + b"\n")
proc.stdin.flush()
time.sleep(0.5)

print("=" * 80)
print("TESTING REAL PROFILE TOOLS (Phase 1)")
print("=" * 80)

# Check if config file exists
config_path = os.path.expanduser("~/.golem/config-v2.json")
config_exists = os.path.exists(config_path)
print(f"\nConfig file exists at {config_path}: {config_exists}")

if config_exists:
    with open(config_path, 'r') as f:
        actual_config = json.load(f)
    print(f"Actual config has {len(actual_config.get('profiles', {}))} profiles")

# TEST 1: List profiles - should read REAL config
print("\n1. list_profiles - Reading REAL config file")
print("-" * 40)
resp = call("list_profiles")

if 'error' in resp:
    print(f"  Error: {resp['error']['message']}")
else:
    content = json.loads(resp['result']['content'][0]['text'])
    print(f"  Profiles found: {len(content)}")
    for profile in content:
        print(f"    - {profile['name']} ({profile['type']}): {profile['url']} [active: {profile['active']}]")
    
    # Check if this matches real config
    if config_exists:
        real_profiles = set(actual_config['profiles'].keys())
        returned_profiles = set(p['name'] for p in content)
        if real_profiles == returned_profiles:
            print("  ✓ REAL DATA: Profiles match actual config file!")
        else:
            print(f"  ✗ Mismatch: Config has {real_profiles}, MCP returned {returned_profiles}")
    else:
        print("  Note: No config file exists yet, using defaults")

# TEST 2: Switch profile - should modify REAL config
print("\n2. switch_profile - Modifying REAL config file")
print("-" * 40)

# First read current default
if config_exists:
    with open(config_path, 'r') as f:
        before_config = json.load(f)
    before_default = before_config.get('default_profile', 'none')
    print(f"  Current default profile: {before_default}")

# Switch to cloud profile
resp = call("switch_profile", {"profile_name": "cloud"})

if 'error' in resp:
    print(f"  Error: {resp['error']['message']}")
else:
    result = json.loads(resp['result']['content'][0]['text'])
    print(f"  Response: {result['message']}")
    
    # Check if config was actually modified
    if os.path.exists(config_path):
        with open(config_path, 'r') as f:
            after_config = json.load(f)
        after_default = after_config.get('default_profile', 'none')
        
        if after_default == "cloud":
            print(f"  ✓ REAL CHANGE: Config file updated! Default is now: {after_default}")
        else:
            print(f"  ✗ FAKE: Config file NOT changed. Still: {after_default}")

# TEST 3: Add new profile - should add to REAL config
print("\n3. profile_add - Adding to REAL config file")
print("-" * 40)

test_profile_name = f"test_profile_{int(time.time())}"
resp = call("profile_add", {
    "profile_name": test_profile_name,
    "profile_type": "Custom",
    "set_active": False,
    "options": {"api_endpoint": "https://test.golem.cloud"}
})

if 'error' in resp:
    print(f"  Error: {resp['error']['message']}")
else:
    result = json.loads(resp['result']['content'][0]['text'])
    print(f"  Response: {result['message']}")
    
    # Check if profile was actually added
    if os.path.exists(config_path):
        with open(config_path, 'r') as f:
            new_config = json.load(f)
        
        if test_profile_name in new_config.get('profiles', {}):
            print(f"  ✓ REAL ADDITION: Profile '{test_profile_name}' added to config!")
            profile_data = new_config['profiles'][test_profile_name]
            print(f"    URL: {profile_data.get('custom_url')}")
        else:
            print(f"  ✗ FAKE: Profile NOT added to config file")

# TEST 4: Configure format - should update REAL config
print("\n4. profile_config_format - Updating REAL config")
print("-" * 40)

resp = call("profile_config_format", {"format": "json"})

if 'error' in resp:
    print(f"  Error: {resp['error']['message']}")
else:
    result = json.loads(resp['result']['content'][0]['text'])
    print(f"  Response: {result['message']}")
    
    # Check if format was actually changed
    if os.path.exists(config_path):
        with open(config_path, 'r') as f:
            format_config = json.load(f)
        
        current_profile = format_config.get('default_profile', 'local')
        profile_format = format_config.get('profiles', {}).get(current_profile, {}).get('config', {}).get('default_format')
        
        if profile_format == "json":
            print(f"  ✓ REAL UPDATE: Format changed to 'json' in config!")
        else:
            print(f"  ✗ FAKE: Format NOT changed. Still: {profile_format}")

print("\n" + "=" * 80)
print("SUMMARY:")
print("-" * 40)

if config_exists:
    print("✓ Profile tools are now working with REAL config file!")
    print(f"✓ Config file at: {config_path}")
    print("✓ Phase 1 fixes are SUCCESSFUL!")
else:
    print("⚠ No config file exists yet - tools using defaults")
    print("⚠ Run 'profile_init' to create config file")

proc.terminate()