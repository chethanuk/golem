# Golem MCP Server Python E2E Tests

End-to-end tests for the Golem MCP Server using PydanticAI.

## Setup

1. Install Python dependencies:
```bash
pip install -r requirements.txt
```

2. Build the Golem MCP Server:
```bash
cd ..
cargo build --bin golem-mcp-server
```

## Running Tests

Run all tests:
```bash
pytest -v
```

Run specific test:
```bash
pytest -v test_e2e.py::test_profile_operations
```

Run with debug output:
```bash
RUST_LOG=debug pytest -v -s
```

## Test Structure

Following KISS principle, we have only 2 test files:

- `conftest.py` - PydanticAI fixtures and setup
- `test_e2e.py` - All E2E tests in one file

## How It Works

1. The `mcp_agent` fixture spawns the Golem MCP server via cargo
2. PydanticAI's `MCPServerStdio` connects to the server
3. Tests use natural language to invoke MCP tools
4. The agent interprets requests and calls appropriate tools

## Current Limitations

These tests work with the current architecture where the MCP server
calls the golem-cli binary via subprocess. When the CLI is not available,
mock responses are returned.

For production testing, you need:
- Running Golem instance
- Valid authentication
- Test components and workers