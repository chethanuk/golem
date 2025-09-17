#!/usr/bin/env python3
"""FINAL VERIFICATION: Check which tools ACTUALLY use the API client"""

import re

# Read the source
with open('/home/chethan/dev/golem/cli/golem-mcp-server/src/rmcp_handler_v2.rs', 'r') as f:
    lines = f.readlines()

# Find all tool functions
current_func = None
func_lines = {}
in_function = False
bracket_count = 0

for i, line in enumerate(lines):
    # Detect tool function start
    if 'pub async fn' in line and current_func is None:
        match = re.search(r'pub async fn (\w+)', line)
        if match:
            current_func = match.group(1)
            func_lines[current_func] = []
            in_function = True
            bracket_count = 0
    
    # Track function body
    if in_function and current_func:
        func_lines[current_func].append(line)
        bracket_count += line.count('{') - line.count('}')
        
        # Function ends when brackets balance
        if bracket_count == 0 and '{' in ''.join(func_lines[current_func]):
            in_function = False
            current_func = None

print("=" * 80)
print("FINAL TOOL VERIFICATION - API CLIENT USAGE")
print("=" * 80)

# Check each function for API client usage
real_api_tools = []
fake_tools = []
profile_tools = []

for func_name, body in func_lines.items():
    body_text = ''.join(body)
    
    # Check for api_client usage
    if 'self.api_client' in body_text:
        real_api_tools.append(func_name)
    # Check for config file operations (profile tools)
    elif 'config-v2.json' in body_text or 'std::fs::' in body_text:
        profile_tools.append(func_name)
    # Check for fake patterns
    elif any(pattern in body_text for pattern in [
        'Simulate', 'simulate', 'placeholder', 
        'would fetch', 'in production', 'todo', 'TODO'
    ]):
        fake_tools.append(func_name)
    else:
        # Check if just returning hardcoded JSON
        if 'serde_json::json!({' in body_text and 'self.api_client' not in body_text:
            fake_tools.append(func_name)

print(f"\n✅ REAL API TOOLS ({len(real_api_tools)})")
print("-" * 40)
for func in sorted(real_api_tools):
    # Check what API method is called
    for line in func_lines[func]:
        if 'self.api_client.' in line:
            api_call = re.search(r'self\.api_client\.(\w+)', line)
            if api_call:
                print(f"  {func:30} -> api_client.{api_call.group(1)}()")
                break

print(f"\n✅ PROFILE/CONFIG TOOLS ({len(profile_tools)})")
print("-" * 40)
for func in sorted(profile_tools):
    print(f"  {func:30} - reads/writes config-v2.json")

print(f"\n❌ FAKE/SIMULATED TOOLS ({len(fake_tools)})")
print("-" * 40)
for func in sorted(fake_tools):
    # Find the fake indicator
    body_text = ''.join(func_lines[func])
    if 'Simulate' in body_text or 'simulate' in body_text:
        print(f"  {func:30} - SIMULATED")
    elif 'placeholder' in body_text or 'TODO' in body_text:
        print(f"  {func:30} - PLACEHOLDER")
    else:
        print(f"  {func:30} - HARDCODED")

# Summary
total = len(func_lines)
real = len(real_api_tools) + len(profile_tools)
fake = len(fake_tools)
unknown = total - real - fake

print("\n" + "=" * 80)
print("SUMMARY:")
print("-" * 40)
print(f"Total tools: {total}")
print(f"REAL: {real} ({real*100//total if total else 0}%)")
print(f"  - API calls: {len(real_api_tools)}")
print(f"  - Config files: {len(profile_tools)}")
print(f"FAKE: {fake} ({fake*100//total if total else 0}%)")
print(f"UNKNOWN: {unknown}")

if unknown > 0:
    print("\nUNKNOWN TOOLS (need manual review):")
    unknown_tools = set(func_lines.keys()) - set(real_api_tools) - set(profile_tools) - set(fake_tools)
    for func in sorted(unknown_tools):
        print(f"  - {func}")