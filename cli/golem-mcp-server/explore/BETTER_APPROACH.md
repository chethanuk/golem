# Better Approach: Leverage Existing CLI Infrastructure

## The Problem with Current Implementation

We've been reimplementing everything from scratch when the CLI already has:
- All command definitions with Clap metadata
- Command handlers that do the actual work
- Context management with proper client initialization
- Error handling and validation

## The Better Approach: CLI Bridge

### Core Idea

Instead of bypassing Clap entirely, we can:
1. **Extract tool metadata from Clap** - Use Clap's command structure to auto-generate MCP tools
2. **Execute via command handlers** - Call the existing handlers directly
3. **Transform output** - Convert CLI output to MCP JSON format

### Implementation Strategy

```rust
// Instead of reimplementing each tool:
pub async fn list_components(&self) -> Result<CallToolResult, McpError> {
    // ... 50+ lines of custom implementation
}

// We can do:
pub async fn execute_cli_command(&self, command: &str, args: Vec<String>) -> Result<CallToolResult, McpError> {
    // Parse command using existing CLI parser
    let cmd = GolemCliCommand::try_parse_from(args)?;
    
    // Create context (reuse CLI's context initialization)
    let ctx = Context::new(self.config)?;
    
    // Execute using existing handler
    let result = match cmd.subcommand {
        GolemCliSubcommand::Component { subcommand } => {
            ctx.component_handler().handle_command(subcommand).await
        }
        // ... other subcommands
    }
    
    // Convert output to MCP format
    Ok(CallToolResult::success(vec![Content::json(result)?]))
}
```

### Benefits

1. **No duplication** - Reuse all existing CLI logic
2. **Always in sync** - Changes to CLI automatically reflected in MCP
3. **Complete coverage** - All CLI commands available as MCP tools
4. **Proper validation** - Clap handles all argument validation
5. **Documentation** - Clap help text becomes tool descriptions

### Tool Generation from Clap

```rust
fn generate_mcp_tools() -> Vec<Tool> {
    let command = GolemCliCommand::command();
    let mut tools = Vec::new();
    
    for subcommand in command.get_subcommands() {
        for sub_subcommand in subcommand.get_subcommands() {
            let tool = Tool {
                name: format!("{}_{}", subcommand.get_name(), sub_subcommand.get_name()),
                description: sub_subcommand.get_about().unwrap_or_default().to_string(),
                input_schema: generate_schema_from_args(sub_subcommand),
            };
            tools.push(tool);
        }
    }
    
    tools
}
```

### Example: Component List

**Current Implementation (100+ lines):**
- Custom API client setup
- Manual parameter handling
- Custom error handling
- Custom JSON formatting

**New Implementation (10 lines):**
```rust
#[tool(description = "List components")]
pub async fn component_list(&self, params: ComponentListParams) -> Result<CallToolResult, McpError> {
    let args = vec!["golem", "component", "list"];
    if let Some(project) = params.project {
        args.extend(["--project", &project]);
    }
    
    self.execute_cli_command(args).await
}
```

### Migration Path

1. **Phase 1**: Create CLI bridge infrastructure
2. **Phase 2**: Generate tools from Clap metadata
3. **Phase 3**: Replace fake implementations with CLI calls
4. **Phase 4**: Optimize hot paths if needed

### Code Organization

```
cli/golem-mcp-server/
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Library exports
│   ├── cli_bridge.rs        # NEW: CLI command execution bridge
│   ├── tool_generator.rs    # NEW: Generate MCP tools from Clap
│   └── mcp_handler.rs       # Simplified MCP protocol handler
```

## Why This Is Better

1. **DRY Principle**: Don't Repeat Yourself - reuse existing code
2. **Maintenance**: Single source of truth for all operations
3. **Completeness**: Every CLI feature automatically available
4. **Correctness**: Tested CLI code vs. new untested implementations
5. **Documentation**: Clap help becomes MCP descriptions

## Implementation Example

```rust
// cli_bridge.rs
pub struct CliBridge {
    context: Arc<Context>,
}

impl CliBridge {
    pub async fn execute(&self, args: Vec<String>) -> Result<serde_json::Value, McpError> {
        // Parse command
        let cmd = GolemCliCommand::try_parse_from(args)
            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
        
        // Capture output
        let output = CapturedOutput::new();
        
        // Execute command
        self.context.handle_command(cmd).await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        
        // Return captured output as JSON
        Ok(output.to_json())
    }
}
```

## Conclusion

Instead of reimplementing 46+ tools with fake data, we should:
1. Leverage the existing CLI infrastructure
2. Auto-generate MCP tools from Clap metadata
3. Execute commands through existing handlers
4. Transform output to MCP format

This gives us **real implementations** with **minimal code** and **zero maintenance burden**.