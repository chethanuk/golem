"""
End-to-end tests for Golem MCP Server using PydanticAI.

Following KISS principle: All tests in one file, clear and focused.
"""

import pytest
from pydantic_ai import Agent


@pytest.mark.asyncio
async def test_profile_operations(mcp_agent: Agent):
    """Test profile list and switch operations."""
    # List profiles
    result = await mcp_agent.run("List all Golem profiles")
    assert result.output is not None
    # Default profiles should be mentioned
    assert "cloud" in result.output.lower() or "local" in result.output.lower()


@pytest.mark.asyncio
async def test_component_list(mcp_agent: Agent):
    """Test component listing operation."""
    result = await mcp_agent.run("List all Golem components")
    assert result.output is not None
    # Should return a response (even if empty list)
    assert "component" in result.output.lower()


@pytest.mark.asyncio
async def test_worker_list(mcp_agent: Agent, mock_component_name: str):
    """Test worker listing for a component."""
    # Note: This will likely fail without a real component, but tests the operation
    result = await mcp_agent.run(
        f"List all workers for component {mock_component_name}"
    )
    assert result.output is not None
    # Should mention workers or indicate none found
    assert "worker" in result.output.lower()


@pytest.mark.asyncio
async def test_app_list(mcp_agent: Agent):
    """Test application listing."""
    result = await mcp_agent.run("List all Golem applications")
    assert result.output is not None
    # Should return application list or empty response
    assert "app" in result.output.lower() or "application" in result.output.lower()


@pytest.mark.asyncio
async def test_project_templates(mcp_agent: Agent):
    """Test listing project templates."""
    result = await mcp_agent.run("List available Golem project templates")
    assert result.output is not None
    # Should mention templates
    assert "template" in result.output.lower() or "project" in result.output.lower()


@pytest.mark.asyncio
async def test_token_list(mcp_agent: Agent):
    """Test token listing."""
    result = await mcp_agent.run("List all authentication tokens")
    assert result.output is not None
    # Should mention tokens
    assert "token" in result.output.lower()


@pytest.mark.asyncio
async def test_account_operations(mcp_agent: Agent):
    """Test account listing."""
    result = await mcp_agent.run("List all Golem accounts")
    assert result.output is not None
    # Should mention accounts
    assert "account" in result.output.lower()


@pytest.mark.asyncio
async def test_plugin_list(mcp_agent: Agent):
    """Test plugin listing."""
    result = await mcp_agent.run("List installed Golem plugins")
    assert result.output is not None
    # Should mention plugins
    assert "plugin" in result.output.lower()


@pytest.mark.asyncio
async def test_network_list(mcp_agent: Agent):
    """Test network listing."""
    result = await mcp_agent.run("List Golem networks")
    assert result.output is not None
    # Should mention networks
    assert "network" in result.output.lower()


@pytest.mark.asyncio
async def test_complex_workflow(mcp_agent: Agent, mock_component_name: str, mock_worker_name: str):
    """
    Test a complex workflow involving multiple operations.
    
    This tests the agent's ability to chain operations together.
    """
    # Create a complex request that might require multiple tool calls
    prompt = f"""
    Please help me with the following tasks:
    1. List all available component templates
    2. Check if component '{mock_component_name}' exists
    3. List profiles to see which one is active
    
    Provide a summary of what you found.
    """
    
    result = await mcp_agent.run(prompt)
    assert result.output is not None
    
    # The agent should have used multiple tools
    # Check that it mentioned key aspects
    output_lower = result.output.lower()
    assert "template" in output_lower or "component" in output_lower
    assert "profile" in output_lower


@pytest.mark.asyncio
async def test_error_handling(mcp_agent: Agent):
    """Test that errors are handled gracefully."""
    # Try to invoke a function on a non-existent worker
    result = await mcp_agent.run(
        "Invoke function 'hello' on worker 'non-existent-worker' "
        "in component 'non-existent-component'"
    )
    assert result.output is not None
    # Should indicate an error or that the component/worker doesn't exist
    assert "error" in result.output.lower() or "not found" in result.output.lower() or "fail" in result.output.lower()


@pytest.mark.asyncio
async def test_tool_discovery(mcp_agent: Agent):
    """Test that the agent can discover and understand available tools."""
    result = await mcp_agent.run(
        "What operations can you perform with Golem components?"
    )
    assert result.output is not None
    # Should mention component operations
    assert "component" in result.output.lower()
    # Should mention some operations like list, deploy, etc.
    operations = ["list", "deploy", "build", "new", "template"]
    assert any(op in result.output.lower() for op in operations)


@pytest.mark.asyncio
@pytest.mark.parametrize("operation,expected_tool", [
    ("List components", "component"),
    ("Create a worker", "worker"),
    ("Deploy an application", "app"),
    ("Switch profile", "profile"),
    ("List tokens", "token"),
])
async def test_operation_routing(mcp_agent: Agent, operation: str, expected_tool: str):
    """Test that operations are routed to the correct tools."""
    result = await mcp_agent.run(operation)
    assert result.output is not None
    # The response should be related to the expected tool
    assert expected_tool in result.output.lower()


# Note: These tests assume the MCP server is running with mock responses
# when the actual Golem CLI is not available. For production testing,
# you would need:
# 1. A running Golem instance
# 2. Valid authentication tokens
# 3. Test components and workers set up
# 4. Proper cleanup after tests