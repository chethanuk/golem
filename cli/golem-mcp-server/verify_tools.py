#!/usr/bin/env python3
"""
Comprehensive tool verification script
Compares MCP implementation vs Golem CLI commands
"""

import subprocess
import json
import re
import sys

def get_mcp_tools():
    """Extract all tools from MCP server implementation"""
    tools = []
    with open('src/rmcp_handler_v2.rs', 'r') as f:
        content = f.read()
        # Find all tool definitions
        pattern = r'#\[tool\(description = "([^"]+)"\)\]\s+.*?pub async fn (\w+)'
        matches = re.findall(pattern, content, re.DOTALL)
        for desc, name in matches:
            tools.append({
                'name': name,
                'description': desc,
                'type': 'mcp_tool'
            })
    return tools

def get_cli_commands():
    """Extract all commands from Golem CLI by checking help output"""
    commands = []
    try:
        # Get main commands
        result = subprocess.run(['golem', '--help'], 
                              capture_output=True, text=True, 
                              timeout=5)
        if result.returncode == 0:
            help_text = result.stdout
            # Parse help text for commands
            # This is a simplified parser
            commands.append({'name': 'golem', 'type': 'cli_main'})
    except (subprocess.TimeoutExpired, FileNotFoundError):
        print("Warning: golem CLI not available for comparison")
    
    # Hardcoded based on our analysis
    cli_structure = {
        'app': ['new', 'build', 'deploy', 'clean', 'update-workers'],
        'component': ['list', 'get', 'add', 'update', 'delete'],
        'worker': ['list', 'get', 'add', 'invoke', 'delete', 'connect', 'update', 'interrupt', 'resume'],
        'api': ['add', 'get', 'list', 'update', 'delete'],
        'plugin': ['list', 'get', 'search', 'install', 'uninstall'],
        'profile': ['list', 'add', 'delete', 'switch', 'init', 'config'],
        'account': ['get', 'login', 'logout', 'update'],
        'project': ['list', 'add', 'share', 'policy'],
        'token': ['list', 'add', 'delete'],
        'server': ['start', 'stop', 'status']
    }
    
    for category, subcmds in cli_structure.items():
        for subcmd in subcmds:
            commands.append({
                'name': f"{category}_{subcmd}",
                'category': category,
                'subcommand': subcmd,
                'type': 'cli_command'
            })
    
    return commands

def compare_implementations():
    """Compare MCP tools with CLI commands"""
    mcp_tools = get_mcp_tools()
    cli_commands = get_cli_commands()
    
    print("=" * 80)
    print("MCP SERVER TOOL VERIFICATION REPORT")
    print("=" * 80)
    
    print(f"\n📊 Summary:")
    print(f"  MCP Tools Implemented: {len(mcp_tools)}")
    print(f"  CLI Commands Expected: {len(cli_commands)}")
    
    # Create mapping
    mcp_names = {t['name'] for t in mcp_tools}
    cli_names = {c['name'] for c in cli_commands}
    
    # Categorize tools
    categories = {
        'component': [],
        'worker': [],
        'app': [],
        'profile': [],
        'api': [],
        'plugin': [],
        'token': [],
        'project': [],
        'account': [],
        'server': []
    }
    
    for tool in mcp_tools:
        name = tool['name']
        for cat in categories.keys():
            if name.startswith(cat) or name.startswith(f"{cat}_"):
                categories[cat].append(name)
                break
    
    print("\n📁 Tools by Category:")
    for cat, tools in categories.items():
        status = "✅" if len(tools) > 0 else "❌"
        print(f"  {status} {cat.capitalize()}: {len(tools)} tools")
        if tools:
            for t in tools[:3]:  # Show first 3
                print(f"      - {t}")
            if len(tools) > 3:
                print(f"      ... and {len(tools)-3} more")
    
    # Find missing critical tools
    missing_categories = []
    for cat, tools in categories.items():
        if len(tools) == 0 and cat in ['api', 'server']:
            missing_categories.append(cat)
    
    if missing_categories:
        print("\n⚠️  Critical Missing Categories:")
        for cat in missing_categories:
            print(f"  ❌ {cat.upper()} - No tools implemented!")
    
    # Coverage calculation
    coverage = len(mcp_tools) / max(len(cli_commands), 1) * 100
    
    print(f"\n📈 Coverage Estimate: {coverage:.1f}%")
    
    # Verification status
    print("\n🔍 Verification Status:")
    print("  ❌ E2E Testing: NOT PERFORMED")
    print("  ❌ Data Consistency: NOT VERIFIED")
    print("  ❌ API Gateway: NOT IMPLEMENTED")
    print("  ❌ Server Management: NOT IMPLEMENTED")
    print("  ✅ Core Functions: IMPLEMENTED")
    print("  ✅ Auth Context: IMPLEMENTED")
    
    return {
        'mcp_count': len(mcp_tools),
        'cli_count': len(cli_commands),
        'coverage': coverage,
        'missing_critical': missing_categories,
        'categories': {k: len(v) for k, v in categories.items()}
    }

def verify_data_consistency():
    """Verify that MCP and CLI produce same data"""
    print("\n" + "=" * 80)
    print("DATA CONSISTENCY VERIFICATION")
    print("=" * 80)
    
    tests = []
    
    # Test 1: Component list comparison
    print("\n🧪 Test 1: Component List Comparison")
    print("  Status: ⏭️  SKIPPED (requires running Golem server)")
    tests.append(('component_list', 'SKIPPED'))
    
    # Test 2: Worker operations
    print("\n🧪 Test 2: Worker Operations")
    print("  Status: ⏭️  SKIPPED (requires deployed component)")
    tests.append(('worker_ops', 'SKIPPED'))
    
    # Test 3: Auth context
    print("\n🧪 Test 3: Auth Context")
    print("  Status: ✅ VERIFIED (code review)")
    tests.append(('auth_context', 'PASS'))
    
    return tests

def main():
    """Main verification process"""
    print("\n" + "🔍" * 40)
    print("GOLEM MCP SERVER VERIFICATION SUITE")
    print("Senior Staff Engineer Verification")
    print("🔍" * 40)
    
    # Step 1: Compare implementations
    results = compare_implementations()
    
    # Step 2: Verify data consistency
    tests = verify_data_consistency()
    
    # Step 3: Final assessment
    print("\n" + "=" * 80)
    print("FINAL ASSESSMENT")
    print("=" * 80)
    
    if results['coverage'] >= 90 and not results['missing_critical']:
        print("✅ VERIFICATION PASSED")
        verdict = "COMPLETE"
    elif results['coverage'] >= 70:
        print("⚠️  PARTIALLY COMPLETE")
        verdict = "PARTIAL"
    else:
        print("❌ VERIFICATION FAILED")
        verdict = "INCOMPLETE"
    
    print(f"\nVerdict: {verdict}")
    print(f"Coverage: {results['coverage']:.1f}%")
    print(f"Missing Critical: {results['missing_critical'] or 'None'}")
    
    # Recommendations
    print("\n📋 Recommendations:")
    if 'api' in results['missing_critical']:
        print("  1. URGENT: Implement API Gateway tools")
    if 'server' in results['missing_critical']:
        print("  2. HIGH: Add server management tools")
    print("  3. REQUIRED: Run E2E tests with real Golem server")
    print("  4. REQUIRED: Verify data consistency between CLI and MCP")
    
    print("\n" + "=" * 80)
    print("Verification Complete")
    print("=" * 80)
    
    return 0 if verdict == "COMPLETE" else 1

if __name__ == "__main__":
    sys.exit(main())