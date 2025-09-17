#!/usr/bin/env python3
"""
Golem MCP Server V2 Test Runner
Using PydanticAI pattern for testing MCP operations
"""

import asyncio
import json
import subprocess
import sys
import time
from dataclasses import dataclass
from typing import Any, Dict, List, Optional
from pathlib import Path

# Check for required packages
try:
    import httpx
    from pydantic import BaseModel, Field
except ImportError:
    print("Installing required packages...")
    subprocess.check_call([sys.executable, "-m", "pip", "install", "httpx", "pydantic"])
    import httpx
    from pydantic import BaseModel, Field


class MCPRequest(BaseModel):
    """MCP Request Model"""
    jsonrpc: str = "2.0"
    id: int
    method: str
    params: Optional[Dict[str, Any]] = Field(default_factory=dict)


class MCPResponse(BaseModel):
    """MCP Response Model"""
    jsonrpc: str = "2.0"
    id: int
    result: Optional[Dict[str, Any]] = None
    error: Optional[Dict[str, Any]] = None


class TestResult(BaseModel):
    """Test Result Model"""
    test_name: str
    passed: bool
    duration_ms: float
    error: Optional[str] = None
    response: Optional[Dict[str, Any]] = None


@dataclass
class MCPTestClient:
    """Simple MCP Test Client using stdio communication"""
    
    server_process: Optional[subprocess.Popen] = None
    request_id: int = 1
    
    async def start_server(self) -> bool:
        """Start the MCP server"""
        try:
            # Build the server first
            print("Building MCP server...")
            build_result = subprocess.run(
                ["cargo", "build", "--release"],
                cwd=Path(__file__).parent,
                capture_output=True,
                text=True
            )
            
            if build_result.returncode != 0:
                print(f"Build failed: {build_result.stderr}")
                return False
            
            # Start the server
            print("Starting MCP server...")
            self.server_process = subprocess.Popen(
                ["cargo", "run", "--release"],
                cwd=Path(__file__).parent,
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                bufsize=0
            )
            
            # Wait for server to be ready
            await asyncio.sleep(2)
            
            # Send initialize request
            init_request = MCPRequest(
                id=self.request_id,
                method="initialize",
                params={
                    "protocolVersion": "0.1.0",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "test-client",
                        "version": "1.0.0"
                    }
                }
            )
            
            response = await self.send_request(init_request)
            if response and response.result:
                print("Server initialized successfully")
                return True
            
            print("Failed to initialize server")
            return False
            
        except Exception as e:
            print(f"Failed to start server: {e}")
            return False
    
    async def send_request(self, request: MCPRequest) -> Optional[MCPResponse]:
        """Send a request to the MCP server"""
        if not self.server_process:
            return None
        
        try:
            # Send request
            request_json = request.model_dump_json()
            self.server_process.stdin.write(request_json + "\n")
            self.server_process.stdin.flush()
            
            # Read response
            response_line = self.server_process.stdout.readline()
            if response_line:
                response_data = json.loads(response_line)
                return MCPResponse(**response_data)
            
        except Exception as e:
            print(f"Request failed: {e}")
        
        return None
    
    async def call_tool(self, tool_name: str, arguments: Dict[str, Any]) -> TestResult:
        """Call an MCP tool and return the result"""
        start_time = time.time()
        
        self.request_id += 1
        request = MCPRequest(
            id=self.request_id,
            method="tools/call",
            params={
                "name": tool_name,
                "arguments": arguments
            }
        )
        
        response = await self.send_request(request)
        duration_ms = (time.time() - start_time) * 1000
        
        if not response:
            return TestResult(
                test_name=f"call_{tool_name}",
                passed=False,
                duration_ms=duration_ms,
                error="No response from server"
            )
        
        if response.error:
            return TestResult(
                test_name=f"call_{tool_name}",
                passed=False,
                duration_ms=duration_ms,
                error=str(response.error)
            )
        
        return TestResult(
            test_name=f"call_{tool_name}",
            passed=True,
            duration_ms=duration_ms,
            response=response.result
        )
    
    async def list_tools(self) -> Optional[List[Dict[str, Any]]]:
        """List available tools"""
        self.request_id += 1
        request = MCPRequest(
            id=self.request_id,
            method="tools/list",
            params={}
        )
        
        response = await self.send_request(request)
        if response and response.result:
            return response.result.get("tools", [])
        return None
    
    def stop_server(self):
        """Stop the MCP server"""
        if self.server_process:
            self.server_process.terminate()
            self.server_process.wait(timeout=5)
            self.server_process = None


async def run_tests():
    """Run MCP V2 tests"""
    print("=" * 60)
    print("Golem MCP Server V2 Test Suite")
    print("=" * 60)
    
    client = MCPTestClient()
    test_results: List[TestResult] = []
    
    try:
        # Start server
        print("\n1. Starting MCP server...")
        if not await client.start_server():
            print("Failed to start server")
            return
        
        # List tools
        print("\n2. Listing available tools...")
        tools = await client.list_tools()
        if tools:
            print(f"Found {len(tools)} tools:")
            for tool in tools:
                print(f"  - {tool.get('name')}: {tool.get('description')}")
            
            # Verify expected V2 tools
            tool_names = {tool.get('name') for tool in tools}
            expected_tools = {
                'list_components',
                'deploy_component',
                'list_workers',
                'create_worker',
                'invoke_worker',
                'list_profiles',
                'switch_profile'
            }
            
            missing_tools = expected_tools - tool_names
            if missing_tools:
                print(f"\nWarning: Missing expected tools: {missing_tools}")
        else:
            print("Failed to list tools")
        
        # Test cases
        print("\n3. Running test cases...")
        
        # Test 1: List components
        print("\n  Test: list_components")
        result = await client.call_tool("list_components", {})
        test_results.append(result)
        print(f"    Result: {'PASS' if result.passed else 'FAIL'} ({result.duration_ms:.1f}ms)")
        
        # Test 2: List profiles
        print("\n  Test: list_profiles")
        result = await client.call_tool("list_profiles", {})
        test_results.append(result)
        print(f"    Result: {'PASS' if result.passed else 'FAIL'} ({result.duration_ms:.1f}ms)")
        
        # Test 3: Invalid tool call
        print("\n  Test: invalid_tool (expected to fail)")
        result = await client.call_tool("invalid_tool_name", {})
        # This should fail, so we invert the pass condition
        result.passed = not result.passed
        test_results.append(result)
        print(f"    Result: {'PASS' if result.passed else 'FAIL'} ({result.duration_ms:.1f}ms)")
        
        # Test 4: Deploy component (will fail without WASM file)
        print("\n  Test: deploy_component (expected to fail - no WASM)")
        result = await client.call_tool("deploy_component", {
            "name": "test-component",
            "wasm_path": "/nonexistent.wasm"
        })
        # Expected to fail due to missing file
        if "Cannot read WASM file" in str(result.error):
            result.passed = True
        test_results.append(result)
        print(f"    Result: {'PASS' if result.passed else 'FAIL'} ({result.duration_ms:.1f}ms)")
        
    finally:
        # Stop server
        print("\n4. Stopping server...")
        client.stop_server()
    
    # Summary
    print("\n" + "=" * 60)
    print("Test Summary")
    print("=" * 60)
    
    total_tests = len(test_results)
    passed_tests = sum(1 for r in test_results if r.passed)
    failed_tests = total_tests - passed_tests
    total_duration = sum(r.duration_ms for r in test_results)
    
    print(f"Total tests: {total_tests}")
    print(f"Passed: {passed_tests}")
    print(f"Failed: {failed_tests}")
    print(f"Total duration: {total_duration:.1f}ms")
    
    if failed_tests > 0:
        print("\nFailed tests:")
        for result in test_results:
            if not result.passed and result.error:
                print(f"  - {result.test_name}: {result.error}")
    
    print("\n" + "=" * 60)
    if failed_tests == 0:
        print("All tests passed! ✅")
    else:
        print(f"Some tests failed. Please review. ⚠️")
    
    return failed_tests == 0


if __name__ == "__main__":
    success = asyncio.run(run_tests())
    sys.exit(0 if success else 1)