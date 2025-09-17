#!/usr/bin/env python3
"""Comprehensive verification: Which tools are REAL vs FAKE"""

import subprocess
import re

# Read the source file
with open('/home/chethan/dev/golem/cli/golem-mcp-server/src/rmcp_handler_v2.rs', 'r') as f:
    content = f.read()

# Find all tool implementations
tool_pattern = re.compile(r'#\[tool\(description = "([^"]+)"\)\]\s+#\[instrument[^\]]*\]\s+pub async fn (\w+)', re.MULTILINE)
tools = tool_pattern.findall(content)

print("=" * 80)
print("COMPREHENSIVE TOOL VERIFICATION - REAL vs FAKE")
print("=" * 80)

# Categorize tools based on implementation
categories = {
    "REAL_API": [],
    "REAL_CONFIG": [],
    "FAKE_HARDCODED": [],
    "FAKE_SIMULATED": [],
    "PLACEHOLDER": []
}

for desc, func_name in tools:
    # Find the function implementation
    func_pattern = re.compile(rf'pub async fn {func_name}.*?\n    \}}', re.DOTALL)
    func_match = func_pattern.search(content)
    
    if func_match:
        func_body = func_match.group(0)
        
        # Check for real API calls
        if 'self.api_client.' in func_body:
            categories["REAL_API"].append((func_name, desc))
        # Check for real config file operations
        elif 'std::fs::read_to_string' in func_body or 'std::fs::write' in func_body:
            categories["REAL_CONFIG"].append((func_name, desc))
        # Check for simulated/fake data
        elif 'Simulate' in func_body or 'simulate' in func_body.lower():
            categories["FAKE_SIMULATED"].append((func_name, desc))
        # Check for hardcoded responses
        elif 'serde_json::json!({' in func_body and 'self.api_client' not in func_body:
            # Check if it's just returning hardcoded data
            if 'placeholder' in func_body.lower() or 'todo' in func_body.lower():
                categories["PLACEHOLDER"].append((func_name, desc))
            else:
                categories["FAKE_HARDCODED"].append((func_name, desc))
        else:
            # Unclear - needs manual review
            categories["PLACEHOLDER"].append((func_name, desc))

# Print results
print(f"\nTotal tools found: {len(tools)}")
print("=" * 80)

print(f"\n✅ REAL API CALLS ({len(categories['REAL_API'])} tools)")
print("-" * 40)
for func, desc in categories["REAL_API"]:
    print(f"  {func:30} - {desc}")

print(f"\n✅ REAL CONFIG FILES ({len(categories['REAL_CONFIG'])} tools)")
print("-" * 40)
for func, desc in categories["REAL_CONFIG"]:
    print(f"  {func:30} - {desc}")

print(f"\n❌ FAKE SIMULATED ({len(categories['FAKE_SIMULATED'])} tools)")
print("-" * 40)
for func, desc in categories["FAKE_SIMULATED"]:
    print(f"  {func:30} - {desc}")

print(f"\n❌ FAKE HARDCODED ({len(categories['FAKE_HARDCODED'])} tools)")
print("-" * 40)
for func, desc in categories["FAKE_HARDCODED"]:
    print(f"  {func:30} - {desc}")

print(f"\n⚠️  PLACEHOLDER/UNCLEAR ({len(categories['PLACEHOLDER'])} tools)")
print("-" * 40)
for func, desc in categories["PLACEHOLDER"]:
    print(f"  {func:30} - {desc}")

# Summary statistics
total = len(tools)
real = len(categories['REAL_API']) + len(categories['REAL_CONFIG'])
fake = len(categories['FAKE_SIMULATED']) + len(categories['FAKE_HARDCODED']) + len(categories['PLACEHOLDER'])

print("\n" + "=" * 80)
print("SUMMARY:")
print("-" * 40)
print(f"REAL implementations: {real}/{total} ({real*100//total}%)")
print(f"  - API calls: {len(categories['REAL_API'])}")
print(f"  - Config files: {len(categories['REAL_CONFIG'])}")
print(f"FAKE implementations: {fake}/{total} ({fake*100//total}%)")
print(f"  - Simulated: {len(categories['FAKE_SIMULATED'])}")
print(f"  - Hardcoded: {len(categories['FAKE_HARDCODED'])}")
print(f"  - Placeholders: {len(categories['PLACEHOLDER'])}")

# List specific fake tools that need fixing
print("\n" + "=" * 80)
print("FAKE TOOLS THAT NEED FIXING:")
print("-" * 40)

fake_tools_to_fix = (
    categories['FAKE_SIMULATED'] + 
    categories['FAKE_HARDCODED'] + 
    categories['PLACEHOLDER']
)

for i, (func, desc) in enumerate(fake_tools_to_fix, 1):
    print(f"{i:2}. {func:30} - {desc}")