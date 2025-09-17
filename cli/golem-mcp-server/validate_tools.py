#!/usr/bin/env python3
"""
Systematic validation of all Golem MCP Server V2 tools
Tests each tool and updates the validation report
"""

import asyncio
import json
import subprocess
import sys
import os
from typing import Dict, Any, Optional, Tuple
from datetime import datetime

class ToolValidator:
    """Validates MCP Server tools systematically"""
    
    def __init__(self):
        self.process: Optional[asyncio.subprocess.Process] = None
        self.request_id = 1
        self.results = {}
        self.validation_report = "/home/chethan/dev/golem/cli/golem-mcp-server/.plan/Validate.MD"
        
    async def start_server(self) -> bool:
        """Start the MCP server"""
        try:
            # Build first
            print("Building MCP server...")
            build = await asyncio.create_subprocess_exec(
                'cargo', 'build', '--bin', 'golem-mcp-server',
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE
            )
            await build.wait()
            
            if build.returncode != 0:
                stderr = await build.stderr.read()
                print(f"Build failed: {stderr.decode()}")
                return False
            
            # Start server
            print("Starting MCP server...")
            self.process = await asyncio.create_subprocess_exec(
                'cargo', 'run', '--bin', 'golem-mcp-server', '--', '--stdio',
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                env={**os.environ, 'RUST_LOG': 'info'}
            )
            
            # Initialize
            await self._initialize()
            return True
            
        except Exception as e:
            print(f"Failed to start server: {e}")
            return False
    
    async def _initialize(self):
        """Initialize MCP connection"""
        # Wait for server to be ready
        await asyncio.sleep(1)
        
        response = await self._send_request('initialize', {
            'protocolVersion': '2024-11-05',
            'capabilities': {},
            'clientInfo': {
                'name': 'validator',
                'version': '1.0.0'
            }
        })
        
        if 'error' in response:
            raise Exception(f"Initialize failed: {response['error']}")
    
    async def _send_request(self, method: str, params: Dict) -> Dict:
        """Send JSON-RPC request"""
        request = {
            'jsonrpc': '2.0',
            'method': method,
            'params': params,
            'id': self.request_id
        }
        self.request_id += 1
        
        # Send
        request_str = json.dumps(request) + '\n'
        self.process.stdin.write(request_str.encode())
        await self.process.stdin.drain()
        
        # Read response, skipping log lines
        max_attempts = 10
        for _ in range(max_attempts):
            try:
                response_line = await asyncio.wait_for(
                    self.process.stdout.readline(),
                    timeout=5.0
                )
            except asyncio.TimeoutError:
                return {'error': 'Timeout'}
            
            if not response_line:
                return {'error': 'Empty response'}
            
            line = response_line.decode().strip()
            # Skip log lines (they contain timestamps and log levels)
            if line.startswith('{') and '"jsonrpc"' in line:
                try:
                    return json.loads(line)
                except json.JSONDecodeError as e:
                    print(f"Failed to parse JSON: {line}")
                    continue
            else:
                # Skip non-JSON lines (logs)
                continue
        
        return {'error': 'No valid JSON response after multiple attempts'}
    
    async def validate_tool(self, name: str, args: Dict, description: str) -> Tuple[str, str, str]:
        """Validate a single tool"""
        print(f"\nValidating: {name}")
        print(f"  Args: {json.dumps(args, indent=2)}")
        
        try:
            response = await self._send_request('tools/call', {
                'name': name,
                'arguments': args
            })
            
            if 'error' in response:
                error_msg = json.dumps(response['error'], indent=2)
                print(f"  ❌ Error: {error_msg}")
                return ('❌ FAILED', error_msg, response)
            else:
                # Check for content in result
                result = response.get('result', {})
                if 'content' in result and result['content']:
                    content = result['content'][0].get('text', 'No text')
                    print(f"  ✅ Success: {content[:100]}...")
                    return ('✅ PASSED', content[:500], response)
                else:
                    print(f"  ✅ Success (empty response)")
                    return ('✅ PASSED', 'Empty response', response)
                    
        except Exception as e:
            error_msg = str(e)
            print(f"  ❌ Exception: {error_msg}")
            return ('❌ FAILED', error_msg, {})
    
    async def run_validation(self):
        """Run all tool validations"""
        
        # Define test cases
        test_cases = [
            ('list_components', {}, 'List all components'),
            ('deploy_component', {
                'name': 'test-component',
                'wasm_path': '/tmp/test.wasm'
            }, 'Deploy a new component from WASM file'),
            ('update_component', {
                'component_id': 'f4c4652b-9818-4178-b7c4-d486fa2aefae',
                'wasm_path': '/tmp/test.wasm'
            }, 'Update an existing component with new WASM'),
            ('delete_component', {
                'component_id': 'test-component-id'
            }, 'Delete a component'),
            ('list_workers', {
                'component_id': 'f4c4652b-9818-4178-b7c4-d486fa2aefae'
            }, 'List all workers for a component'),
            ('create_worker', {
                'component_id': 'f4c4652b-9818-4178-b7c4-d486fa2aefae',
                'name': 'test-worker',
                'env': [],
                'args': []
            }, 'Create a new worker instance'),
            ('invoke_worker', {
                'component_id': 'f4c4652b-9818-4178-b7c4-d486fa2aefae',
                'worker_name': 'test-worker',
                'function_name': 'initialize',
                'parameters': []
            }, 'Invoke a function on a worker'),
            ('delete_worker', {
                'component_id': 'f4c4652b-9818-4178-b7c4-d486fa2aefae',
                'worker_name': 'test-worker'
            }, 'Delete a worker instance'),
            ('list_profiles', {}, 'List all available profiles'),
            ('switch_profile', {
                'profile_name': 'local'
            }, 'Switch to a different profile'),
            ('new_app', {
                'app_name': 'test-app',
                'template': 'rust'
            }, 'Create a new app from template'),
            ('build_app', {
                'app_name': 'test-app'
            }, 'Build an application'),
            ('deploy_app', {
                'app_name': 'test-app'
            }, 'Deploy an application'),
        ]
        
        # Run each test
        for name, args, desc in test_cases:
            status, result, raw = await self.validate_tool(name, args, desc)
            self.results[name] = {
                'status': status,
                'result': result,
                'raw': raw,
                'description': desc
            }
        
        # Update report
        await self.update_report()
    
    async def update_report(self):
        """Update the validation report"""
        print("\n\nUpdating validation report...")
        
        # Read current report
        with open(self.validation_report, 'r') as f:
            lines = f.readlines()
        
        # Build new report
        new_report = []
        tool_index = 1
        
        for line in lines:
            if line.startswith('### ') and '. ' in line:
                # Extract tool name
                tool_name = line.split('. ')[1].strip().split('\n')[0]
                
                if tool_name in self.results:
                    result = self.results[tool_name]
                    # Keep the header
                    new_report.append(line)
                    
                    # Update status
                    new_report.append(f"**Description:** {result['description']}\n")
                    new_report.append(f"**Status:** {result['status']}\n")
                    
                    # Skip old content until Result:
                    while lines and not lines[0].startswith('**Result:**'):
                        lines.pop(0)
                    
                    # Add result
                    new_report.append("**Result:**\n")
                    new_report.append("```json\n")
                    if result['status'] == '✅ PASSED':
                        new_report.append(json.dumps(result.get('raw', {}), indent=2)[:1000] + '\n')
                    else:
                        new_report.append(f"Error: {result['result']}\n")
                    new_report.append("```\n\n")
                    
                    # Skip old result
                    while lines and not lines[0].startswith('### '):
                        lines.pop(0)
                else:
                    new_report.append(line)
            else:
                new_report.append(line)
        
        # Update summary table
        summary_lines = []
        summary_lines.append("\n## Summary\n\n")
        summary_lines.append("| Tool | Status | Notes |\n")
        summary_lines.append("|------|--------|-------|\n")
        
        passed = 0
        failed = 0
        
        for name, result in self.results.items():
            status_icon = result['status'].split()[0]
            if '✅' in status_icon:
                passed += 1
                note = 'Working'
            else:
                failed += 1
                note = result['result'][:50] + '...' if len(result['result']) > 50 else result['result']
            
            summary_lines.append(f"| {name} | {status_icon} | {note} |\n")
        
        summary_lines.append(f"\n**Total:** {len(self.results)} tools tested\n")
        summary_lines.append(f"**Passed:** {passed}\n")
        summary_lines.append(f"**Failed:** {failed}\n")
        summary_lines.append(f"\n## Test Execution Log\n")
        summary_lines.append(f"**Completed:** {datetime.now().isoformat()}\n")
        
        # Write updated report
        final_report = ''.join(new_report[:100]) + ''.join(summary_lines)
        
        with open(self.validation_report, 'w') as f:
            f.write(final_report)
        
        print(f"Report updated: {self.validation_report}")
        print(f"\nResults: {passed} passed, {failed} failed")
    
    async def close(self):
        """Close the validator"""
        if self.process:
            self.process.terminate()
            await self.process.wait()


async def main():
    """Main validation routine"""
    validator = ToolValidator()
    
    try:
        # Start server
        if not await validator.start_server():
            print("Failed to start server")
            return 1
        
        print("Server started successfully")
        
        # Run validation
        await validator.run_validation()
        
        return 0
        
    except Exception as e:
        print(f"Validation error: {e}")
        return 1
        
    finally:
        await validator.close()


if __name__ == '__main__':
    sys.exit(asyncio.run(main()))