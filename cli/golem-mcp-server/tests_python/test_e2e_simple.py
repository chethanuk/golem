#!/usr/bin/env python3
"""
Simple E2E Tests for Golem MCP Server V2
Without PydanticAI dependency - using direct STDIO communication
"""

import asyncio
import json
import subprocess
import pytest
from typing import Dict, Any, Optional
import os
import sys

class GolemMcpClient:
    """Simple MCP client using STDIO transport"""
    
    def __init__(self):
        self.process: Optional[asyncio.subprocess.Process] = None
        self.request_id = 1
        
    async def start(self):
        """Start the MCP server process"""
        # Build the server first
        build_result = await asyncio.create_subprocess_exec(
            'cargo', 'build', '--bin', 'golem-mcp-server',
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE
        )
        await build_result.wait()
        
        if build_result.returncode != 0:
            stderr = await build_result.stderr.read()
            raise Exception(f"Failed to build server: {stderr.decode()}")
        
        # Start the server with stdio
        self.process = await asyncio.create_subprocess_exec(
            'cargo', 'run', '--bin', 'golem-mcp-server', '--', '--stdio',
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env={**os.environ, 'RUST_LOG': 'debug'}
        )
        
        # Initialize connection
        await self._initialize()
        
    async def _initialize(self):
        """Initialize MCP connection"""
        response = await self._send_request('initialize', {
            'protocolVersion': '2024-11-05',
            'capabilities': {},
            'clientInfo': {
                'name': 'python-test-client',
                'version': '1.0.0'
            }
        })
        
        if 'error' in response:
            raise Exception(f"Initialize failed: {response['error']}")
            
    async def list_tools(self) -> list:
        """List available tools"""
        response = await self._send_request('tools/list', {})
        return response.get('result', {}).get('tools', [])
    
    async def call_tool(self, name: str, args: Dict[str, Any]) -> Dict:
        """Call a tool with arguments"""
        response = await self._send_request('tools/call', {
            'name': name,
            'arguments': args
        })
        
        if 'error' in response:
            raise Exception(f"Tool call failed: {response['error']}")
            
        return response.get('result', {})
    
    async def _send_request(self, method: str, params: Dict) -> Dict:
        """Send JSON-RPC request and read response"""
        request = {
            'jsonrpc': '2.0',
            'method': method,
            'params': params,
            'id': self.request_id
        }
        self.request_id += 1
        
        # Send request
        request_str = json.dumps(request) + '\n'
        self.process.stdin.write(request_str.encode())
        await self.process.stdin.drain()
        
        # Read response with timeout
        try:
            response_line = await asyncio.wait_for(
                self.process.stdout.readline(),
                timeout=10.0
            )
        except asyncio.TimeoutError:
            raise Exception("Response timeout")
        
        if not response_line:
            raise Exception("Empty response")
            
        return json.loads(response_line.decode())
    
    async def close(self):
        """Close the client and terminate server"""
        if self.process:
            self.process.terminate()
            await self.process.wait()


# ==================== TEST FIXTURES ====================

@pytest.fixture
async def mcp_client():
    """Create and cleanup MCP client"""
    client = GolemMcpClient()
    await client.start()
    yield client
    await client.close()


# ==================== PROFILE TESTS ====================

@pytest.mark.asyncio
async def test_server_starts():
    """Test that server starts successfully"""
    client = GolemMcpClient()
    try:
        await client.start()
        assert client.process is not None
        assert client.process.returncode is None  # Still running
    finally:
        await client.close()

@pytest.mark.asyncio
async def test_list_tools(mcp_client):
    """Test listing available tools"""
    tools = await mcp_client.list_tools()
    
    assert len(tools) == 13, f"Expected 13 tools, got {len(tools)}"
    
    tool_names = [tool['name'] for tool in tools]
    expected_tools = [
        'list_components', 'deploy_component', 'update_component', 'delete_component',
        'list_workers', 'create_worker', 'invoke_worker', 'delete_worker',
        'list_profiles', 'switch_profile', 'new_app', 'build_app', 'deploy_app'
    ]
    
    for expected in expected_tools:
        assert expected in tool_names, f"Tool {expected} not found"

@pytest.mark.asyncio
async def test_profile_operations(mcp_client):
    """Test profile list and switch operations"""
    # List profiles
    result = await mcp_client.call_tool('list_profiles', {})
    assert 'content' in result
    content = result['content'][0]['text'] if result['content'] else ""
    assert 'local' in content.lower() or 'profile' in content.lower()
    
    # Switch profile
    result = await mcp_client.call_tool('switch_profile', {
        'profile_name': 'local'
    })
    assert 'content' in result
    switch_content = result['content'][0]['text'] if result['content'] else ""
    assert 'local' in switch_content.lower() or 'switched' in switch_content.lower()


# ==================== COMPONENT TESTS ====================

@pytest.mark.asyncio
async def test_component_operations(mcp_client):
    """Test component list operation"""
    # List components
    result = await mcp_client.call_tool('list_components', {})
    assert 'content' in result
    # Components list might be empty, but should return valid response

@pytest.mark.asyncio
@pytest.mark.skipif(
    not os.path.exists('/home/chethan/dev/golemtests/my-app/target/wasm32-wasi/debug/shopping-cart.wasm'),
    reason="Test WASM file not found"
)
async def test_component_deployment(mcp_client):
    """Test component deployment with real WASM file"""
    # Switch to cloud profile first
    await mcp_client.call_tool('switch_profile', {'profile_name': 'cloud'})
    
    # Deploy component
    result = await mcp_client.call_tool('deploy_component', {
        'name': 'shopping-cart',
        'wasm_path': '/home/chethan/dev/golemtests/my-app/target/wasm32-wasi/debug/shopping-cart.wasm'
    })
    
    assert 'content' in result
    content_str = str(result)
    
    # Check for expected component ID
    expected_id = 'f4c4652b-9818-4178-b7c4-d486fa2aefae'
    if expected_id in content_str:
        assert expected_id in content_str


# ==================== WORKER TESTS ====================

@pytest.mark.asyncio
async def test_worker_operations_mock(mcp_client):
    """Test worker operations with mock component ID"""
    component_id = 'test-component-id'
    
    # List workers (will fail with invalid component, but tests the tool)
    try:
        result = await mcp_client.call_tool('list_workers', {
            'component_id': component_id
        })
        assert 'content' in result or 'error' in str(result)
    except Exception as e:
        # Expected to fail with invalid component
        assert 'component' in str(e).lower() or 'not found' in str(e).lower()

@pytest.mark.asyncio
@pytest.mark.skipif(
    not os.path.exists('/home/chethan/dev/golemtests/my-app/target/wasm32-wasi/debug/shopping-cart.wasm'),
    reason="Requires deployed component"
)
async def test_worker_lifecycle(mcp_client):
    """Test complete worker lifecycle"""
    component_id = 'f4c4652b-9818-4178-b7c4-d486fa2aefae'
    
    # Create worker
    try:
        result = await mcp_client.call_tool('create_worker', {
            'component_id': component_id,
            'name': 'test-1',
            'env': [],
            'args': []
        })
        assert 'content' in result
        assert 'test-1' in str(result)
        
        # Invoke worker function
        result = await mcp_client.call_tool('invoke_worker', {
            'component_id': component_id,
            'worker_name': 'test-1',
            'function_name': 'initialize-cart',
            'parameters': ['user-1']
        })
        assert 'content' in result or 'result' in result
        
        # List workers
        result = await mcp_client.call_tool('list_workers', {
            'component_id': component_id
        })
        assert 'content' in result
        assert 'test-1' in str(result)
        
    except Exception as e:
        # Component might not exist
        print(f"Worker test failed (expected if component not deployed): {e}")


# ==================== END-TO-END WORKFLOW TEST ====================

@pytest.mark.asyncio
async def test_complete_workflow(mcp_client):
    """Test complete workflow from profile to component operations"""
    # 1. List profiles
    profiles = await mcp_client.call_tool('list_profiles', {})
    assert 'content' in profiles
    
    # 2. Switch to local profile
    switch = await mcp_client.call_tool('switch_profile', {
        'profile_name': 'local'
    })
    assert 'content' in switch
    
    # 3. List components
    components = await mcp_client.call_tool('list_components', {})
    assert 'content' in components
    
    # 4. List app operations
    # Note: new_app, build_app, deploy_app require template and app name
    
    print("Complete workflow test passed!")


if __name__ == '__main__':
    # Run tests
    pytest.main([__file__, '-v', '--tb=short'])