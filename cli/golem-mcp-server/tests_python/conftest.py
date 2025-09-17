"""
PydanticAI MCP test fixtures for Golem MCP Server.

Following KISS principle: Single fixture file, minimal complexity.
"""

import asyncio
import os
from pathlib import Path
from typing import AsyncGenerator

import pytest
from pydantic_ai import Agent
from pydantic_ai.mcp import MCPServerStdio


@pytest.fixture(scope="session")
async def mcp_agent() -> AsyncGenerator[Agent, None]:
    """
    Production-ready MCP client using PydanticAI.
    
    This fixture:
    1. Spawns the Golem MCP server via cargo
    2. Connects using PydanticAI's MCPServerStdio
    3. Verifies the server is ready
    4. Provides an Agent for testing
    """
    # Find the golem-mcp-server directory
    project_root = Path(__file__).parent.parent
    
    # Configure environment for the MCP server
    env = os.environ.copy()
    env["RUST_LOG"] = "debug"
    env["GOLEM_CLI_PATH"] = "golem-cli"  # Use system golem-cli
    
    # Create the MCP server connection
    server = MCPServerStdio(
        'cargo',
        args=['run', '--bin', 'golem-mcp-server', '--', '--stdio'],
        cwd=str(project_root),
        env=env,
    )
    
    # Create an agent with the MCP server as a toolset
    agent = Agent(
        'test-agent',
        system_prompt="You are testing the Golem MCP server",
        toolsets=[server],
    )
    
    # Start the server and verify it's ready
    async with agent:
        # List tools to verify server is ready
        tools = await server.list_tools()
        
        # Verify we have the expected 10 tools
        assert len(tools) == 10, f"Expected 10 tools, got {len(tools)}"
        
        # Verify tool names are simplified (no prefixes/suffixes)
        expected_names = {
            "component", "worker", "app", "project", "profile",
            "token", "account", "plugin", "share", "network"
        }
        tool_names = {tool.name for tool in tools}
        assert expected_names == tool_names, f"Tool names mismatch. Got: {tool_names}"
        
        yield agent


@pytest.fixture
async def timeout():
    """Fixture to set a timeout for async tests."""
    return 30  # 30 seconds timeout for operations


@pytest.fixture
def mock_component_name():
    """Generate a unique component name for testing."""
    import uuid
    return f"test-component-{uuid.uuid4().hex[:8]}"


@pytest.fixture
def mock_worker_name():
    """Generate a unique worker name for testing."""
    import uuid
    return f"test-worker-{uuid.uuid4().hex[:8]}"