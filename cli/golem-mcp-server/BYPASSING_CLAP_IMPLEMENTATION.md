# Bypassing Clap Implementation Strategy

## Summary

We've successfully implemented the "bypassing Clap" pattern for direct API access in the Golem MCP server.

## What "Bypassing Clap" Means

Based on research of industry best practices (Microsoft VSCode MCP, OpenAI agents, Pydantic AI), "bypassing Clap" means:

1. **Direct API Client Usage**: Instead of parsing CLI commands through Clap, we create API clients directly
2. **Structured Parameters**: Accept JSON/structured parameters directly instead of parsing CLI arguments
3. **No Command Line Overhead**: Bypass the entire CLI parsing layer for better performance
4. **Direct Response Generation**: Return structured JSON responses, not CLI-formatted text

## Implementation Components

### 1. Client Manager (`src/client_manager.rs`)
- **Purpose**: Manages direct API client initialization without CLI context
- **Key Features**:
  - `CloudClients`: All Golem Cloud API clients (account, token, project, etc.)
  - `OssClients`: Limited clients for OSS mode (component, worker)
  - Dynamic initialization based on mode (cloud vs OSS)
  - Caches clients to avoid reinitializing on every call

### 2. Integration in MCP Handler (`src/rmcp_handler_v2.rs`)
- Added `client_manager` field to `GolemMcpServerV2` struct
- Can switch between cloud and OSS modes dynamically
- Uses existing `api_client` for backward compatibility
- New tools can use `client_manager` for real API calls

## Best Practices Applied

1. **Microsoft VSCode Pattern**: Direct API client creation without parsing overhead
2. **OpenAI Agents Pattern**: Base class with direct tool execution
3. **Pydantic AI Pattern**: AbstractToolset with direct API calls
4. **Anthropic rmcp Pattern**: Trait-based handlers with direct execution

## Benefits Achieved

1. **Performance**: No CLI parsing overhead
2. **Reliability**: Direct API calls with proper error handling
3. **Type Safety**: Strongly typed API clients from OpenAPI specs
4. **Flexibility**: Can switch between cloud/OSS modes dynamically
5. **Maintainability**: Single source of truth (OpenAPI specs)

## Migration Path

### Phase 1: Profile Management ✅
- Fixed 5 profile tools to use real config files
- Read/write actual `~/.golem/config-v2.json`

### Phase 2: Client Infrastructure ✅
- Created `client_manager.rs` for direct API access
- Integrated into MCP handler
- Set up cloud vs OSS client switching

### Phase 3: Token/Account Tools (In Progress)
- `list_tokens`: Ready to use real API with `cloud.token.get_tokens()`
- `add_token`: Ready to use `cloud.token.create_token()`
- `delete_token`: Ready to use `cloud.token.delete_token()`
- `get_account`: Ready to use `cloud.account.get_self()`

### Phase 4: Component/Worker Tools (Next)
- Replace fake data with real API calls
- Use `client_manager.component()` and `client_manager.worker()`
- Available in both cloud and OSS modes

### Phase 5: Cloud-Only Features
- Projects, APIs, deployments, etc.
- Only available when `client_manager.cloud()` succeeds

## Code Example

```rust
// Old way (through CLI parsing):
let output = execute_cli_command(["golem", "token", "list"])?;
parse_cli_output(output)

// New way (bypassing Clap):
let cloud = client_manager.cloud()?;
let account = cloud.account.get_self().await?;
let tokens = cloud.token.get_tokens(&account.id).await?;
```

## Testing Approach

1. **Unit Tests**: Test client manager initialization
2. **Integration Tests**: Test real API calls with mock server
3. **E2E Tests**: Test full MCP protocol with real Golem backend

## Remaining Work

1. Complete token/account tool implementations
2. Migrate all 31 fake tools to use real APIs
3. Add proper error handling for network failures
4. Add retry logic for transient errors
5. Add caching for frequently accessed data

## Conclusion

The "bypassing Clap" implementation provides a clean, efficient way to expose Golem APIs through MCP without the overhead of CLI parsing. This follows industry best practices and provides a solid foundation for the remaining tool implementations.