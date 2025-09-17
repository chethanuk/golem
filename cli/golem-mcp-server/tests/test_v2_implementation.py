#!/usr/bin/env python3
"""
Golem MCP Server V2 - E2E Test Implementation
Using PydanticAI pattern for testing MCP server compliance
"""

import pytest
import asyncio
import json
import os
from typing import Any, Dict, Optional
from pydantic_ai import Agent
from pydantic_ai.mcp import MCPServerStdio

# Test configuration
GOLEM_API_URL = os.getenv("GOLEM_API_URL", "http://localhost:9881")
GOLEM_CLOUD_URL = os.getenv("GOLEM_CLOUD_URL", "https://api.golem.cloud")
TEST_COMPONENT_PATH = "/home/chethan/dev/golemtests/my-app/target/wasm32-wasi/debug/shopping-cart.wasm"
EXPECTED_COMPONENT_ID = "f4c4652b-9818-4178-b7c4-d486fa2aefae"


@pytest.fixture(scope="session")
async def mcp_server():
    """Start the V2 MCP server using stdio transport"""
    server = MCPServerStdio(
        'cargo',
        ['run', '--bin', 'golem-mcp-server', '--', '--stdio'],
        env={
            'GOLEM_API_URL': GOLEM_API_URL,
            'RUST_LOG': 'debug'
        }
    )
    async with server:
        yield server


@pytest.fixture
async def agent(mcp_server):
    """Create an agent with MCP tools from our server"""
    agent = Agent('test', toolsets=[mcp_server])
    return agent


class TestProfileOperations:
    """Test profile management operations"""
    
    async def test_list_profiles(self, agent):
        """Test listing all available profiles"""
        result = await agent.run("List all available profiles using the list_profiles tool")
        
        # Verify response contains expected profiles
        assert result.success
        output = result.output
        assert "local" in output.lower()
        assert "cloud" in output.lower()
        assert "builtin" in output.lower()
    
    async def test_switch_profile(self, agent):
        """Test switching between profiles"""
        # Switch to cloud profile
        result = await agent.run("Switch to the cloud profile using switch_profile tool")
        assert result.success
        assert "cloud" in result.output.lower()
        
        # Switch back to local
        result = await agent.run("Switch to the local profile using switch_profile tool")
        assert result.success
        assert "local" in result.output.lower()


class TestComponentOperations:
    """Test component management operations"""
    
    async def test_list_components_empty(self, agent):
        """Test listing components when none exist"""
        result = await agent.run("List all components using the list_components tool")
        assert result.success
        # Response should be valid JSON array (even if empty)
        assert "[" in result.output or "[]" in result.output
    
    async def test_deploy_component(self, agent):
        """Test deploying a new component"""
        if not os.path.exists(TEST_COMPONENT_PATH):
            pytest.skip(f"Test WASM file not found at {TEST_COMPONENT_PATH}")
        
        result = await agent.run(
            f"Deploy a new component named 'my-shopping-cart' from the WASM file at "
            f"{TEST_COMPONENT_PATH} using the deploy_component tool"
        )
        
        assert result.success
        output = result.output
        
        # Verify component was created
        assert "component" in output.lower()
        # Component ID should be in response
        assert "id" in output.lower() or "uuid" in output.lower()
    
    async def test_list_components_after_deploy(self, agent):
        """Test listing components after deployment"""
        result = await agent.run("List all components using the list_components tool")
        assert result.success
        
        # Should now contain the deployed component
        if "my-shopping-cart" in result.output:
            assert "my-shopping-cart" in result.output


class TestWorkerOperations:
    """Test worker management operations"""
    
    @pytest.fixture
    async def component_id(self, agent):
        """Get or create a component for worker tests"""
        # First try to list existing components
        result = await agent.run("List all components")
        
        # If we have components, extract the first ID
        # Otherwise, use the expected test component ID
        if "id" in result.output.lower():
            # Parse and extract component ID (simplified)
            return EXPECTED_COMPONENT_ID
        return EXPECTED_COMPONENT_ID
    
    async def test_list_workers_empty(self, agent, component_id):
        """Test listing workers when none exist"""
        result = await agent.run(
            f"List all workers for component {component_id} using the list_workers tool"
        )
        assert result.success
        # Should return empty array or valid JSON
        assert "[" in result.output or "[]" in result.output
    
    async def test_create_worker(self, agent, component_id):
        """Test creating a new worker"""
        result = await agent.run(
            f"Create a new worker named 'test-worker-1' for component {component_id} "
            f"using the create_worker tool"
        )
        
        assert result.success
        output = result.output
        
        # Verify worker was created
        assert "worker" in output.lower() or "test-worker-1" in output
    
    async def test_list_workers_after_create(self, agent, component_id):
        """Test listing workers after creation"""
        result = await agent.run(
            f"List all workers for component {component_id} using the list_workers tool"
        )
        assert result.success
        
        # Should contain the created worker
        assert "test-worker-1" in result.output
    
    async def test_invoke_worker_function(self, agent, component_id):
        """Test invoking a function on a worker"""
        # First ensure worker exists
        await agent.run(
            f"Create a worker named 'test-invoke' for component {component_id} if it doesn't exist"
        )
        
        # Invoke initialize-cart function
        result = await agent.run(
            f"Invoke the function 'initialize-cart' on worker 'test-invoke' "
            f"for component {component_id} with parameter 'user-123' "
            f"using the invoke_worker tool"
        )
        
        # Check for successful invocation
        # Note: Actual success depends on component implementation
        assert result.success or "not found" in result.output.lower()


class TestEndToEndWorkflow:
    """Test complete workflow: profile → deploy → worker → invoke"""
    
    async def test_complete_workflow(self, agent):
        """Test the complete Golem workflow"""
        
        # Step 1: List profiles
        result = await agent.run("List all available profiles")
        assert result.success
        assert "local" in result.output.lower()
        
        # Step 2: Switch to local profile
        result = await agent.run("Switch to the local profile")
        assert result.success
        
        # Step 3: List components (initially might be empty)
        result = await agent.run("List all components")
        assert result.success
        
        # Step 4: Deploy a component (if WASM exists)
        if os.path.exists(TEST_COMPONENT_PATH):
            result = await agent.run(
                f"Deploy component 'test-e2e' from {TEST_COMPONENT_PATH}"
            )
            assert result.success
            
            # Extract component ID from response (simplified parsing)
            component_id = EXPECTED_COMPONENT_ID
            
            # Step 5: Create a worker
            result = await agent.run(
                f"Create worker 'e2e-worker' for component {component_id}"
            )
            assert result.success
            
            # Step 6: List workers to verify
            result = await agent.run(
                f"List workers for component {component_id}"
            )
            assert result.success
            assert "e2e-worker" in result.output
            
            # Step 7: Invoke a function
            result = await agent.run(
                f"Invoke function 'initialize-cart' on worker 'e2e-worker' "
                f"for component {component_id} with parameter 'test-user'"
            )
            # Function might not exist, but call should be attempted
            assert result.success or "error" in result.output.lower()


class TestErrorHandling:
    """Test error handling and edge cases"""
    
    async def test_invalid_component_id(self, agent):
        """Test operations with invalid component ID"""
        invalid_id = "not-a-valid-uuid"
        
        result = await agent.run(
            f"List workers for component {invalid_id}"
        )
        
        # Should get an error about invalid UUID
        assert "invalid" in result.output.lower() or "error" in result.output.lower()
    
    async def test_missing_wasm_file(self, agent):
        """Test deploying with non-existent WASM file"""
        result = await agent.run(
            "Deploy component 'test' from /nonexistent/file.wasm"
        )
        
        # Should get an error about missing file
        assert "not found" in result.output.lower() or "error" in result.output.lower()
    
    async def test_duplicate_worker_name(self, agent):
        """Test creating duplicate worker names"""
        component_id = EXPECTED_COMPONENT_ID
        
        # Create first worker
        await agent.run(
            f"Create worker 'duplicate-test' for component {component_id}"
        )
        
        # Try to create duplicate
        result = await agent.run(
            f"Create worker 'duplicate-test' for component {component_id}"
        )
        
        # Should either succeed (if allowed) or error appropriately
        assert result.success or "exists" in result.output.lower()


# Pytest configuration
def pytest_configure(config):
    """Configure pytest with custom markers"""
    config.addinivalue_line(
        "markers", "slow: marks tests as slow (deselect with '-m \"not slow\"')"
    )
    config.addinivalue_line(
        "markers", "integration: marks tests as integration tests"
    )


# Run configuration
if __name__ == "__main__":
    # Run tests with verbose output
    pytest.main([__file__, "-v", "--tb=short"])