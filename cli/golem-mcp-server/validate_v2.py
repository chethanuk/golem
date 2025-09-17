#!/usr/bin/env python3
"""
V2 Validation Script - Verifies all requirements are met
"""

import os
import subprocess
import json
import time
from typing import Dict, Any, Tuple

class V2Validator:
    def __init__(self):
        self.results = {}
        self.process = None
        
    def validate_no_subprocess(self) -> Tuple[bool, str]:
        """Verify V2 does NOT use subprocess execution"""
        with open('src/rmcp_handler_v2.rs', 'r') as f:
            code = f.read()
        
        issues = []
        if 'std::process' in code:
            issues.append("Contains std::process import")
        if 'Command::new' in code:
            issues.append("Contains Command::new")
        if 'golem_cli' in code:
            issues.append("References golem_cli")
        
        if issues:
            return False, f"FAILED: {', '.join(issues)}"
        return True, "PASSED: No subprocess execution"
    
    def validate_direct_api(self) -> Tuple[bool, str]:
        """Verify V2 uses direct API calls"""
        with open('src/rmcp_handler_v2.rs', 'r') as f:
            code = f.read()
        
        checks = []
        if 'GolemApiClient' in code:
            checks.append("Uses GolemApiClient")
        if 'api_client' in code:
            checks.append("Has api_client field")
        if '.await' in code:
            checks.append("Uses async/await")
        
        if len(checks) >= 3:
            return True, f"PASSED: Direct API ({', '.join(checks)})"
        return False, f"FAILED: Missing direct API patterns"
    
    def validate_tools(self) -> Tuple[bool, str]:
        """Verify all 13 tools are implemented"""
        expected_tools = [
            "list_components", "deploy_component", "update_component", "delete_component",
            "list_workers", "create_worker", "invoke_worker", "delete_worker",
            "list_profiles", "switch_profile", "new_app", "build_app", "deploy_app"
        ]
        
        with open('src/rmcp_handler_v2.rs', 'r') as f:
            code = f.read()
        
        found = []
        for tool in expected_tools:
            if f"async fn {tool}" in code:
                found.append(tool)
        
        if len(found) == 13:
            return True, f"PASSED: All 13 tools implemented"
        return False, f"FAILED: Only {len(found)}/13 tools found"
    
    def validate_json_responses(self) -> Tuple[bool, str]:
        """Verify JSON response pattern"""
        with open('src/rmcp_handler_v2.rs', 'r') as f:
            code = f.read()
        
        if 'Content::Text' in code or 'Content::json' in code:
            return True, "PASSED: Uses proper Content responses"
        return False, "FAILED: Missing Content response pattern"
    
    def validate_server_starts(self) -> Tuple[bool, str]:
        """Test that server starts and responds to initialize"""
        try:
            # Start server
            self.process = subprocess.Popen(
                ['cargo', 'run', '--bin', 'golem-mcp-server', '--', '--stdio'],
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.DEVNULL,
                text=True
            )
            
            # Send initialize
            request = {
                "jsonrpc": "2.0",
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {"name": "validator", "version": "1.0.0"}
                },
                "id": 1
            }
            
            self.process.stdin.write(json.dumps(request) + '\n')
            self.process.stdin.flush()
            
            # Read response
            response_line = self.process.stdout.readline()
            if response_line:
                response = json.loads(response_line)
                if 'result' in response:
                    instructions = response['result'].get('instructions', '')
                    server_info = response['result'].get('serverInfo', {})
                    
                    # Check either in top-level instructions or serverInfo.instructions
                    all_instructions = str(instructions) + str(server_info.get('instructions', ''))
                    
                    if 'V2' in all_instructions:
                        return True, "PASSED: V2 server starts and responds"
                    else:
                        return False, f"FAILED: Server is not V2 (instructions: {all_instructions[:100]})"
                        
            return False, "FAILED: No valid response"
            
        except Exception as e:
            return False, f"FAILED: {str(e)}"
        finally:
            if self.process:
                self.process.terminate()
                self.process = None
    
    def run_all_validations(self):
        """Run all validation checks"""
        print("=" * 60)
        print("GOLEM MCP SERVER V2 VALIDATION")
        print("=" * 60)
        print()
        
        validations = [
            ("1. No Subprocess Execution", self.validate_no_subprocess),
            ("2. Direct API Calls", self.validate_direct_api),
            ("3. All 13 Tools Implemented", self.validate_tools),
            ("4. JSON Response Pattern", self.validate_json_responses),
            ("5. Server Starts (V2)", self.validate_server_starts),
        ]
        
        passed = 0
        failed = 0
        
        for name, validator in validations:
            print(f"{name}: ", end="")
            success, message = validator()
            
            if success:
                print(f"✅ {message}")
                passed += 1
            else:
                print(f"❌ {message}")
                failed += 1
        
        print()
        print("=" * 60)
        print(f"RESULTS: {passed} PASSED, {failed} FAILED")
        
        if failed == 0:
            print("✅✅✅ V2 IMPLEMENTATION IS FULLY COMPLIANT ✅✅✅")
        else:
            print("⚠️  Some validations failed - review above")
        print("=" * 60)
        
        return failed == 0

if __name__ == '__main__':
    validator = V2Validator()
    success = validator.run_all_validations()